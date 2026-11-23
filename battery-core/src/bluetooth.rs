//! Bluetooth battery access. Not implemented in this fork: discovery returns no devices,
//! so the "Add Bluetooth" dialog stays empty instead of failing.
use crate::{Descriptor, Reading};

/// Per-thread COM/WinRT apartment guard.
pub struct Apartment(());

impl Apartment {
    pub fn enter() -> Result<Self, String> {
        Ok(Self(()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BtTarget {
    pub info: Descriptor,
}

impl BtTarget {
    pub fn same_endpoint(&self, prior: &Self) -> bool {
        self.info.id == prior.info.id
    }

    pub fn read(&mut self) -> Result<Reading, String> {
        Err("Bluetooth battery reading is not implemented".into())
    }

    pub fn read_in(&mut self, _batch: &ReadBatch) -> Result<Reading, String> {
        self.read()
    }

    /// Returns true when a pending category lookup completed and updated `info`.
    pub fn poll_category(&mut self) -> bool {
        false
    }

    pub fn category_pending(&self) -> bool {
        false
    }
}

/// Batched reads for several Bluetooth targets.
pub struct ReadBatch(());

impl ReadBatch {
    pub fn prepare<'a>(_targets: impl Iterator<Item = &'a BtTarget>) -> Self {
        Self(())
    }
}

pub fn discover() -> Result<Vec<BtTarget>, String> {
    Ok(Vec::new())
}
