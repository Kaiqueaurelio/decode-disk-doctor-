//! Versioned image manifest validation and streaming chunk verification.
//! This is a metadata contract, not a raw block-device imaging implementation.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{self, Read};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageChunk {
    pub index: u64,
    pub offset_bytes: u64,
    pub length_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageManifest {
    pub format: String,
    pub schema_version: u32,
    pub source_identity: String,
    pub source_capacity_bytes: u64,
    pub sector_size_bytes: u32,
    pub chunk_size_bytes: u64,
    pub total_bytes: u64,
    pub chunks: Vec<ImageChunk>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ImageFormatError {
    #[error("Unsupported image format or schema version.")]
    UnsupportedVersion,
    #[error("Invalid sector or chunk size.")]
    InvalidGeometry,
    #[error("Source identity is missing.")]
    MissingSourceIdentity,
    #[error("Chunk indexes or offsets are not contiguous.")]
    NonContiguousChunks,
    #[error("Chunk length exceeds image capacity.")]
    ExceedsCapacity,
    #[error("Manifest total does not match its chunks.")]
    TotalMismatch,
    #[error("Invalid SHA-256 digest.")]
    InvalidDigest,
    #[error("Chunk length does not match manifest.")]
    LengthMismatch,
    #[error("Chunk digest does not match manifest.")]
    DigestMismatch,
    #[error("I/O error while verifying chunk: {0}")]
    Io(String),
}

impl ImageManifest {
    pub fn validate(&self) -> Result<(), ImageFormatError> {
        if self.format != "decode-disk-image" || self.schema_version != 1 {
            return Err(ImageFormatError::UnsupportedVersion);
        }
        if self.source_identity.trim().is_empty() {
            return Err(ImageFormatError::MissingSourceIdentity);
        }
        if self.sector_size_bytes == 0
            || self.chunk_size_bytes == 0
            || self.chunk_size_bytes % u64::from(self.sector_size_bytes) != 0
        {
            return Err(ImageFormatError::InvalidGeometry);
        }
        let mut expected_offset = 0_u64;
        let mut total = 0_u64;
        for (index, chunk) in self.chunks.iter().enumerate() {
            if chunk.index != index as u64 || chunk.offset_bytes != expected_offset {
                return Err(ImageFormatError::NonContiguousChunks);
            }
            if chunk.length_bytes == 0
                || chunk.length_bytes > self.chunk_size_bytes
                || chunk.offset_bytes % u64::from(self.sector_size_bytes) != 0
                || chunk.length_bytes % u64::from(self.sector_size_bytes) != 0
            {
                return Err(ImageFormatError::InvalidGeometry);
            }
            if chunk.sha256.len() != 64 || !chunk.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(ImageFormatError::InvalidDigest);
            }
            expected_offset = expected_offset.checked_add(chunk.length_bytes)
                .ok_or(ImageFormatError::ExceedsCapacity)?;
            total = total.checked_add(chunk.length_bytes).ok_or(ImageFormatError::ExceedsCapacity)?;
            if expected_offset > self.source_capacity_bytes { return Err(ImageFormatError::ExceedsCapacity); }
        }
        if total != self.total_bytes { return Err(ImageFormatError::TotalMismatch); }
        Ok(())
    }
}

/// Reads at most the declared chunk length using a 1 MiB buffer. The caller must
/// provide a reader scoped to exactly one chunk; extra trailing bytes are ignored.
pub fn verify_chunk_reader<R: Read>(reader: &mut R, chunk: &ImageChunk) -> Result<(), ImageFormatError> {
    let mut buffer = [0_u8; 1024 * 1024];
    let mut remaining = chunk.length_bytes;
    let mut total = 0_u64;
    let mut hasher = Sha256::new();
    while remaining > 0 {
        let limit = usize::try_from(remaining.min(buffer.len() as u64)).unwrap_or(buffer.len());
        let count = reader.read(&mut buffer[..limit]).map_err(|e: io::Error| ImageFormatError::Io(e.to_string()))?;
        if count == 0 { return Err(ImageFormatError::LengthMismatch); }
        hasher.update(&buffer[..count]);
        total += count as u64;
        remaining -= count as u64;
    }
    if total != chunk.length_bytes { return Err(ImageFormatError::LengthMismatch); }
    let digest = format!("{:x}", hasher.finalize());
    if digest != chunk.sha256.to_ascii_lowercase() { return Err(ImageFormatError::DigestMismatch); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn validates_contiguous_manifest_and_hashes_stream() {
        let bytes = b"chunk-data";
        let digest = format!("{:x}", Sha256::digest(bytes));
        let chunk = ImageChunk { index: 0, offset_bytes: 0, length_bytes: bytes.len() as u64, sha256: digest };
        let manifest = ImageManifest { format: "decode-disk-image".into(), schema_version: 1,
            source_identity: "fixture-device".into(), source_capacity_bytes: 1024,
            sector_size_bytes: 512, chunk_size_bytes: 512, total_bytes: bytes.len() as u64,
            chunks: vec![chunk.clone()] };
        manifest.validate().unwrap();
        verify_chunk_reader(&mut Cursor::new(bytes), &chunk).unwrap();
    }

    #[test]
    fn rejects_missing_identity_and_oversized_chunks() {
        let digest = "a".repeat(64);
        let mut manifest = ImageManifest {
            format: "decode-disk-image".into(),
            schema_version: 1,
            source_identity: "   ".into(),
            source_capacity_bytes: 4096,
            sector_size_bytes: 512,
            chunk_size_bytes: 1024,
            total_bytes: 2048,
            chunks: vec![
                ImageChunk { index: 0, offset_bytes: 0, length_bytes: 2048, sha256: digest.clone() },
            ],
        };
        assert_eq!(manifest.validate(), Err(ImageFormatError::MissingSourceIdentity));
        manifest.source_identity = "fixture-device".into();
        assert_eq!(manifest.validate(), Err(ImageFormatError::InvalidGeometry));
    }

    #[test]
    fn rejects_gap_and_wrong_digest() {
        let chunk = ImageChunk { index: 0, offset_bytes: 1, length_bytes: 1, sha256: "0".repeat(64) };
        let manifest = ImageManifest { format: "decode-disk-image".into(), schema_version: 1,
            source_identity: "fixture".into(), source_capacity_bytes: 100,
            sector_size_bytes: 512, chunk_size_bytes: 512, total_bytes: 1, chunks: vec![chunk.clone()] };
        assert_eq!(manifest.validate(), Err(ImageFormatError::NonContiguousChunks));
        let correct_offset = ImageChunk { offset_bytes: 0, ..chunk };
        assert_eq!(verify_chunk_reader(&mut Cursor::new(b"x"), &correct_offset), Err(ImageFormatError::DigestMismatch));
    }
}
