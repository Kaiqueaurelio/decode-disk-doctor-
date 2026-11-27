//! Read-only block-device discovery for Linux and Windows.
//! Discovery reads system inventory metadata and never opens disks for writing.

use decode_core::DeviceIdentity;
use std::{io, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("Device discovery is currently supported only on Linux and Windows.")]
    UnsupportedPlatform,
    #[error("Could not read system device inventory: {0}")]
    Io(#[from] io::Error),
    #[error("Could not parse system device inventory: {0}")]
    InvalidOutput(String),
}

pub fn discover_devices() -> Result<Vec<DeviceIdentity>, DiscoveryError> {
    #[cfg(target_os = "linux")]
    {
        discover_linux(Path::new("/sys/block"))
    }
    #[cfg(target_os = "windows")]
    {
        discover_windows()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        Err(DiscoveryError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "linux")]
fn discover_linux(root: &Path) -> Result<Vec<DeviceIdentity>, DiscoveryError> {
    use std::fs;

    let mut devices = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        // Exclude virtual memory devices and loop devices from the initial physical-drive view.
        if name.starts_with("loop") || name.starts_with("ram") || name.starts_with("zram") {
            continue;
        }

        let base = entry.path();
        let sectors = read_trimmed(base.join("size").as_path())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        let capacity_bytes = sectors.saturating_mul(512);
        if capacity_bytes == 0 {
            continue;
        }

        let model = read_trimmed(base.join("device/model").as_path())
            .or_else(|| read_trimmed(base.join("device/name").as_path()))
            .unwrap_or_else(|| name.clone());
        let vendor = read_trimmed(base.join("device/vendor").as_path()).unwrap_or_default();
        let serial = read_trimmed(base.join("device/serial").as_path());
        let removable = read_trimmed(base.join("removable").as_path()).as_deref() == Some("1");
        let sysfs_canonical = fs::canonicalize(&base).unwrap_or(base.clone());
        let stable_id = Some(format!("sysfs:{}", sysfs_canonical.display()));

        let display_model = if vendor.is_empty() {
            model
        } else {
            format!("{} {}", vendor, model).trim().to_string()
        };

        devices.push(DeviceIdentity {
            device_path: format!("/dev/{name}"),
            stable_id,
            serial,
            model: if removable { format!("{display_model} (removable)") } else { display_model },
            capacity_bytes,
        });
    }

    devices.sort_by(|a, b| a.device_path.cmp(&b.device_path));
    Ok(devices)
}

#[cfg(target_os = "linux")]
fn read_trimmed(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

#[cfg(target_os = "windows")]
fn discover_windows() -> Result<Vec<DeviceIdentity>, DiscoveryError> {
    use std::process::Command;

    // Fixed PowerShell query; no user-provided value is interpolated into this script.
    let script = "$ErrorActionPreference='Stop'; Get-CimInstance -ClassName Win32_DiskDrive | Select-Object DeviceID,Model,SerialNumber,Size,PNPDeviceID | ConvertTo-Json -Compress";
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()?;

    if !output.status.success() {
        return Err(DiscoveryError::InvalidOutput(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    parse_windows_inventory(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(any(target_os = "windows", test))]
fn parse_windows_inventory(raw: &str) -> Result<Vec<DeviceIdentity>, DiscoveryError> {
    use serde_json::Value;

    let value: Value = serde_json::from_str(raw)
        .map_err(|error| DiscoveryError::InvalidOutput(error.to_string()))?;
    let rows: Vec<&Value> = match &value {
        Value::Array(items) => items.iter().collect(),
        Value::Object(_) => vec![&value],
        Value::Null => Vec::new(),
        _ => return Err(DiscoveryError::InvalidOutput("Unexpected PowerShell JSON shape.".into())),
    };

    let mut devices = Vec::new();
    for row in rows {
        let device_path = row.get("DeviceID").and_then(Value::as_str).unwrap_or_default().trim();
        if !device_path.to_ascii_lowercase().starts_with(r"\\.\physicaldrive") {
            continue;
        }

        let capacity_bytes = row.get("Size")
            .and_then(|size| size.as_u64().or_else(|| size.as_str().and_then(|s| s.parse::<u64>().ok())))
            .unwrap_or(0);
        if capacity_bytes == 0 {
            continue;
        }

        let model = row.get("Model").and_then(Value::as_str).unwrap_or("Unknown disk").trim().to_string();
        let serial = row.get("SerialNumber").and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
        let pnp = row.get("PNPDeviceID").and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty());
        let stable_id = pnp.map(|id| format!("pnp:{id}"))
            .or_else(|| serial.as_ref().map(|id| format!("serial:{id}")));

        devices.push(DeviceIdentity {
            device_path: device_path.to_string(),
            stable_id,
            serial,
            model,
            capacity_bytes,
        });
    }

    devices.sort_by(|a, b| a.device_path.cmp(&b.device_path));
    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn skips_virtual_devices_and_reads_capacity() {
        use std::fs;

        let root = std::env::temp_dir().join(format!("decode-sysfs-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("sda/device")).unwrap();
        fs::create_dir_all(root.join("loop0")).unwrap();
        fs::write(root.join("sda/size"), "2048").unwrap();
        fs::write(root.join("sda/device/model"), "Fixture Disk").unwrap();
        fs::write(root.join("sda/device/serial"), "FIXTURE-01").unwrap();
        fs::write(root.join("loop0/size"), "2048").unwrap();

        let result = discover_linux(&root).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].device_path, "/dev/sda");
        assert_eq!(result[0].capacity_bytes, 1_048_576);
        assert_eq!(result[0].serial.as_deref(), Some("FIXTURE-01"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn parses_windows_disk_inventory_without_device_access() {
        let raw = r#"[{"DeviceID":"\\\\.\\PHYSICALDRIVE0","Model":"Example SSD","SerialNumber":"SER-01","Size":"1000000000","PNPDeviceID":"SCSI\\\\DISK&VEN_EXAMPLE"}]"#;
        let result = parse_windows_inventory(raw).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].capacity_bytes, 1_000_000_000);
        assert_eq!(result[0].serial.as_deref(), Some("SER-01"));
    }
}
