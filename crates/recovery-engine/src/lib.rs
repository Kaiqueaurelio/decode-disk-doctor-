//! Recovery workflow state machine and ddrescue planning contracts.
//! This crate does not execute ddrescue or write to block devices.

use decode_core::DeviceIdentity;
use safety_policy::{EndpointKind, StorageEndpoint};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecoveryStage {
    Planned,
    AcquiringImage,
    ImagePaused,
    ImageCompleted,
    ScanningCopy,
    RecoveringFiles,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySession {
    pub id: Uuid,
    pub source: DeviceIdentity,
    pub image_path: Option<String>,
    pub mapfile_path: Option<String>,
    pub stage: RecoveryStage,
    pub warning: String,
}

impl RecoverySession {
    pub fn new(source: DeviceIdentity) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            image_path: None,
            mapfile_path: None,
            stage: RecoveryStage::Planned,
            warning: "For failing media, acquire an image first and perform recovery on the copy.".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DdrescuePlan {
    pub source: StorageEndpoint,
    pub destination_image_path: String,
    pub destination_storage_identity: String,
    pub mapfile_path: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DdrescuePlanError {
    #[error("The source must be a block device with a stable identity.")]
    InvalidSource,
    #[error("Image and mapfile must be absolute paths.")]
    PathsMustBeAbsolute,
    #[error("The image and mapfile paths must be different.")]
    ImageAndMapfileMustDiffer,
    #[error("The image/mapfile storage must be different from the source device.")]
    DestinationMustBeSeparate,
    #[error("Destination storage identity is required.")]
    MissingDestinationIdentity,
}

impl DdrescuePlan {
    /// Validates a plan and returns argv for a conservative first pass.
    /// The caller must still prove the image and mapfile paths resolve to the
    /// declared separate physical destination before starting a process.
    pub fn first_pass_args(&self) -> Result<Vec<String>, DdrescuePlanError> {
        if self.source.kind != EndpointKind::BlockDevice
            || self.source.stable_identity.as_deref().filter(|s| !s.trim().is_empty()).is_none()
        {
            return Err(DdrescuePlanError::InvalidSource);
        }
        if self.destination_storage_identity.trim().is_empty() {
            return Err(DdrescuePlanError::MissingDestinationIdentity);
        }
        if self.source.stable_identity.as_deref() == Some(self.destination_storage_identity.as_str()) {
            return Err(DdrescuePlanError::DestinationMustBeSeparate);
        }
        if !Path::new(&self.destination_image_path).is_absolute()
            || !Path::new(&self.mapfile_path).is_absolute()
        {
            return Err(DdrescuePlanError::PathsMustBeAbsolute);
        }
        if self.destination_image_path == self.mapfile_path {
            return Err(DdrescuePlanError::ImageAndMapfileMustDiffer);
        }
        Ok(vec![
            "--no-scrape".into(),
            self.source.path.clone(),
            self.destination_image_path.clone(),
            self.mapfile_path.clone(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> StorageEndpoint {
        StorageEndpoint { kind: EndpointKind::BlockDevice, path: "/dev/sdz".into(),
            stable_identity: Some("wwn:source".into()), capacity_bytes: 1_000,
            is_system: false, is_mounted: false, is_read_only: true }
    }

    #[test]
    fn creates_conservative_ddrescue_argv_without_execution() {
        let plan = DdrescuePlan { source: source(),
            destination_image_path: "/mnt/recovery/disk.img".into(),
            destination_storage_identity: "wwn:destination".into(),
            mapfile_path: "/mnt/recovery/disk.map".into() };
        let args = plan.first_pass_args().unwrap();
        assert_eq!(args, vec!["--no-scrape", "/dev/sdz", "/mnt/recovery/disk.img", "/mnt/recovery/disk.map"]);
    }

    #[test]
    fn rejects_same_physical_destination() {
        let plan = DdrescuePlan { source: source(),
            destination_image_path: "/mnt/recovery/disk.img".into(),
            destination_storage_identity: "wwn:source".into(),
            mapfile_path: "/mnt/recovery/disk.map".into() };
        assert_eq!(plan.first_pass_args(), Err(DdrescuePlanError::DestinationMustBeSeparate));
    }

    #[test]
    fn rejects_relative_paths() {
        let plan = DdrescuePlan { source: source(),
            destination_image_path: "disk.img".into(),
            destination_storage_identity: "wwn:destination".into(),
            mapfile_path: "/mnt/recovery/disk.map".into() };
        assert_eq!(plan.first_pass_args(), Err(DdrescuePlanError::PathsMustBeAbsolute));
    }
}
