//! Sinowealth vendor channel (receiver `258A:002F`, cable `258A:002E`).
//!
//! Protocol (reverse-engineered by the redragon-m910ks project and confirmed on a
//! ThunderRobot ML701 Pro): feature report `0x05` (7 data bytes) on the `FF00` collection.
//!
//! ```text
//! SET_FEATURE [05 90 00 00 00 00 00 00]
//! GET_FEATURE [05 ..] -> [05 90 link percent ...]
//! ```
//!
//! Only the read-only status command `0x90` is ever sent. The same channel also carries
//! config writes and the `0x75` bootloader command, which can soft-brick the mouse, so
//! [`query`] accepts no command byte from callers.
use hidapi::{HidApi, HidDevice};
use std::{ffi::CString, thread, time::Duration};

pub const VID: u16 = 0x258A;
pub const PID_RECEIVER: u16 = 0x002F;
pub const PID_WIRED: u16 = 0x002E;

const REPORT_ID: u8 = 0x05;
const STATUS: u8 = 0x90;

/// The vendor collection carrying feature report 0x05 on one USB node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Endpoint {
    pub pid: u16,
    pub path: CString,
    pub serial: Option<String>,
}

/// Find vendor endpoints for all supported nodes currently present.
pub fn endpoints(api: &HidApi) -> Vec<Endpoint> {
    let mut found = Vec::new();
    for info in api.device_list() {
        if info.vendor_id() != VID
            || !matches!(info.product_id(), PID_RECEIVER | PID_WIRED)
            || info.usage_page() != 0xFF00
        {
            continue;
        }
        // Several FF00 collections exist; only the one declaring feature report 0x05 answers.
        let Ok(dev) = info.open_device(api) else { continue };
        let mut desc = [0u8; 512];
        let Ok(n) = dev.get_report_descriptor(&mut desc) else { continue };
        if declares_feature_report(&desc[..n], REPORT_ID) {
            found.push(Endpoint {
                pid: info.product_id(),
                path: info.path().to_owned(),
                serial: info.serial_number().filter(|s| !s.is_empty()).map(str::to_owned),
            });
        }
    }
    found
}

/// True when the descriptor sets `Report ID (id)` followed by a Feature main item.
fn declares_feature_report(desc: &[u8], id: u8) -> bool {
    let Some(at) = desc.windows(2).position(|w| w == [0x85, id]) else {
        return false;
    };
    desc[at + 2..].iter().any(|&b| b & 0xFC == 0xB0)
}

/// Send the status command and return the raw 8-byte reply.
pub fn query(api: &HidApi, path: &CString) -> Result<[u8; 8], String> {
    let dev = api.open_path(path).map_err(|e| format!("open: {e}"))?;
    query_device(&dev)
}

fn query_device(dev: &HidDevice) -> Result<[u8; 8], String> {
    let mut last = String::from("no reply");
    for attempt in 0..3 {
        if attempt > 0 {
            thread::sleep(Duration::from_millis(40));
        }
        dev.send_feature_report(&[REPORT_ID, STATUS, 0, 0, 0, 0, 0, 0])
            .map_err(|e| format!("send: {e}"))?;
        thread::sleep(Duration::from_millis(20));
        let mut buf = [0u8; 8];
        buf[0] = REPORT_ID;
        match dev.get_feature_report(&mut buf) {
            // Byte 1 echoes the command once the device has processed it.
            Ok(_) if buf[1] == STATUS => return Ok(buf),
            Ok(_) => last = format!("stale reply {:02X?}", &buf[..4]),
            Err(e) => last = format!("read: {e}"),
        }
    }
    Err(last)
}

#[cfg(test)]
mod tests {
    use super::declares_feature_report;

    #[test]
    fn finds_only_the_feature_collection() {
        // Descriptors captured from a 258A:002F receiver.
        let col05 = [
            0x06, 0x00, 0xFF, 0x09, 0x01, 0xA1, 0x01, 0x85, 0x05, 0x19, 0x01, 0x29, 0x02, 0x15,
            0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x07, 0xB1, 0x02, 0xC0,
        ];
        let col04_input = [
            0x06, 0x00, 0xFF, 0x09, 0x01, 0xA1, 0x01, 0x85, 0x07, 0x19, 0x01, 0x29, 0x02, 0x15,
            0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x95, 0x07, 0x81, 0x00, 0xC0,
        ];
        let col03_large = [
            0x06, 0x00, 0xFF, 0x09, 0x01, 0xA1, 0x01, 0x85, 0x08, 0x19, 0x01, 0x29, 0x02, 0x15,
            0x00, 0x26, 0xFF, 0x00, 0x75, 0x08, 0x96, 0x07, 0x02, 0xB1, 0x02, 0xC0,
        ];
        assert!(declares_feature_report(&col05, 5));
        assert!(!declares_feature_report(&col04_input, 5));
        assert!(!declares_feature_report(&col03_large, 5));
    }
}
