//! Safe planning contracts for TestDisk analysis and PhotoRec recovery.
//! No external recovery process is launched by this crate.

use safety_policy::{EndpointKind, StorageEndpoint};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileRecoveryPlan {
    pub source: StorageEndpoint,
    pub destination: StorageEndpoint,
    pub output_directory: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FileRecoveryError {
    #[error("Source and recovery destination must have different stable identities.")]
    SameDevice,
    #[error("Stable source and destination identities are required.")]
    MissingIdentity,
    #[error("Recovery destination must be a writable, non-system location.")]
    UnsafeDestination,
    #[error("Recovery output directory must be an absolute path.")]
    InvalidOutputPath,
    #[error("TestDisk write mode is disabled; analysis-only mode is required.")]
    TestDiskWriteModeForbidden,
}

impl FileRecoveryPlan {
    pub fn validate_photorec(&self) -> Result<(), FileRecoveryError> {
        let source = self.source.stable_identity.as_deref().filter(|s| !s.trim().is_empty())
            .ok_or(FileRecoveryError::MissingIdentity)?;
        let destination = self.destination.stable_identity.as_deref().filter(|s| !s.trim().is_empty())
            .ok_or(FileRecoveryError::MissingIdentity)?;
        if source == destination { return Err(FileRecoveryError::SameDevice); }
        if self.destination.is_system || self.destination.is_mounted && self.destination.is_read_only
            || self.destination.is_read_only {
            return Err(FileRecoveryError::UnsafeDestination);
        }
        if self.output_directory.trim().is_empty() || !Path::new(&self.output_directory).is_absolute() {
            return Err(FileRecoveryError::InvalidOutputPath);
        }
        if self.source.kind != EndpointKind::BlockDevice {
            return Err(FileRecoveryError::MissingIdentity);
        }
        Ok(())
    }

    pub fn validate_testdisk_analysis(&self, write_mode: bool) -> Result<(), FileRecoveryError> {
        if write_mode { return Err(FileRecoveryError::TestDiskWriteModeForbidden); }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint(id: &str, system: bool, read_only: bool) -> StorageEndpoint {
        StorageEndpoint { kind: EndpointKind::BlockDevice, path: format!("/dev/{id}"),
            stable_identity: Some(id.into()), capacity_bytes: 1000,
            is_system: system, is_mounted: false, is_read_only: read_only }
    }

    #[test]
    fn accepts_separate_recovery_destination() {
        let plan = FileRecoveryPlan { source: endpoint("source", false, true),
            destination: endpoint("destination", false, false),
            output_directory: "/mnt/recovery/files".into() };
        assert!(plan.validate_photorec().is_ok());
        assert!(plan.validate_testdisk_analysis(false).is_ok());
    }

    #[test]
    fn rejects_recovery_to_source_and_testdisk_write_mode() {
        let plan = FileRecoveryPlan { source: endpoint("same", false, true),
            destination: endpoint("same", false, false),
            output_directory: "/mnt/recovery/files".into() };
        assert_eq!(plan.validate_photorec(), Err(FileRecoveryError::SameDevice));
        assert_eq!(plan.validate_testdisk_analysis(true), Err(FileRecoveryError::TestDiskWriteModeForbidden));
    }
}
