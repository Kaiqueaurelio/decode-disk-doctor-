//! Central preflight rules for operations that can overwrite storage.
//! Validation here does not execute commands or grant OS-level permissions.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EndpointKind { BlockDevice, ImageFile }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageEndpoint {
    pub kind: EndpointKind,
    pub path: String,
    pub stable_identity: Option<String>,
    pub capacity_bytes: u64,
    pub is_system: bool,
    pub is_mounted: bool,
    pub is_read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobKind { FileBackup, DiskImage, BlockClone, Restore, FileRecovery }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationPlan {
    pub job_kind: JobKind,
    pub source: StorageEndpoint,
    pub destination: StorageEndpoint,
    pub requested_bytes: u64,
}

impl OperationPlan {
    pub fn fingerprint(&self) -> Result<String, PreflightError> {
        let bytes = serde_json::to_vec(self).map_err(|_| PreflightError::InvalidPlan)?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }

    pub fn confirmation_phrase(&self) -> Result<String, PreflightError> {
        let fingerprint = self.fingerprint()?;
        Ok(format!("DECODE-{}", &fingerprint[..12]).to_ascii_uppercase())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanConfirmation {
    pub plan_fingerprint: String,
    pub phrase: String,
}

impl PlanConfirmation {
    pub fn confirm(plan: &OperationPlan, phrase: &str) -> Result<Self, PreflightError> {
        let expected = plan.confirmation_phrase()?;
        if phrase != expected { return Err(PreflightError::ConfirmationMismatch); }
        Ok(Self { plan_fingerprint: plan.fingerprint()?, phrase: phrase.to_string() })
    }

    pub fn validate_for(&self, plan: &OperationPlan) -> Result<(), PreflightError> {
        if self.plan_fingerprint != plan.fingerprint()? || self.phrase != plan.confirmation_phrase()? {
            return Err(PreflightError::ConfirmationMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PreflightError {
    #[error("Operation plan could not be serialized.")]
    InvalidPlan,
    #[error("Source and destination must have different stable identities.")]
    SameDevice,
    #[error("A stable device identity is required.")]
    MissingIdentity,
    #[error("Destination is a system device.")]
    SystemDestination,
    #[error("Destination is mounted and cannot safely be overwritten.")]
    MountedDestination,
    #[error("Destination is read-only.")]
    ReadOnlyDestination,
    #[error("Destination capacity is insufficient.")]
    DestinationTooSmall,
    #[error("This operation requires block-device endpoints.")]
    WrongEndpointKind,
    #[error("Confirmation does not match this exact operation plan.")]
    ConfirmationMismatch,
}

pub fn validate_block_clone(plan: &OperationPlan) -> Result<(), PreflightError> {
    if plan.job_kind != JobKind::BlockClone { return Err(PreflightError::InvalidPlan); }
    if plan.source.kind != EndpointKind::BlockDevice || plan.destination.kind != EndpointKind::BlockDevice {
        return Err(PreflightError::WrongEndpointKind);
    }
    validate_common(plan)
}

pub fn validate_restore(plan: &OperationPlan) -> Result<(), PreflightError> {
    if plan.job_kind != JobKind::Restore { return Err(PreflightError::InvalidPlan); }
    if plan.destination.kind != EndpointKind::BlockDevice {
        return Err(PreflightError::WrongEndpointKind);
    }
    validate_common(plan)
}

fn validate_common(plan: &OperationPlan) -> Result<(), PreflightError> {
    let source = plan.source.stable_identity.as_deref().filter(|s| !s.trim().is_empty())
        .ok_or(PreflightError::MissingIdentity)?;
    let destination = plan.destination.stable_identity.as_deref().filter(|s| !s.trim().is_empty())
        .ok_or(PreflightError::MissingIdentity)?;
    if source == destination { return Err(PreflightError::SameDevice); }
    if plan.destination.is_system { return Err(PreflightError::SystemDestination); }
    if plan.destination.is_mounted { return Err(PreflightError::MountedDestination); }
    if plan.destination.is_read_only { return Err(PreflightError::ReadOnlyDestination); }
    if plan.destination.capacity_bytes < plan.requested_bytes {
        return Err(PreflightError::DestinationTooSmall);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint(id: &str, capacity: u64) -> StorageEndpoint {
        StorageEndpoint {
            kind: EndpointKind::BlockDevice, path: format!("/dev/{id}"),
            stable_identity: Some(id.into()), capacity_bytes: capacity,
            is_system: false, is_mounted: false, is_read_only: false,
        }
    }

    fn plan() -> OperationPlan {
        OperationPlan { job_kind: JobKind::BlockClone, source: endpoint("source", 1000),
            destination: endpoint("destination", 1200), requested_bytes: 1000 }
    }

    #[test]
    fn accepts_safe_clone_preflight() { assert!(validate_block_clone(&plan()).is_ok()); }

    #[test]
    fn rejects_same_device() {
        let mut p = plan();
        p.destination.stable_identity = p.source.stable_identity.clone();
        assert_eq!(validate_block_clone(&p), Err(PreflightError::SameDevice));
    }

    #[test]
    fn confirmation_is_bound_to_exact_plan() {
        let p = plan();
        let phrase = p.confirmation_phrase().unwrap();
        let confirmation = PlanConfirmation::confirm(&p, &phrase).unwrap();
        confirmation.validate_for(&p).unwrap();
        let mut changed = p.clone();
        changed.requested_bytes -= 1;
        assert_eq!(confirmation.validate_for(&changed), Err(PreflightError::ConfirmationMismatch));
    }

    #[test]
    fn blocks_mounted_system_and_small_targets() {
        let mut p = plan(); p.destination.is_mounted = true;
        assert_eq!(validate_block_clone(&p), Err(PreflightError::MountedDestination));
        let mut p = plan(); p.destination.is_system = true;
        assert_eq!(validate_block_clone(&p), Err(PreflightError::SystemDestination));
        let mut p = plan(); p.destination.capacity_bytes = 10;
        assert_eq!(validate_block_clone(&p), Err(PreflightError::DestinationTooSmall));
    }
}
