# Development guide

## Desktop app

The desktop workspace lives in apps/desktop. It uses Tauri 2, React, TypeScript and Vite.

### Prerequisites

- Node.js 22 LTS (or compatible current LTS) and npm
- Rust stable with rustfmt and Clippy
- Tauri 2 native prerequisites for your OS
- Linux: GTK/WebKitGTK development packages required by the official Tauri 2 setup guide

### Start

From apps/desktop:

1. Run npm install.
2. Run npm run tauri dev.

Build the frontend with npm run build. Build native bundles with npm run tauri build.

### Current supported hardware path

The initial storage discovery provider reads Linux sysfs metadata and does not open raw disks. SMART inspection invokes smartctl directly using an argument array and read-only flags. Install smartmontools separately and ensure the current user has permission to query the selected device.

The initial discovery provider does not yet enumerate Windows devices. Do not interpret an unsupported-platform response as evidence that no disks exist.

## Rust workspace

From the repository root:

- cargo fmt --all
- cargo test --workspace --exclude decode-disk-doctor-app
- cargo clippy --workspace --exclude decode-disk-doctor-app --all-targets -- -D warnings

The desktop Tauri crate is excluded from the current core test job because native WebKit/GTK build prerequisites differ by runner. The frontend is built separately in CI.

## Safety when developing

- Never test destructive operations on a host disk.
- Use disposable virtual disks or loop devices for future imaging/clone tests.
- Keep SMART and discovery probes read-only.
- Do not enable write paths until source/destination identity checks, dry-run preview, typed confirmation, interruption handling and audit logging are tested.
- Do not claim a disk is healthy based on a partial or unsupported SMART response.
