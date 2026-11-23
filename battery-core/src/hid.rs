//! HID discovery and polling sessions.
use crate::{
    BatteryCapability, ConnectionKind, Descriptor, DeviceCategory, DeviceIdentity, Kind,
    ModelSource, Reading, codec, sinowealth,
};
use hidapi::HidApi;
use serde::{Deserialize, Serialize};

/// Non-fatal discovery problems surfaced in the panel (upstream vendors; never produced here).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiscoveryWarning {
    LogiLongEndpoint(u16),
    LogiDeviceType,
    OmniPairingPort,
    OmniPairingList,
    OmniDeviceEndpoint,
}

/// A receiver-connected keyboard physically present in the latest inventory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyboardPresence {
    pub id: String,
    pub serial: Option<String>,
}

/// Result of one complete HID scan.
pub struct Inventory {
    pub targets: Vec<Target>,
    pub keyboards: Vec<KeyboardPresence>,
    pub unpaired: Vec<Descriptor>,
    pub warnings: Vec<DiscoveryWarning>,
}

/// Identity change observed while polling (none of the implemented protocols emit one).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityUpdate {
    pub identity: DeviceIdentity,
}

/// Stable logical id: the same mouse keeps its name and alias across receiver and cable.
const SINOWEALTH_ID: &str = "hid:258a:002f:Mouse:sinowealth";

/// A pollable HID device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub info: Descriptor,
    serial: Option<String>,
}

impl Target {
    fn sinowealth(endpoints: &[sinowealth::Endpoint]) -> Self {
        let wired = endpoints.iter().all(|e| e.pid == sinowealth::PID_WIRED);
        Self {
            info: Descriptor {
                id: SINOWEALTH_ID.into(),
                name: "ThunderRobot ML701 Pro".into(),
                kind: Kind::Mouse,
                legacy_ids: Vec::new(),
                identity: Some(DeviceIdentity {
                    model_key: Some("sinowealth:258a:002f".into()),
                    category: DeviceCategory::Mouse,
                    connection: if wired { ConnectionKind::Usb } else { ConnectionKind::Rf24 },
                    battery: BatteryCapability::Present,
                    model_source: Some(ModelSource::CatalogRule),
                }),
                generic_name: None,
            },
            serial: endpoints.iter().find_map(|e| e.serial.clone()),
        }
    }

    pub fn same_endpoint(&self, prior: &Self) -> bool {
        self.info.id == prior.info.id
    }

    pub fn retain_identity(&mut self, prior: &Self) {
        if self.same_endpoint(prior) && self.info.identity.is_none() {
            self.info.identity.clone_from(&prior.info.identity);
        }
    }

    /// The physical peripheral behind the endpoint changed (samples must not carry over).
    pub fn association_changed_from(&self, prior: &Self) -> bool {
        !self.same_endpoint(prior)
    }

    pub fn open(&self) -> Result<Session, String> {
        let api = HidApi::new().map_err(|e| format!("HID init: {e}"))?;
        if sinowealth::endpoints(&api).is_empty() {
            return Err("Receiver is not connected".into());
        }
        Ok(Session { api, named: false })
    }

    /// Look the device up again after its endpoint disappeared.
    pub fn rediscover(&self) -> Result<Option<Target>, String> {
        let api = HidApi::new().map_err(|e| format!("HID init: {e}"))?;
        let found = sinowealth::endpoints(&api);
        Ok((!found.is_empty()).then(|| Target::sinowealth(&found)))
    }

    pub fn apply_identity_update(&mut self, update: IdentityUpdate) -> bool {
        let changed = self.info.identity.as_ref() != Some(&update.identity);
        self.info.identity = Some(update.identity);
        changed
    }

    /// Record a name reported by the device; returns whether the association changed.
    pub fn apply_legacy_name(&mut self, name: String) -> bool {
        if !self.info.legacy_ids.contains(&name) {
            self.info.legacy_ids.push(name);
        }
        false
    }

    pub fn serial(&self) -> Option<&str> {
        self.serial.as_deref()
    }
}

/// An open polling session.
pub struct Session {
    api: HidApi,
    named: bool,
}

impl Session {
    pub fn reset_filter(&mut self) {}

    pub fn reset_identity_query(&mut self) {}

    /// Read the battery. `_now_ms` is the caller's monotonic clock (unused here).
    pub fn read(&mut self, _now_ms: u64) -> Result<Reading, String> {
        // Endpoints come and go when the cable is plugged in; re-enumerate every poll.
        if let Err(e) = self.api.refresh_devices() {
            return Err(format!("HID refresh: {e}"));
        }
        let mut endpoints = sinowealth::endpoints(&self.api);
        if endpoints.is_empty() {
            return Err("Receiver is not connected".into());
        }
        // With the cable in, the mouse turns its radio off and the receiver reports
        // "asleep"; the wired node is then the only honest source.
        endpoints.sort_by_key(|e| e.pid != sinowealth::PID_WIRED);
        let mut last = String::new();
        for e in &endpoints {
            match sinowealth::query(&self.api, &e.path).and_then(|r| codec::sinowealth_status(&r)) {
                Ok(r) => return Ok(r),
                Err(err) => last = err,
            }
        }
        Err(last)
    }

    pub fn take_identity_update(&mut self) -> Option<IdentityUpdate> {
        None
    }

    /// A device-reported name, offered once per session.
    pub fn name_once(&mut self) -> Option<String> {
        self.named = true;
        None
    }
}

/// Scan all supported HID devices.
pub fn discover_inventory_with_diagnostics() -> Result<Inventory, String> {
    let api = HidApi::new().map_err(|e| format!("HID init: {e}"))?;
    let found = sinowealth::endpoints(&api);
    let targets = if found.is_empty() {
        Vec::new()
    } else {
        vec![Target::sinowealth(&found)]
    };
    Ok(Inventory {
        targets,
        keyboards: Vec::new(),
        unpaired: Vec::new(),
        warnings: Vec::new(),
    })
}

/// Bluetooth devices reachable only through a vendor HID channel (none implemented).
pub fn discover_private_bluetooth() -> Result<Vec<Target>, String> {
    Ok(Vec::new())
}
