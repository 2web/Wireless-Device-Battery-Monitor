//! Report decoders shared with the GUI.
use crate::{Charge, Reading, Source};

/// Error text for a receiver slot with no paired peripheral (KYSONA, not implemented).
pub const KYSONA_SLOT_UNPAIRED: &str = "Receiver slot is not paired";

/// Minimal HID++ battery decoder kept for API compatibility (Logitech is not implemented).
/// `data[0]` is the level; bit 7 of the flags byte marks external power.
pub fn hidpp_reading(feature: u16, data: &[u8]) -> Option<Reading> {
    let level = *data.first()?;
    let flags = data.get(2).copied().unwrap_or(0);
    let charge = if flags & 0x80 != 0 {
        if flags & 0x01 != 0 { Charge::Full } else { Charge::Charging }
    } else {
        Charge::Discharging
    };
    Reading::new(level.min(100), charge, Source::Hidpp(feature))
}

/// Decode a Sinowealth `0x90` status reply: `[05, 90, link, percent, ...]`.
///
/// * `link == 0x11` — wireless; `percent == 0` means the mouse is asleep (unknown, not 0%).
/// * `link == 0x10` — USB cable; the mouse is charging, `percent == 0` means "not yet known".
pub fn sinowealth_status(reply: &[u8]) -> Result<Reading, String> {
    let [id, cmd, link, percent, ..] = *reply else {
        return Err("Short Sinowealth reply".into());
    };
    if id != 0x05 || cmd != 0x90 {
        return Err(format!("Unexpected Sinowealth reply {id:02X} {cmd:02X}"));
    }
    match (link, percent) {
        (0x11, 0) => Err("Mouse is asleep or out of range".into()),
        (0x11, p) => Reading::new(p, Charge::Discharging, Source::Sinowealth)
            .ok_or_else(|| format!("Invalid battery level {p}")),
        (0x10, 0) => Ok(Reading::charging_without_percentage(Source::Sinowealth)),
        (0x10, 100) => Ok(Reading::new(100, Charge::Full, Source::Sinowealth).unwrap()),
        (0x10, p) => Reading::new(p, Charge::Charging, Source::Sinowealth)
            .ok_or_else(|| format!("Invalid battery level {p}")),
        (l, _) => Err(format!("Unknown Sinowealth link state {l:02X}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wireless_level_is_discharging() {
        let r = sinowealth_status(&[5, 0x90, 0x11, 0x4C, 0, 0, 0, 0]).unwrap();
        assert_eq!((r.raw, r.charge, r.percentage_known), (76, Charge::Discharging, true));
    }

    #[test]
    fn wireless_zero_is_asleep_not_empty() {
        assert!(sinowealth_status(&[5, 0x90, 0x11, 0, 0, 0, 0, 0]).is_err());
    }

    #[test]
    fn cable_reports_charging() {
        let r = sinowealth_status(&[5, 0x90, 0x10, 0x57, 0, 0, 0, 0]).unwrap();
        assert_eq!((r.raw, r.charge), (87, Charge::Charging));
        let r = sinowealth_status(&[5, 0x90, 0x10, 0, 0, 0, 0, 0]).unwrap();
        assert!(!r.percentage_known && r.charge == Charge::Charging);
        let r = sinowealth_status(&[5, 0x90, 0x10, 100, 0, 0, 0, 0]).unwrap();
        assert_eq!(r.charge, Charge::Full);
    }

    #[test]
    fn rejects_foreign_replies() {
        assert!(sinowealth_status(&[5, 0x00, 0x65, 0x00]).is_err());
        assert!(sinowealth_status(&[5, 0x90, 0x11, 101]).is_err());
        assert!(sinowealth_status(&[5, 0x90]).is_err());
    }
}
