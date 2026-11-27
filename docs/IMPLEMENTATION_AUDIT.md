# Implementation audit — physical operations milestone

Audit date: 2026-11-27  
Target branch: `feat/continuity-foundation-v0.4.3`  
Baseline: `main` at `00fd096428354cfcc35a4857f28c0820f2df4e60`

## Repository and architecture

The repository's existing Tauri 2 + React + TypeScript desktop application and Rust workspace were preserved. The work is committed to the feature branch and the pull request remains open; `main` has not been changed.

## Implemented execution paths

- **Physical clone (Linux):** opens source read-only and destination read/write only after plan-bound confirmation; requires Linux block-device nodes; revalidates sysfs hardware serials; rejects same physical devices and mounted/system disks; queries actual capacity with BLKGETSIZE64; streams in bounded memory; syncs and hashes the write; reads the destination back and compares SHA-256.
- **Raw image acquisition (Linux):** reads a physical block device, requires the source to be unmounted, checks the selected destination block device against the actual filesystem containing the output path, verifies distinct physical disks, checks free space, refuses to overwrite an existing image, hashes the image and reads it back for verification.
- **Image inspection:** checks that the image is a regular file on the selected storage device and returns its byte length and SHA-256.
- **Raw image restoration (Linux):** requires an image-file source and block-device destination, plan-bound confirmation, matching source/destination serials, distinct physical disks, unmounted target, exact image length and actual target capacity; performs streaming write, sync and read-back hash verification.
- **GNU ddrescue:** executes a conservative first pass using `--no-scrape`; checks source and destination hardware identity and ensures the image/mapfile are on the selected separate destination disk; preserves partial image/mapfile pairs for resume and reports non-zero exit status without discarding partial recovery data.
- **PhotoRec:** executes the actual PhotoRec process; supports a block device or regular image as source; validates the selected destination disk against the output directory's physical filesystem; creates the output directory only after checking the destination; never writes recovered files to the source.
- **Desktop UI:** connected to clone, image acquisition, image inspection, restore, ddrescue and PhotoRec commands. Destructive workflows require a typed, plan-specific confirmation phrase.

## Validation

- Rust workspace formatting and core compile/test/Clippy checks are run by GitHub Actions.
- The Tauri desktop backend is included in CI compilation with Linux Tauri system libraries.
- The React frontend is built in CI.
- The most recent CI run for the current implementation is still being finalized; consult the pull request checks for the final result.
- Tests use in-memory readers and temporary fixtures. They do **not** prove operation on physical hardware.

## Explicit limitations — not production-ready

1. Physical block-device execution is Linux-only. Windows physical clone, image and restore adapters are not implemented.
2. No disposable physical-disk test has been completed. Hardware-level validation remains required before production use.
3. No privilege broker/polkit helper exists. Raw-device operations require the application process to already have suitable OS permissions.
4. No live progress reporting or cancellation for long-running clone, image, ddrescue or PhotoRec operations.
5. No persistent operation journal/audit database is wired to these commands.
6. TestDisk partition-write workflows, Recovery ISO, directory/incremental backup and automatic mount management remain unimplemented.
7. GNU ddrescue, PhotoRec and smartmontools are external dependencies and must be installed on the target system.

## Safety notes

- Do not test on a system disk or any disk containing data that matters.
- Clone and restore destinations are overwritten.
- Source disks are never selected as recovery output destinations.
- Missing or mismatched hardware serials fail closed.
- CI success is not a substitute for disposable-device and real-hardware validation.
