# Third-party software inventory

This is a compliance register. A project listed here is a candidate/reference only until a specific version, artifact, source file and license have been inspected and approved.

| Component | Intended role | Upstream | Integration policy |
|---|---|---|---|
| smartmontools | SMART/SATA/NVMe health data | https://github.com/smartmontools/smartmontools | Direct subprocess adapter; verify exact release, GPL obligations and bundled binaries |
| GNU ddrescue | Acquisition of failing media | https://www.gnu.org/software/ddrescue/ | Mapfile-based acquisition; preserve source; verify distribution obligations |
| Partclone | Filesystem-aware imaging | https://github.com/Thomas-Tsai/partclone | Supported filesystems only; pin version and review GPL obligations |
| Clonezilla | Imaging/cloning workflow | https://github.com/clonezilla/clonezilla | Treat as a distribution/workflow, not a simple Rust library |
| Rescuezilla | Recovery workflow/reference | https://github.com/rescuezilla/rescuezilla | Study integration/build approach; audit all bundled components |
| TestDisk / PhotoRec | Partition and file recovery | https://github.com/cgsecurity/testdisk | Recover to separate destination; verify exact license and version |
| GSmartControl | SMART UI/reference | https://github.com/ashaduri/gsmartcontrol | Reference/fixtures; no unreviewed GPL code copying |
| DiskMaster | Diagnostics/reference | https://github.com/hclivess/diskmaster | Audit source, dependencies and license before reuse |
| Partition Manager | Rust storage abstractions/reference | https://github.com/inalto/partition-manager | Reuse only reviewed abstractions; isolate destructive operations |

## Required approval record

Before a third-party component is shipped, record exact upstream URL and pinned version/commit, exact license identifier and full license text, modifications, integration method, required notices, source-code obligations, build scripts and patches, commercial-distribution compatibility and legal review status.

Store license texts and notices under `licenses/third-party/`.

A process boundary, separate folder or separate USB image does not automatically eliminate copyleft obligations. Do not distribute a component while its compliance status is unknown.
