# Recovery Edition

Target: customized Ubuntu Live environment branded Decode Analytics.

Planned capabilities:
- UEFI boot; legacy BIOS only where the selected base image and hardware support it.
- Offline device discovery and SMART/NVMe inspection.
- ddrescue-first acquisition for failing media.
- Imaging, backup and restore workflows after safety review.
- TestDisk analysis and PhotoRec recovery to a separate destination.
- Terminal and technical logs.
- Optional persistence only after security and reliability review.

Build requirements:
- Reproducible build scripts and pinned package manifest.
- Third-party license inventory and notices.
- ISO checksum and package manifest.
- VM boot tests for UEFI and supported legacy BIOS.
- Virtual disk discovery and non-destructive workflow tests.
- Clear warning that no recovery is guaranteed.

Status: not built; no ISO has been generated or boot-tested.
