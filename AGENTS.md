# Agent instructions — Decode Disk Doctor

This repository is an existing Rust/Tauri/React product. Preserve its crate names, APIs and architectural decisions unless a reviewed migration requires otherwise.

## Mandatory workflow
1. Inspect the current implementation and documentation before editing.
2. Make small, reviewable changes.
3. Run `cargo fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and the frontend build when toolchains are available.
4. Report commands that could not run; never claim unexecuted tests passed.
5. Update README, ROADMAP, CHANGELOG and relevant docs when behavior changes.

## Storage safety
- Default to read-only.
- Never format, partition, erase, sanitize, repair or write to a block device without an explicitly approved and reviewed workflow.
- Never use shell-concatenated user input.
- Bind confirmation to the exact immutable plan and revalidate physical device identity immediately before writes.
- Never recover files to the source device.
- Prefer ddrescue imaging before extended tests on failing media.
- Automated tests must use fixtures, memory readers, temporary files or disposable virtual devices.

## Product truthfulness
- Do not represent planning contracts as operational engines.
- Do not fabricate SMART, NVMe, health or recovery results.
- Do not mark hardware support as validated without real-device evidence.
- Do not claim a Recovery ISO exists until it has been generated and boot-tested.

## Open source compliance
Review exact versions and licenses before bundling or copying upstream code. Keep THIRD_PARTY.md and licenses/third-party current. Process isolation alone does not remove copyleft obligations.
