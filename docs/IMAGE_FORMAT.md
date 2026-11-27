# Decode Disk Image format — schema v1

Status: initial manifest contract only. No raw block acquisition or restore writer is implemented.

## Manifest fields

- `format`: `decode-disk-image`
- `schema_version`: 1
- `source_identity`: stable source identifier captured at acquisition time
- `source_capacity_bytes`: source capacity
- `sector_size_bytes`: logical sector size
- `chunk_size_bytes`: nominal chunk size
- `total_bytes`: sum of chunk lengths
- `chunks[]`: ordered entries containing index, byte offset, length and SHA-256

## Validation

Chunks must start at offset zero, have contiguous indexes and offsets, use nonzero lengths, contain 64-character hexadecimal SHA-256 digests, remain within source capacity and sum to total_bytes without integer overflow.

## Verification

The image-format crate verifies a chunk through a bounded 1 MiB streaming buffer. The caller must provide a reader scoped to the declared chunk length.

## Future schema work

Compression (evaluate Zstandard), authenticated encryption using established libraries, global integrity index, metadata versioning, partial acquisition maps, sparse ranges, random access and migration policy require separate design and tests. Never invent cryptography.
