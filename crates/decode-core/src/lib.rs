//! Shared domain contracts. This crate performs no disk I/O.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceIdentity {
    /// Current OS path, such as /dev/sda. Never use this alone as a long-term identity.
    pub device_path: String,
    pub stable_id: Option<String>,
    pub serial: Option<String>,
    pub model: String,
    pub capacity_bytes: u64,
}

impl DeviceIdentity {
    /// Conservative equality check used before writes.
    pub fn identifies_same_device(&self, other: &Self) -> bool {
        let left_serial = self.serial.as_deref().map(str::trim).filter(|value| !value.is_empty());
        let right_serial = other.serial.as_deref().map(str::trim).filter(|value| !value.is_empty());

        // Matching hardware serials take precedence over OS paths and topology IDs:
        // the same physical disk may be exposed through different device aliases.
        if let (Some(left), Some(right)) = (left_serial, right_serial) {
            if left == right {
                return true;
            }
        }

        if let (Some(left), Some(right)) = (
            self.stable_id.as_deref().map(str::trim).filter(|value| !value.is_empty()),
            other.stable_id.as_deref().map(str::trim).filter(|value| !value.is_empty()),
        ) {
            if left == right {
                return true;
            }
        }

        // A conflicting pair of serials is evidence that these are different devices.
        if left_serial.is_some() && right_serial.is_some() {
            return false;
        }

        // Path equality is only a conservative fallback when stronger identifiers
        // are unavailable; callers must still require a verified identity for writes.
        self.device_path == other.device_path
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationKind {
    ReadOnlyDiagnostic,
    ImageSource,
    Clone,
    Restore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPlan {
    pub id: Uuid,
    pub kind: OperationKind,
    pub source: DeviceIdentity,
    pub destination: Option<DeviceIdentity>,
    pub requested_bytes: Option<u64>,
    pub dry_run: bool,
}

impl OperationPlan {
    pub fn new(
        kind: OperationKind,
        source: DeviceIdentity,
        destination: Option<DeviceIdentity>,
        requested_bytes: Option<u64>,
    ) -> Self {
        Self { id: Uuid::new_v4(), kind, source, destination, requested_bytes, dry_run: true }
    }

    /// Validate invariants only. This does not authorize or execute a write.
    pub fn validate(&self) -> Result<(), SafetyError> {
        match &self.kind {
            OperationKind::ReadOnlyDiagnostic => {
                if self.destination.is_some() {
                    return Err(SafetyError::UnexpectedDestination);
                }
            }
            OperationKind::ImageSource | OperationKind::Clone | OperationKind::Restore => {
                let destination = self.destination.as_ref().ok_or(SafetyError::DestinationRequired)?;
                if self.source.identifies_same_device(destination) {
                    return Err(SafetyError::SourceEqualsDestination);
                }
                if let Some(required) = self.requested_bytes {
                    if destination.capacity_bytes < required {
                        return Err(SafetyError::DestinationTooSmall {
                            required,
                            available: destination.capacity_bytes,
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SafetyError {
    #[error("A destination must be selected for this operation.")]
    DestinationRequired,
    #[error("Source and destination identify the same physical device.")]
    SourceEqualsDestination,
    #[error("Destination is too small: requires {required} bytes, has {available} bytes.")]
    DestinationTooSmall { required: u64, available: u64 },
    #[error("Read-only diagnostics must not have a destination.")]
    UnexpectedDestination,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disk(path: &str, serial: &str, capacity: u64) -> DeviceIdentity {
        DeviceIdentity {
            device_path: path.into(),
            stable_id: Some(format!("sysfs:{serial}")),
            serial: Some(serial.into()),
            model: "Test Disk".into(),
            capacity_bytes: capacity,
        }
    }

    #[test]
    fn recognizes_same_physical_disk_through_different_topology_ids() {
        let mut source = disk("/dev/sda", "SERIAL-123", 1_000);
        let mut destination = disk("/dev/disk/by-id/ata-example", "SERIAL-123", 1_000);
        source.stable_id = Some("sysfs:/pci/slot-a".into());
        destination.stable_id = Some("sysfs:/pci/slot-b".into());
        assert!(source.identifies_same_device(&destination));
    }

    #[test]
    fn does_not_treat_different_serials_as_same_disk_only_by_path_alias() {
        let mut source = disk("/dev/sda", "SERIAL-A", 1_000);
        let mut destination = disk("/dev/sda", "SERIAL-B", 1_000);
        source.stable_id = None;
        destination.stable_id = None;
        assert!(!source.identifies_same_device(&destination));
    }

    #[test]
    fn rejects_same_device_even_when_path_alias_differs() {
        let source = disk("/dev/sda", "ABC123", 1_000);
        let destination = disk("/dev/disk/by-id/ABC123", "ABC123", 1_000);
        let plan = OperationPlan::new(OperationKind::Clone, source, Some(destination), Some(500));
        assert_eq!(plan.validate(), Err(SafetyError::SourceEqualsDestination));
    }

    #[test]
    fn rejects_small_destination() {
        let plan = OperationPlan::new(
            OperationKind::ImageSource,
            disk("/dev/sda", "SRC", 1_000),
            Some(disk("/dev/sdb", "DST", 400)),
            Some(500),
        );
        assert!(matches!(plan.validate(), Err(SafetyError::DestinationTooSmall { .. })));
    }

    #[test]
    fn allows_read_only_plan_without_destination() {
        let plan = OperationPlan::new(
            OperationKind::ReadOnlyDiagnostic,
            disk("/dev/sda", "SRC", 1_000),
            None,
            None,
        );
        assert!(plan.validate().is_ok());
        assert!(plan.dry_run);
    }
}
