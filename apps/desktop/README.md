# Desktop application

The desktop app uses Tauri 2, React, TypeScript and Vite.

## Prerequisites

- Node.js LTS and npm
- Rust stable toolchain
- Tauri 2 system dependencies for the target OS
- On Linux, install the WebKitGTK and GTK development packages required by the official Tauri prerequisites

## Run in development

From this directory:

1. Install dependencies with npm install.
2. Start the app with npm run tauri dev.

Build the frontend with npm run build. Build native bundles with npm run tauri build.

## Current functionality

The first screen provides a user-triggered, read-only Linux block-device discovery command through sysfs. It does not read raw disk sectors, query SMART, write to disks, or perform repair. Windows discovery is not implemented yet and returns an explicit unsupported-platform error.

Consult the root ROADMAP.md before treating planned modules as available.
