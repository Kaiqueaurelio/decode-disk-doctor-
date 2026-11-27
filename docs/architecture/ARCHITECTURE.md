# Architecture

## Goals

Decode Disk Doctor is a local-first disk diagnostics and recovery platform. The desktop app coordinates specialized engines; it must not become a monolithic UI that performs raw disk operations itself.

## High-level components

1. Presentation: Tauri 2 + React + TypeScript. Displays device state, explains risks and gathers explicit consent.
2. Application orchestration: Rust commands and services. Validates requests, enforces operation policies, records audit events and invokes adapters.
3. Domain core: shared device identity, capabilities, operation plans, progress, results and error types.
4. Engine adapters: isolated modules for discovery, SMART, imaging, backup, clone, recovery and reporting.
5. External tools: pinned and verified upstream tools, invoked with argument arrays (never shell-concatenated user input).
6. Persistence: SQLite for metadata, job state, settings and audit history. Disk contents remain on user-selected storage.
7. Recovery OS: reproducibly built Ubuntu Live environment with a separate recovery UI and reviewed packages.

## Workspace target

- apps/desktop: Tauri desktop application
- apps/recovery-ui: recovery environment interface
- crates/decode-core: domain contracts and shared orchestration
- crates/device-discovery: device enumeration and identity
- crates/diagnostic-engine: read-only diagnostic workflows
- crates/smart-engine: SMART/NVMe adapters
- crates/imaging-engine: acquisition and verification
- crates/backup-engine: backup plans and manifests
- crates/clone-engine: clone plans and execution
- crates/recovery-engine: recovery orchestration
- crates/report-engine: report generation
- integrations: third-party tool adapters and compliance records
- recovery: live image build, boot and package definitions

## Device identity

A device must not be identified by a transient path alone. Capture available attributes such as OS device path (for current invocation only), vendor, model and serial (when available), capacity and sector sizes, transport/bus, removable status, partition and filesystem observations, and platform-specific stable identifiers.

Re-enumerate and compare identity immediately before any operation that writes. If identity changed or is ambiguous, abort.

## Engine contract

Every operation should expose validated input parameters, source and destination identity where relevant, capability and privilege requirements, dry-run/preview result, progress events, cancellation semantics, structured result and error codes, tool/version metadata, and audit log references.

## Process boundary

External tools should be invoked with explicit executable paths and argument arrays, restricted environment variables, controlled working directories and captured stdout/stderr. Never interpolate device paths or user input into a shell command string. Validate executable provenance and version.

## Backup semantics

Do not label a backup incremental or differential unless the implementation has a reliable change-tracking or snapshot mechanism for that filesystem and platform. A sequence of full images is not incremental backup. Define manifests, consistency guarantees, retention and restore verification before exposing these terms in the UI.

## Recovery environment

Use a reproducible Ubuntu Live build. Keep package versions pinned, produce a package/license manifest, verify boot media checksums and test UEFI plus legacy BIOS where supported. The desktop app and recovery UI may share domain contracts, but the live environment must remain operable without internet access.
