//! Backup metadata model. Scheduling, snapshots and data copying are not
//! implemented yet; do not advertise incremental backup until a reliable
//! change-tracking strategy exists for the selected filesystem.

use decode_core::DeviceIdentity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupMode {
    Full,
    Incremental,
    Differential,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub id: Uuid,
    pub source: DeviceIdentity,
    pub mode: BackupMode,
    pub parent_manifest: Option<Uuid>,
    pub created_unix_seconds: i64,
    pub consistency_note: String,
    pub verified: bool,
}

impl BackupManifest {
    pub fn validate(&self) -> Result<(), &'static str> {
        match self.mode {
            BackupMode::Full if self.parent_manifest.is_some() => {
                Err("A full backup cannot reference a parent manifest.")
            }
            BackupMode::Incremental | BackupMode::Differential if self.parent_manifest.is_none() => {
                Err("Incremental and differential backups require a parent manifest.")
            }
            _ => Ok(()),
        }
    }
}
