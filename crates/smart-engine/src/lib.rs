//! SMART/NVMe adapter boundary.
//! No command is executed by this crate yet. A future adapter must pin and
//! verify smartmontools, invoke it without a shell, and parse structured output.

use decode_core::DeviceIdentity;
use diagnostic_engine::DiagnosticSnapshot;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SmartError {
    #[error("SMART adapter is not configured. Install and verify the approved smartmontools package.")]
    NotConfigured,
    #[error("Device does not expose SMART data: {0}")]
    Unsupported(String),
}

pub trait SmartProvider {
    fn inspect_read_only(&self, device: &DeviceIdentity) -> Result<DiagnosticSnapshot, SmartError>;
}

pub struct UnconfiguredSmartProvider;

impl SmartProvider for UnconfiguredSmartProvider {
    fn inspect_read_only(&self, _device: &DeviceIdentity) -> Result<DiagnosticSnapshot, SmartError> {
        Err(SmartError::NotConfigured)
    }
}
