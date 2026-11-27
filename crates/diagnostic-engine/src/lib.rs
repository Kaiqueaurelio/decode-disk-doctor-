//! Diagnostic result contracts. Hardware probes will be implemented behind
//! read-only adapters and must never fabricate measurements.

use decode_core::DeviceIdentity;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Measurement<T> {
    Measured(T),
    Unsupported { reason: String },
    Unavailable { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSnapshot {
    pub device: DeviceIdentity,
    pub captured_at: OffsetDateTime,
    pub temperature_celsius: Measurement<f32>,
    pub power_on_hours: Measurement<u64>,
    pub reallocated_sector_count: Measurement<u64>,
    pub pending_sector_count: Measurement<u64>,
    pub overall_note: String,
}

impl DiagnosticSnapshot {
    pub fn new(device: DeviceIdentity) -> Self {
        Self {
            device,
            captured_at: OffsetDateTime::now_utc(),
            temperature_celsius: Measurement::Unavailable { reason: "No hardware probe has been run.".into() },
            power_on_hours: Measurement::Unavailable { reason: "No hardware probe has been run.".into() },
            reallocated_sector_count: Measurement::Unavailable { reason: "No hardware probe has been run.".into() },
            pending_sector_count: Measurement::Unavailable { reason: "No hardware probe has been run.".into() },
            overall_note: "Not yet analyzed. No health conclusion is available.".into(),
        }
    }
}
