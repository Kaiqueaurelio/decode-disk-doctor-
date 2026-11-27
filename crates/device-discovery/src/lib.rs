//! Read-only Linux block-device discovery using sysfs.
//! This module does not open devices for writing or issue disk commands.

use decode_core::DeviceIdentity;
use std::{fs, io, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("Device discovery is currently supported only on Linux.")]
    UnsupportedPlatform,
    #[error("Could not read sysfs: {0}")]
    Io(#[from] io::Error),
}

pub fn discover_devices() -> Result<Vec<DeviceIdentity>, DiscoveryError> {
    #[cfg(target_os = "linux")]
    {
        discover_linux(Path::new("/sys/block"))
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err(DiscoveryError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "linux")]
fn discover_linux(root: &Path) -> Result<Vec<DeviceIdentity>, DiscoveryError> {
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
    fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn skips_virtual_devices_and_reads_capacity() {
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
}
