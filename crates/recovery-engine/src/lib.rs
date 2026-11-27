//! Recovery workflow state machine and ddrescue planning contracts.
//! This crate does not execute ddrescue or write to block devices.

use decode_core::DeviceIdentity;
use safety_policy::{EndpointKind, StorageEndpoint};
use serde::{Deserialize, Serialize};
use std::{path::Path, process::Command};
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
    pub destination_device_path: String,
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


#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub struct DdrescueReport{pub exit_code:Option<i32>,pub image_path:String,pub mapfile_path:String}
#[derive(Debug,Error)]pub enum DdrescueExecutionError{
#[error("Source is not a Linux block device.")]InvalidSource,
#[error("Image and mapfile must be on one separate destination filesystem.")]InvalidDestination,
#[error("Image and mapfile must both exist for resume or both be new.")]IncompleteResumePair,
#[error("Could not start ddrescue: {0}")]Start(String),#[error("ddrescue failed with code {0:?}")]Failed(Option<i32>),
#[error("Linux only.")]Unsupported}
pub fn run_ddrescue_first_pass(p:&DdrescuePlan)->Result<DdrescueReport,DdrescueExecutionError>{
let args=p.first_pass_args().map_err(|e|DdrescueExecutionError::Start(e.to_string()))?;
#[cfg(target_os="linux")]{use std::os::unix::fs::{FileTypeExt,MetadataExt};
let sm=std::fs::metadata(&p.source.path).map_err(|_|DdrescueExecutionError::InvalidSource)?;
if !sm.file_type().is_block_device(){return Err(DdrescueExecutionError::InvalidSource)}
let ip=Path::new(&p.destination_image_path);let mp=Path::new(&p.mapfile_path);
let idir=ip.parent().filter(|x|!x.as_os_str().is_empty()).unwrap_or(Path::new("."));let mdir=mp.parent().filter(|x|!x.as_os_str().is_empty()).unwrap_or(Path::new("."));
let im=std::fs::metadata(idir).map_err(|_|DdrescueExecutionError::InvalidDestination)?;let mm=std::fs::metadata(mdir).map_err(|_|DdrescueExecutionError::InvalidDestination)?;
let dm=std::fs::metadata(&p.destination_device_path).map_err(|_|DdrescueExecutionError::InvalidDestination)?;
if !dm.file_type().is_block_device()||!im.is_dir()||!mm.is_dir()||im.dev()!=mm.dev()
||same_physical_disk(im.dev(),sm.rdev()).map_err(|e|DdrescueExecutionError::Start(e.to_string()))?
||!same_physical_disk(im.dev(),dm.rdev()).map_err(|e|DdrescueExecutionError::Start(e.to_string()))?{
return Err(DdrescueExecutionError::InvalidDestination)}
let ie=ip.exists();let me=mp.exists();if ie!=me{return Err(DdrescueExecutionError::IncompleteResumePair)}
if ie{let m=std::fs::symlink_metadata(ip).map_err(|_|DdrescueExecutionError::InvalidDestination)?;if !m.file_type().is_file()||m.file_type().is_symlink(){return Err(DdrescueExecutionError::InvalidDestination)}}
else{std::fs::OpenOptions::new().write(true).create_new(true).open(ip).map_err(|e|DdrescueExecutionError::Start(e.to_string()))?;}
let status=Command::new("ddrescue").args(&args).status().map_err(|e|DdrescueExecutionError::Start(e.to_string()))?;
if !status.success(){return Err(DdrescueExecutionError::Failed(status.code()))}
Ok(DdrescueReport{exit_code:status.code(),image_path:p.destination_image_path.clone(),mapfile_path:p.mapfile_path.clone()})}
#[cfg(not(target_os="linux"))]{let _=(p,args);Err(DdrescueExecutionError::Unsupported)}}

#[cfg(target_os="linux")]fn disk_root(raw:u64)->Result<std::path::PathBuf,std::io::Error>{let a=libc::major(raw as libc::dev_t)as u64;let b=libc::minor(raw as libc::dev_t)as u64;let p=std::fs::canonicalize(format!("/sys/dev/block/{a}:{b}"))?;if p.join("partition").exists(){Ok(p.parent().unwrap_or(&p).to_path_buf())}else{Ok(p)}}
#[cfg(target_os="linux")]fn same_physical_disk(a:u64,b:u64)->Result<bool,std::io::Error>{Ok(disk_root(a)?==disk_root(b)?)}
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
            destination_storage_identity: "wwn:destination".into(), destination_device_path: "/dev/sdy".into(),
            mapfile_path: "/mnt/recovery/disk.map".into() };
        let args = plan.first_pass_args().unwrap();
        assert_eq!(args, vec!["--no-scrape", "/dev/sdz", "/mnt/recovery/disk.img", "/mnt/recovery/disk.map"]);
    }

    #[test]
    fn rejects_same_physical_destination() {
        let plan = DdrescuePlan { source: source(),
            destination_image_path: "/mnt/recovery/disk.img".into(),
            destination_storage_identity: "wwn:source".into(), destination_device_path: "/dev/sdz".into(),
            mapfile_path: "/mnt/recovery/disk.map".into() };
        assert_eq!(plan.first_pass_args(), Err(DdrescuePlanError::DestinationMustBeSeparate));
    }

    #[test]
    fn rejects_relative_paths() {
        let plan = DdrescuePlan { source: source(),
            destination_image_path: "disk.img".into(),
            destination_storage_identity: "wwn:destination".into(), destination_device_path: "/dev/sdy".into(),
            mapfile_path: "/mnt/recovery/disk.map".into() };
        assert_eq!(plan.first_pass_args(), Err(DdrescuePlanError::PathsMustBeAbsolute));
    }
}
