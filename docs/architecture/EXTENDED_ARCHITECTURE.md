# Extended architecture: backup, imaging, cloning and recovery

This document extends the existing architecture without replacing it.

## Product editions

- **Desktop Edition:** diagnostics, device inventory, history, reports, file backup and approved storage workflows.
- **Recovery Edition:** bootable Ubuntu Live environment for offline diagnosis, acquisition and recovery.
- **Technician Edition:** service records, customer/device history, exportable reports and controlled workflows for technicians.

## Engine responsibilities

| Engine | Responsibility | Current boundary |
|---|---|---|
| device-discovery | Enumerate devices and collect identity/capabilities | Read-only inventory |
| smart-engine | SMART/NVMe via smartmontools | Read-only subprocess; process supervision needed |
| diagnostic-engine | Normalize measurements and findings | No fabricated readings |
| backup-engine | File backup, manifests, verification and job state | Single-file streaming primitive only |
| image-format | Image manifest and chunk integrity | Metadata validation and reader verification |
| imaging-engine | Acquire a source image | Planning contract; no raw writer |
| clone-engine | Clone planning and execution | Preflight only |
| safety-policy | Centralized destination and confirmation rules | Preflight only; revalidation adapter still needed |
| recovery-engine | ddrescue-first acquisition and recovery orchestration | Session contract only |
| report-engine | Export measured findings and operation records | Initial contract |

## Data flow

The React frontend calls typed Tauri commands. Tauri validates request shape and forwards to Rust application services. Services call the appropriate engine and safety policy. Platform adapters resolve device identity and capabilities. External tools are supervised by isolated adapters. SQLite stores metadata, state transitions and audit records. Bulk image/backup data remains in user-selected storage.

## Storage identity and operation binding

An operation plan is immutable after confirmation. Its canonical serialized representation is fingerprinted with SHA-256. Any change to source, destination or parameters invalidates the confirmation. Immediately before a write, platform adapters must independently re-enumerate the devices and prove that the selected stable identities still resolve to the same physical endpoints.

The fingerprint does not prove device identity; it only binds consent to the exact plan the user reviewed.

## Image format v1

The initial manifest schema records format identifier, schema version, source identity, source capacity, sector size, chunk size, total byte count and ordered chunk entries. Each chunk records index, byte offset, length and SHA-256 digest. Validation rejects gaps, overlaps, invalid digest shape, zero-length chunks, overflow and capacity overruns.

Compression, encryption, global Merkle verification, random access index and cross-version migration are not yet implemented.

## Backup semantics

The first implementation copies one regular file using bounded streaming memory and hashes the data as it is written. It stages output in the destination directory and publishes it without overwriting an existing file. Directory traversal, metadata, ACLs, symlinks, hardlinks, sparse files, consistency snapshots, resume and retention are not yet implemented.

Do not expose incremental/differential modes as functional until a reliable filesystem/platform change-tracking strategy exists.

## External process policy

Every adapter must use executable allowlists or verified paths, direct argument arrays, restricted environment, bounded stdout/stderr, timeout, cancellation, process-tree cleanup, exit-code mapping and audit metadata. Never construct shell commands from user-controlled strings.

## Recovery priority

When a device appears physically unstable, stop nonessential scans and prioritize a ddrescue image plus persistent mapfile on separate physical storage. Perform filesystem and file recovery on the image/copy, never on the failing source.
