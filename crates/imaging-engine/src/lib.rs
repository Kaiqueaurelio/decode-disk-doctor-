//! Image acquisition contracts. No raw disk writer is implemented yet.

use decode_core::{DeviceIdentity, OperationPlan};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageJob {
    pub id: Uuid,
    pub source: DeviceIdentity,
    pub destination_path: String,
    pub mapfile_path: Option<String>,
    pub plan: OperationPlan,
    pub verify_after_completion: bool,
}

impl ImageJob {
    pub fn validate(&self) -> Result<(), String> {
        self.plan.validate().map_err(|e| e.to_string())?;
        if self.destination_path.trim().is_empty() {
            return Err("An image destination path is required.".into());
        }
        if self.destination_path.starts_with("/dev/") {
            return Err("Image output must be a regular file on a separate destination, not a raw device path.".into());
        }
        Ok(())
    }
}
