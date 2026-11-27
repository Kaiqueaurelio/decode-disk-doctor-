# Decode Disk Doctor

**A professional disk diagnostics, imaging, backup, cloning and recovery platform by Decode Analytics.**

> Status: foundation / early development. This repository is the source of truth for the product. Hardware operations are not considered production-ready until their safety checks, integration tests and real-device validation are complete.

## Product vision

Decode Disk Doctor is a desktop and bootable recovery toolkit for technicians and advanced users. It is designed to work offline for core operations and to prioritize data preservation over speed.

### Product surfaces

- **Desktop application:** Windows and Linux, built with Tauri 2, React, TypeScript and Rust.
- **Recovery environment:** a customized Ubuntu-based live system with UEFI support and legacy BIOS support where the selected base image and hardware permit it.
- **Technician workflow:** customer/device records, service history, diagnostic reports and PDF export.

### Planned capabilities

- Device discovery and hardware inventory
- SMART and NVMe health inspection
- Temperature and health/risk summaries
- Read-only surface scanning and controlled benchmarks
- Disk and partition imaging
- Full and incremental/differential backup workflows, using a suitable snapshot/change-tracking strategy
- Disk and partition cloning
- Restore and recovery workflows
- Failing-drive acquisition using GNU ddrescue mapfiles
- File-system and file recovery integrations using TestDisk/PhotoRec
- Technician reports, audit history and operation logs

## Safety principles

1. **Read-only by default.** Discovery and diagnostics must not modify source media.
2. **Preserve first.** If a drive shows read errors or signs of failure, recommend imaging with GNU ddrescue before extended tests.
3. **Explicit destructive-operation consent.** Clone destinations, restore targets, partition operations and formatting require clear device identity confirmation and a second confirmation step.
4. **Never guess a target.** Identify devices by stable attributes (model, serial where available, capacity, transport and device path) and revalidate immediately before writes.
5. **No fake health data.** Production screens must display measured results, unavailable states or clearly labeled test fixtures—never fabricated device readings.
6. **Auditable operations.** Record the selected source/destination, operation parameters, timestamps, tool versions, exit codes and logs.
7. **Protect user data.** Avoid telemetry by default; never upload disk contents or SMART data without explicit consent.
8. **Test safely.** Automated tests use fixtures, loop devices or disposable virtual disks—not a user's real disk.

## Technology direction

- Desktop shell: Tauri 2
- UI: React + TypeScript
- Core: Rust
- Local metadata: SQLite
- External engines: smartmontools, GNU ddrescue, Partclone, TestDisk/PhotoRec and selected Rescuezilla/Clonezilla workflows
- Recovery base: Ubuntu Live, customized and reproducibly built

The exact integration method for every third-party engine (library, subprocess, packaged utility or separate recovery environment) must be documented and reviewed before distribution.

## Repository layout

```text
apps/
  desktop/             Tauri + React desktop application
  recovery-ui/         Recovery-environment user interface
  technician/          Technician workflow surface (planned)
crates/
  decode-core/         Shared domain types and orchestration
  device-discovery/    Device enumeration and identity
  diagnostic-engine/   Diagnostic workflows and safety policies
  smart-engine/        SMART/NVMe adapters
  imaging-engine/      Image acquisition and verification
  backup-engine/       Backup manifests and job orchestration
  clone-engine/        Clone planning and execution
  recovery-engine/     Recovery orchestration
  report-engine/       Report generation
integrations/
  smartmontools/
  ddrescue/
  partclone/
  testdisk/
  rescuezilla/
  clonezilla/
recovery/
  live-system/
  boot/
  packages/
docs/
  architecture/
  safety/
  development/
  release/
licenses/
  third-party/
tests/
  fixtures/
  integration/
scripts/
```

Directories will be added as their corresponding implementations land. This layout is the target architecture, not a claim that every module is already implemented.


## Current implementation status

The first development slice is committed:

- Tauri 2 + React + TypeScript desktop application scaffold
- Decode dark/blue dashboard
- Read-only Linux block-device discovery through sysfs
- Read-only smartmontools adapter using JSON output and direct process arguments
- Shared Rust domain contracts and unit tests for source/destination safety
- Initial contracts for diagnostics, imaging, backup manifests, clone planning, recovery sessions and service reports
- GitHub Actions workflow for Rust core checks and frontend compilation

This is an early foundation, not a finished disk utility. The desktop runtime and CI have not yet been verified on physical hardware. Windows device discovery, NVMe-specific validation, SQLite persistence, surface scans, benchmark, actual image/backup/clone/restore execution, PDF generation and the Ubuntu Recovery USB remain pending.

To run the desktop app, follow docs/development/DEVELOPMENT.md. The SMART feature requires smartmontools installed separately and appropriate device permissions.

## Development status

See [ROADMAP.md](ROADMAP.md), [ARCHITECTURE.md](docs/architecture/ARCHITECTURE.md), [SAFETY.md](docs/safety/SAFETY.md) and [THIRD_PARTY.md](THIRD_PARTY.md).

## Licensing

The original Decode Disk Doctor source code is intended to be proprietary to Decode Analytics. Third-party components retain their own licenses and notices. See [LICENSE](LICENSE) and [THIRD_PARTY.md](THIRD_PARTY.md).

Do not distribute a build until the exact third-party binaries, libraries, source modifications and corresponding license obligations have been reviewed. This repository does not claim that subprocess isolation automatically resolves copyleft obligations.

## Brand

**Decode Analytics**  
Decode Disk Doctor  
Copyright © 2026 Decode Analytics. All rights reserved, except where a third-party component's license states otherwise.
