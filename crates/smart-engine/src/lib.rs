//! Read-only SMART inspection through smartmontools.
//! The executable is launched directly with argument arrays; no shell is used.

use decode_core::DeviceIdentity;
use diagnostic_engine::{DiagnosticSnapshot, Measurement};
use serde_json::Value;
use std::{path::PathBuf, process::Command};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SmartError {
    #[error("SMART adapter is not configured. Install smartmontools and retry.")]
    NotConfigured,
    #[error("Invalid device path: {0}")]
    InvalidDevicePath(String),
    #[error("Could not start smartctl: {0}")]
    ProcessStart(String),
    #[error("smartctl returned no valid JSON: {0}")]
    InvalidOutput(String),
}

pub trait SmartProvider {
    fn inspect_read_only(&self, device: &DeviceIdentity) -> Result<DiagnosticSnapshot, SmartError>;
}

pub struct SmartctlProvider {
    executable: PathBuf,
}

impl Default for SmartctlProvider {
    fn default() -> Self {
        Self { executable: PathBuf::from(if cfg!(windows) { "smartctl.exe" } else { "smartctl" }) }
    }
}

impl SmartctlProvider {
    pub fn new(executable: PathBuf) -> Self {
        Self { executable }
    }
}

impl SmartProvider for SmartctlProvider {
    fn inspect_read_only(&self, device: &DeviceIdentity) -> Result<DiagnosticSnapshot, SmartError> {
        let path = device.device_path.trim();
        let valid = path.starts_with("/dev/") || path.to_ascii_lowercase().starts_with(r"\\.\physicaldrive");
        if !valid {
            return Err(SmartError::InvalidDevicePath(path.to_string()));
        }

        // smartctl --json --xall reads extended device health and information data. Never add write, test,
        // sanitize, erase or firmware-update flags in this provider.
        let output = Command::new(&self.executable)
            .arg("--json")
            .arg("--xall")
            .arg(path)
            .output()
            .map_err(|error| SmartError::ProcessStart(error.to_string()))?;

        let parsed: Value = serde_json::from_slice(&output.stdout)
            .map_err(|error| SmartError::InvalidOutput(format!("{error}; stderr: {}", String::from_utf8_lossy(&output.stderr))))?;

        let mut snapshot = DiagnosticSnapshot::new(device.clone());
        let exit_status = parsed.pointer("/smartctl/exit_status").and_then(Value::as_u64).unwrap_or(u64::from(output.status.code().unwrap_or(-1).max(0) as u32));

        let temperature = parsed.pointer("/temperature/current")
            .and_then(Value::as_f64)
            .or_else(|| parsed.pointer("/nvme_smart_health_information_log/temperature").and_then(Value::as_f64));
        snapshot.temperature_celsius = measurement_f32(temperature, "Temperature is not exposed by this device.");

        let power_on_hours = parsed.pointer("/power_on_time/hours")
            .and_then(Value::as_u64)
            .or_else(|| parsed.pointer("/nvme_smart_health_information_log/power_on_hours").and_then(Value::as_u64));
        snapshot.power_on_hours = measurement_u64(power_on_hours, "Power-on hours are not exposed by this device.");

        let attributes = parsed.pointer("/ata_smart_attributes/table").and_then(Value::as_array);
        let reallocated = attributes.and_then(|rows| find_raw_attribute(rows, 5));
        let pending = attributes.and_then(|rows| find_raw_attribute(rows, 197));
        snapshot.reallocated_sector_count = measurement_u64(reallocated, "Reallocated-sector attribute is not available.");
        snapshot.pending_sector_count = measurement_u64(pending, "Pending-sector attribute is not available.");

        snapshot.overall_note = if exit_status == 0 {
            "SMART data collected. Review individual measurements; this is not a guarantee against failure.".into()
        } else {
            format!("smartctl reported device/status flags (exit status {exit_status}). Review the complete SMART report and consider imaging before further tests.")
        };
        Ok(snapshot)
    }
}

fn find_raw_attribute(rows: &[Value], id: u64) -> Option<u64> {
    rows.iter()
        .find(|row| row.get("id").and_then(Value::as_u64) == Some(id))
        .and_then(|row| row.pointer("/raw/value").and_then(Value::as_u64))
}

fn measurement_u64(value: Option<u64>, reason: &str) -> Measurement<u64> {
    value.map(Measurement::Measured)
        .unwrap_or_else(|| Measurement::Unavailable { reason: reason.into() })
}

fn measurement_f32(value: Option<f64>, reason: &str) -> Measurement<f32> {
    value.filter(|v| v.is_finite())
        .map(|v| Measurement::Measured(v as f32))
        .unwrap_or_else(|| Measurement::Unavailable { reason: reason.into() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_attribute_raw_value() {
        let rows = vec![
            serde_json::json!({"id": 5, "raw": {"value": 12}}),
            serde_json::json!({"id": 197, "raw": {"value": 2}})
        ];
        assert_eq!(find_raw_attribute(&rows, 5), Some(12));
        assert_eq!(find_raw_attribute(&rows, 197), Some(2));
        assert_eq!(find_raw_attribute(&rows, 9), None);
    }

    #[test]
    fn rejects_non_device_path() {
        let provider = SmartctlProvider::default();
        let device = DeviceIdentity {
            device_path: "sda; rm -rf /".into(),
            stable_id: None,
            serial: None,
            model: "Bad path".into(),
            capacity_bytes: 100,
        };
        assert!(matches!(provider.inspect_read_only(&device), Err(SmartError::InvalidDevicePath(_))));
    }

    #[test]
    fn missing_measurements_are_explicitly_unavailable() {
        match measurement_u64(None, "missing") {
            Measurement::Unavailable { reason } => assert_eq!(reason, "missing"),
            _ => panic!("missing values must never be fabricated"),
        }
    }
}
