//! Open replacement for the private `mouse-battery-core` crate.
//!
//! The upstream GUI depends on a closed-source protocol crate. This fork-local crate
//! re-creates the public API surface the GUI uses, with real support for Sinowealth
//! `258A:002F` / `258A:002E` mice (e.g. ThunderRobot ML701 Pro). Every other vendor
//! listed in [`Source`] exists only so the GUI compiles; discovery never yields them.

use serde::{Deserialize, Serialize};

pub mod bluetooth;
pub mod codec;
pub mod hid;
mod sinowealth;

/// Logical device class used by the GUI for binding and tray priority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    Mouse,
    Keyboard,
    Bluetooth,
    Audio,
    Gamepad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Charge {
    #[default]
    Unknown,
    Discharging,
    Charging,
    Full,
}

pub mod catalog {
    pub mod atk {
        use serde::{Deserialize, Serialize};

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum Provider {
            BitMouse,
            Bolutek,
            KeyLinker,
            Hfd,
        }
    }
}

/// Which protocol produced a reading.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Source {
    /// Sinowealth vendor channel, command `0x90` (implemented in this crate).
    Sinowealth,
    Ble,
    Classic,
    Hidpp(u16),
    Razer,
    Rog,
    Weikav,
    Atk(catalog::atk::Provider),
    ArdorFury,
    Kysona24g,
    KysonaBluetooth,
    MchoseQhw,
    MchoseMouse,
    MchoseAudio,
    AulaLegacy,
    AulaYb,
    AulaVision,
    AulaYxt,
    AulaXxwd,
    AulaBytechMagnetic,
    AulaHfd,
    AulaSparkLink,
    AulaHx,
    AulaJm,
    AulaRlw,
    AulaBytech,
    HyperX,
    TurtleBeach,
    Scyrox,
}

/// One battery sample. `raw` is the device value, `display` the value shown to users.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reading {
    pub raw: u8,
    pub display: u8,
    #[serde(default = "yes")]
    pub percentage_known: bool,
    pub charge: Charge,
    pub source: Source,
}

fn yes() -> bool {
    true
}

impl Reading {
    /// A percentage reading; `None` when the value is not a percentage.
    pub fn new(percent: u8, charge: Charge, source: Source) -> Option<Self> {
        (percent <= 100).then_some(Self {
            raw: percent,
            display: percent,
            percentage_known: true,
            charge,
            source,
        })
    }

    /// The percentage to show, or `None` when the device did not report one.
    pub fn percentage(&self) -> Option<u8> {
        self.percentage_known.then_some(self.display)
    }

    /// The device is charging but has not reported a level yet. Never shown as 0%.
    pub fn charging_without_percentage(source: Source) -> Self {
        Self {
            raw: 0,
            display: 0,
            percentage_known: false,
            charge: Charge::Charging,
            source,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceCategory {
    Mouse,
    Keyboard,
    Headset,
    Gamepad,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionKind {
    Rf24,
    Bluetooth,
    Usb,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BatteryCapability {
    Present,
    Absent,
    Unknown,
}

/// How a model key was established; only confirmed sources drive brand badges.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelSource {
    CatalogRule,
    ProtocolIdentity,
    ProductString,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub model_key: Option<String>,
    pub category: DeviceCategory,
    pub connection: ConnectionKind,
    pub battery: BatteryCapability,
    #[serde(default)]
    pub model_source: Option<ModelSource>,
}

/// A name the GUI may localize, instead of showing `Descriptor::name` verbatim.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenericName {
    UnknownDevice,
    Receiver24g {
        brand: Option<String>,
        channel: Option<DeviceCategory>,
    },
}

/// Identity facts observed on the wire (used by upstream reconciliation tests).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdentityEvidence {
    PeripheralIds { vid: u16, pid: u16 },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Descriptor {
    pub id: String,
    pub name: String,
    pub kind: Kind,
    #[serde(default)]
    pub legacy_ids: Vec<String>,
    #[serde(default)]
    pub identity: Option<DeviceIdentity>,
    #[serde(default)]
    pub generic_name: Option<GenericName>,
}
