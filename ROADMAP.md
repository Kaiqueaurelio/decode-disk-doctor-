# Product roadmap

This roadmap describes intended work, not completed functionality. A feature is complete only when implementation, automated tests, documentation and risk-appropriate real-device validation are recorded.

## Phase 0 — Repository and engineering foundation
- [x] Establish product scope and safety principles
- [x] Define target monorepo architecture
- [x] Add proprietary license notice for original code
- [x] Add third-party compliance register
- [ ] Add CI for formatting, linting, tests and security checks
- [ ] Add reproducible development environment
- [ ] Add threat model and release checklist

## Phase 1 — Read-only desktop MVP
- [ ] Tauri 2 desktop shell (Windows/Linux)
- [ ] React + TypeScript interface and Decode visual system
- [ ] Rust device discovery with stable device identity
- [ ] Hardware inventory and device detail
- [ ] SMART/NVMe read-only inspection
- [ ] Temperature reporting where supported
- [ ] SQLite history and structured operation logs
- [ ] Exportable diagnostic report
- [ ] Safe fixture-based tests

## Phase 2 — Diagnostics
- [ ] Surface scan with explicit read-only mode
- [ ] Controlled benchmark on user-selected devices
- [ ] Health/risk explanation based on measured signals
- [ ] Drive failure warning and ddrescue-first workflow
- [ ] Technician/customer/equipment records

## Phase 3 — Imaging, backup and cloning
- [ ] Image format and manifest specification
- [ ] Imaging with progress, resume and verification
- [ ] Backup job model and retention policy
- [ ] Incremental/differential strategy based on supported snapshot/change tracking
- [ ] Clone planning and source/destination confirmation
- [ ] Restore workflow with destructive-operation safeguards
- [ ] Recovery from interrupted jobs

## Phase 4 — Recovery environment
- [ ] Reproducible Ubuntu Live build
- [ ] UEFI boot validation
- [ ] Legacy BIOS validation where supported
- [ ] Hardware discovery and storage drivers
- [ ] Recovery UI
- [ ] SMART, ddrescue, TestDisk/PhotoRec and approved imaging tools
- [ ] Boot media creation and integrity verification

## Phase 5 — Technician and release
- [ ] Technician edition workflows
- [ ] PDF service reports
- [ ] Windows portable package
- [ ] Linux AppImage/TAR.GZ package
- [ ] Signed release artifacts and checksums
- [ ] Third-party license compliance audit
- [ ] Hardware compatibility matrix and release qualification

## Release gates

No destructive disk operation may be marked stable until it has a written operation contract and safety review, dry-run/preview mode, stable source and destination identity checks, explicit confirmation and a second confirmation for irreversible actions, recovery guidance, disposable virtual-media tests, and documented hardware validation.
