# Third-party software inventory

This is a living compliance register. A project listed here is a candidate/reference only until a specific version, artifact, source file and license have been inspected and approved.

| Component | Intended role | License status | Integration policy |
|---|---|---|---|
| smartmontools | SMART/SATA/NVMe health data | Verify exact release and license before bundling | Prefer packaged upstream binary or documented adapter |
| GNU ddrescue | Acquisition of failing media | Verify exact release and GPL obligations | Preserve mapfiles and logs; do not write to source |
| Partclone | Filesystem-aware imaging | Verify exact release and GPL obligations | Use only for supported filesystems and verified targets |
| TestDisk / PhotoRec | Partition and file recovery | Verify exact release and GPL obligations | Recovery output must go to a separate destination |
| Rescuezilla | Recovery workflow/reference | Verify exact release and all bundled components | Study build/integration approach; do not copy blindly |
| Clonezilla | Imaging/cloning workflow/reference | Verify exact release and all bundled components | Use as a separately reviewed recovery distribution component |
| GSmartControl | SMART UI/reference | Verify exact release and license | Reference only unless separately approved |
| DiskMaster | Diagnostic/reference code | Verify repository license, dependencies and provenance | Audit before reuse; no unreviewed copying |

## Required approval record

Before a third-party component is shipped, record its exact upstream URL and pinned version/commit, exact license identifier and full license text, modifications, integration method, required notices, source-code obligations, build scripts and patches, compatibility with commercial distribution, and legal review status.

Store license texts and notices in the component's folder under licenses/third-party/.

**Important:** process boundaries, separate folders or a separate USB image do not automatically eliminate copyleft obligations. Distribution architecture must be reviewed against the exact license and facts. Do not ship an artifact while its compliance status is unknown.
