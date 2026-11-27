//! Safe planning contracts for TestDisk analysis and PhotoRec recovery.
//! No external recovery process is launched by this crate.

use safety_policy::{EndpointKind, StorageEndpoint};
use serde::{Deserialize, Serialize};
use std::{path::Path, process::Command};
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
        if !matches!(self.source.kind, EndpointKind::BlockDevice | EndpointKind::ImageFile) {
            return Err(FileRecoveryError::MissingIdentity);
        }
        Ok(())
    }

    pub fn validate_testdisk_analysis(&self, write_mode: bool) -> Result<(), FileRecoveryError> {
        if write_mode { return Err(FileRecoveryError::TestDiskWriteModeForbidden); }
        Ok(())
    }
}


#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub struct PhotoRecReport{pub exit_code:Option<i32>,pub output_directory:String}
#[derive(Debug,Error)]pub enum PhotoRecExecutionError{
#[error("Source must be a block device or regular image file.")]InvalidSource,
#[error("Output folder must be on the selected separate destination device.")]InvalidDestination,
#[error("Could not start PhotoRec: {0}")]Start(String),#[error("PhotoRec failed with code {0:?}")]Failed(Option<i32>),
#[error("Linux only.")]Unsupported}
pub fn photorec_args(p:&FileRecoveryPlan)->Result<Vec<String>,FileRecoveryError>{p.validate_photorec()?;Ok(vec!["/log".into(),"/d".into(),p.output_directory.clone(),p.source.path.clone()])}
pub fn recover_with_photorec(p:&FileRecoveryPlan)->Result<PhotoRecReport,PhotoRecExecutionError>{
#[cfg(target_os="linux")]{use std::os::unix::fs::{FileTypeExt,MetadataExt};
let sm=std::fs::metadata(&p.source.path).map_err(|_|PhotoRecExecutionError::InvalidSource)?;
let sd=match p.source.kind{EndpointKind::BlockDevice if sm.file_type().is_block_device()=>sm.rdev(),
EndpointKind::ImageFile if sm.file_type().is_file()=>{let parent=Path::new(&p.source.path).parent().filter(|x|!x.as_os_str().is_empty()).unwrap_or(Path::new("."));std::fs::metadata(parent).map_err(|_|PhotoRecExecutionError::InvalidSource)?.dev()},
_=>return Err(PhotoRecExecutionError::InvalidSource)};
let dm=std::fs::metadata(&p.destination.path).map_err(|_|PhotoRecExecutionError::InvalidDestination)?;
if !dm.file_type().is_block_device(){return Err(PhotoRecExecutionError::InvalidDestination)}
let out=std::fs::metadata(&p.output_directory).map_err(|_|PhotoRecExecutionError::InvalidDestination)?;
if !out.is_dir()||!same_physical_disk(out.dev(),dm.rdev())?||same_physical_disk(out.dev(),sd)?{return Err(PhotoRecExecutionError::InvalidDestination)}
let args=photorec_args(p).map_err(|e|PhotoRecExecutionError::Start(e.to_string()))?;
let status=Command::new("photorec").args(&args).status().map_err(|e|PhotoRecExecutionError::Start(e.to_string()))?;
if !status.success(){return Err(PhotoRecExecutionError::Failed(status.code()))}
Ok(PhotoRecReport{exit_code:status.code(),output_directory:p.output_directory.clone()})}
#[cfg(not(target_os="linux"))]{let _=p;Err(PhotoRecExecutionError::Unsupported)}}

#[cfg(target_os="linux")]fn disk_root(raw:u64)->Result<std::path::PathBuf,PhotoRecExecutionError>{let a=libc::major(raw as libc::dev_t)as u64;let b=libc::minor(raw as libc::dev_t)as u64;let p=std::fs::canonicalize(format!("/sys/dev/block/{a}:{b}")).map_err(|_|PhotoRecExecutionError::InvalidDestination)?;if p.join("partition").exists(){Ok(p.parent().unwrap_or(&p).to_path_buf())}else{Ok(p)}}
#[cfg(target_os="linux")]fn same_physical_disk(a:u64,b:u64)->Result<bool,PhotoRecExecutionError>{Ok(disk_root(a)?==disk_root(b)?)}
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
        assert_eq!(photorec_args(&plan).unwrap(), vec!["/log", "/d", "/mnt/recovery/files", "/dev/source"]);
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
