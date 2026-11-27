//! Recovery workflow state machine contracts. Recovery tools are not invoked here.

use decode_core::DeviceIdentity;
use serde::{Deserialize, Serialize};
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
