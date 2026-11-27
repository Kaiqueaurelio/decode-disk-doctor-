# Initial threat model

## Assets

- User data stored on source and destination drives
- Device identity and operation parameters
- Recovery images, mapfiles and manifests
- Technician/customer records and exported reports
- Application binaries and third-party tools

## Threats and controls

| Threat | Required control |
|---|---|
| Wrong disk selected | Show model/capacity/serial/path; re-enumerate before write; typed confirmation |
| Device path spoofing or shell injection | Never build shell command strings; invoke a pinned executable with argument arrays |
| Source media deteriorates during diagnostics | Read-only default; ddrescue-first guidance for failing media |
| Incomplete image mistaken for valid backup | Explicit job state, mapfile/manifest, verification and incomplete marker |
| Malicious or malformed tool output | Parse structured output defensively; treat all output as untrusted |
| Unauthorized privilege escalation | Least privilege; elevate only at the operation boundary |
| Sensitive information leaks through logs | Redact serials and customer details where not required; no telemetry by default |
| Compromised dependency or binary | Pin versions, verify checksums/signatures, maintain third-party license and provenance inventory |

## Open risks

- Cross-platform device identity needs separate Windows and Linux validation.
- SMART data availability varies by controller, USB bridge, RAID layer and device firmware.
- A process boundary does not automatically resolve open-source license obligations.
- Power loss and physical disconnect handling must be validated before imaging and clone workflows are released.

This document is an initial engineering threat model, not a formal security certification.
