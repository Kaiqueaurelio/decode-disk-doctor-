//! Backup metadata and a conservative file-level streaming backup primitive.
//! This crate does not perform raw-device imaging or incremental snapshots.
//! It never overwrites an existing destination and never deletes source files.

use decode_core::DeviceIdentity;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;
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
        match &self.mode {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileBackupEntry {
    pub source_path: PathBuf,
    pub destination_path: PathBuf,
    pub bytes_copied: u64,
    pub sha256: String,
}

#[derive(Debug, Error)]
pub enum FileBackupError {
    #[error("Source is not a regular file or is a symbolic link.")]
    UnsupportedSource,
    #[error("Destination already exists: {0}")]
    DestinationExists(PathBuf),
    #[error("Destination parent directory does not exist.")]
    MissingDestinationParent,
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

/// Copies one regular file using bounded memory, hashes the bytes while copying,
/// fsyncs a temporary file and publishes it without replacing an existing file.
/// The temporary file is created in the destination directory so publication
/// stays on the same filesystem. This is file backup, not a block-device backup.
pub fn backup_file_streaming(source: &Path, destination: &Path) -> Result<FileBackupEntry, FileBackupError> {
    let source_meta = fs::symlink_metadata(source)?;
    if !source_meta.file_type().is_file() || source_meta.file_type().is_symlink() {
        return Err(FileBackupError::UnsupportedSource);
    }
    if destination.exists() {
        return Err(FileBackupError::DestinationExists(destination.to_path_buf()));
    }
    let parent = destination.parent().filter(|p| !p.as_os_str().is_empty())
        .ok_or(FileBackupError::MissingDestinationParent)?;
    if !parent.is_dir() {
        return Err(FileBackupError::MissingDestinationParent);
    }

    let temp_path = parent.join(format!(".decode-backup-{}.partial", Uuid::new_v4()));
    let result = copy_and_hash(source, &temp_path);
    let (bytes_copied, sha256) = match result {
        Ok(value) => value,
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            return Err(error);
        }
    };

    // hard_link fails if destination exists, preventing a race-time overwrite.
    if let Err(error) = fs::hard_link(&temp_path, destination) {
        let _ = fs::remove_file(&temp_path);
        if error.kind() == io::ErrorKind::AlreadyExists {
            return Err(FileBackupError::DestinationExists(destination.to_path_buf()));
        }
        return Err(FileBackupError::Io(error));
    }
    fs::remove_file(&temp_path)?;

    Ok(FileBackupEntry {
        source_path: source.to_path_buf(),
        destination_path: destination.to_path_buf(),
        bytes_copied,
        sha256,
    })
}

fn copy_and_hash(source: &Path, temporary_destination: &Path) -> Result<(u64, String), FileBackupError> {
    let mut input = File::open(source)?;
    let mut output = OpenOptions::new().write(true).create_new(true).open(temporary_destination)?;
    let mut buffer = [0_u8; 1024 * 1024];
    let mut hasher = Sha256::new();
    let mut total = 0_u64;

    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 { break; }
        output.write_all(&buffer[..count])?;
        hasher.update(&buffer[..count]);
        total = total.saturating_add(count as u64);
    }
    output.flush()?;
    output.sync_all()?;
    Ok((total, format!("{:x}", hasher.finalize())))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let path = std::env::temp_dir().join(format!("decode-backup-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn streams_file_and_refuses_overwrite() {
        let dir = temp_dir();
        let source = dir.join("source.bin");
        let destination = dir.join("backup.bin");
        fs::write(&source, b"Decode Disk Doctor backup fixture").unwrap();

        let entry = backup_file_streaming(&source, &destination).unwrap();
        assert_eq!(fs::read(&destination).unwrap(), fs::read(&source).unwrap());
        assert_eq!(entry.bytes_copied, 33);
        assert_eq!(entry.sha256.len(), 64);
        assert!(matches!(
            backup_file_streaming(&source, &destination),
            Err(FileBackupError::DestinationExists(_))
        ));
        assert!(source.exists(), "source must never be deleted");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_directory_as_source() {
        let dir = temp_dir();
        let destination = dir.join("backup.bin");
        assert!(matches!(
            backup_file_streaming(&dir, &destination),
            Err(FileBackupError::UnsupportedSource)
        ));
        fs::remove_dir_all(dir).unwrap();
    }
}
