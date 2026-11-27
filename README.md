# Decode Disk Doctor

**A professional disk diagnostics, imaging, backup, cloning and recovery platform by Decode Analytics.**

> Status: foundation / early development. The repository is the source of truth. Hardware operations are not production-ready until safety checks, integration tests and real-device validation are complete.

## Product vision

A local-first desktop and bootable recovery toolkit for technicians and advanced users, prioritizing data preservation over speed.

### Product editions
- **Desktop Edition:** Windows and Linux diagnostics, SMART/NVMe, history, reports and storage workflows.
- **Recovery Edition:** customized Ubuntu Live system for offline diagnosis, acquisition and recovery.
- **Technician Edition:** service records, device history and professional reports.

## Technology
- Tauri 2, React, TypeScript and Vite
- Rust workspace
- SQLite (planned persistence layer)
- smartmontools, GNU ddrescue, Partclone, TestDisk/PhotoRec and selected Rescuezilla/Clonezilla workflows, subject to license review

## Safety principles
1. Read-only by default.
2. Preserve failing media; image first with ddrescue before extended tests.
3. Require explicit, plan-bound confirmation for destructive operations.
4. Identify devices by stable attributes and revalidate immediately before writes.
5. Never fabricate health measurements.
6. Record operation audit events.
7. Recover files to a separate destination.
8. Test with fixtures and disposable virtual media, never a user's system disk.

## Current implementation status

The repository includes a Tauri/React desktop scaffold, Linux/Windows discovery paths, a read-only smartmontools adapter, shared domain contracts, an initial backup manifest, image/clone/recovery planning contracts and CI configuration.

The continuity branch adds:
- centralized safety preflight and SHA-256 plan-bound confirmation;
- image manifest v1 validation and streaming chunk verification;
- single-file streaming backup with SHA-256 and no-overwrite publication.

These are foundational components, not a complete backup or disk-imaging product. Directory backup, SQLite persistence, supervised external-process adapters, raw imaging, clone, restore, file recovery, Recovery ISO and hardware validation remain pending. Tests added in this branch must be run by CI or a local Rust toolchain before they can be called passing.

## Repository layout

- `apps/desktop`: Tauri + React desktop app
- `crates/decode-core`: shared domain contracts
- `crates/device-discovery`: read-only device inventory
- `crates/diagnostic-engine`, `crates/smart-engine`: diagnostics
- `crates/backup-engine`, `crates/image-format`, `crates/imaging-engine`: backup and image foundations
- `crates/clone-engine`, `crates/recovery-engine`, `crates/safety-policy`: safe operation planning
- `docs`: architecture, implementation audit, safety and product editions

## Development

See [docs/development/DEVELOPMENT.md](docs/development/DEVELOPMENT.md). Rust formatting, tests, Clippy and frontend build are required before release.

## Documentation
- [Implementation audit](docs/IMPLEMENTATION_AUDIT.md)
- [Architecture](docs/architecture/ARCHITECTURE.md)
- [Extended architecture](docs/architecture/EXTENDED_ARCHITECTURE.md)
- [Operation safety](docs/OPERATION_SAFETY.md)
- [Image format](docs/IMAGE_FORMAT.md)
- [Recovery Edition](docs/RECOVERY_EDITION.md)
- [Product editions](docs/PRODUCT_EDITIONS.md)
- [Roadmap](ROADMAP.md)
- [Third-party inventory](THIRD_PARTY.md)

## Licensing

Original Decode Disk Doctor source is intended to be proprietary to Decode Analytics. Third-party components retain their own licenses and notices. See [LICENSE](LICENSE) and [THIRD_PARTY.md](THIRD_PARTY.md). Do not distribute builds until exact third-party versions, binaries, source modifications and license obligations have been reviewed.

**Decode Analytics — Decode Disk Doctor**  
Copyright © 2026 Decode Analytics, except where third-party licenses state otherwise.
