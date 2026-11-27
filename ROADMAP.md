# Product roadmap

This roadmap describes intended work, not completed functionality. A feature is complete only when implementation, automated tests, documentation and risk-appropriate real-device validation are recorded.

## Phase 0 — Repository and engineering foundation
- [x] Establish product scope and safety principles
- [x] Define target monorepo architecture
- [x] Add proprietary license notice for original code
- [x] Add third-party compliance register
- [x] Add CI workflow for Rust core checks and frontend compilation (branch CI still needs execution)
- [x] Add local development and setup guide
- [x] Add initial threat model
- [x] Add centralized safety preflight and exact-plan confirmation contracts
- [x] Add image manifest v1 validation and streaming chunk hash verification
- [x] Add single-file streaming backup primitive with no-overwrite publication

## Phase 1 — Read-only desktop MVP
- [x] Tauri 2 desktop shell scaffold
- [x] React + TypeScript dashboard scaffold and Decode visual system
- [x] Read-only Linux sysfs and Windows CIM/PowerShell block-device discovery (hardware validation pending)
- [x] Basic hardware inventory and device detail view
- [x] smartmontools read-only SMART adapter (supervision and hardware mapping pending)
- [ ] SQLite history and structured operation logs
- [ ] Exportable diagnostic report
- [x] Initial fixture-based and safety contract unit tests (execution pending)

## Phase 2 — Diagnostics
- [ ] Surface scan with explicit read-only mode
- [ ] Controlled read-only benchmark
- [ ] Explainable health/risk findings based only on measured signals
- [ ] Drive-failure warning and ddrescue-first workflow
- [ ] Technician/customer/equipment records

## Phase 3 — Backup, imaging and cloning
- [x] Single regular-file streaming backup primitive
- [ ] Directory backup, metadata policy, exclusions and manifests
- [ ] Backup verification and persistent job state
- [ ] Incremental/differential strategy based on supported snapshot/change tracking
- [x] Image manifest schema v1 and chunk verification
- [ ] Raw disk/partition acquisition with progress, resume and verification
- [ ] Clone planning and destination revalidation
- [ ] Restore workflow with destructive-operation safeguards

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
