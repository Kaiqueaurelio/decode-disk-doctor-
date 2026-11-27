# Disk operation safety requirements

## Default posture

All discovery and health diagnostics are read-only. No automatic repair, formatting, partition-table update, filesystem check with write mode, secure erase, firmware update or restore operation is allowed without a separate reviewed workflow and explicit user authorization.

## Failing media

When SMART data, kernel logs or read errors indicate possible failure:
- Warn that continued use may worsen data loss.
- Recommend stopping unnecessary tests.
- Prefer sector-level acquisition with GNU ddrescue and a persistent mapfile.
- Store the image and mapfile on a different physical device with sufficient free capacity.
- Do not run surface write tests on the source.
- Explain that recovery cannot be guaranteed.

## Destructive actions

Before clone/restore/write:
1. Show source and destination separately with model, serial (if available), capacity and device path.
2. State exactly what will be overwritten.
3. Require the user to type a confirmation phrase containing the destination identity.
4. Re-enumerate devices and verify identities immediately before execution.
5. Abort if source and destination resolve to the same device, identity is ambiguous, or capacity is insufficient.
6. Persist an audit record and operation log.

## Cancellation and interruption

Cancellation must not silently leave a destination presented as valid. Mark incomplete images/jobs as incomplete, preserve logs and mapfiles, and provide a safe resume or discard path. Never delete source data during cleanup.

## Testing

Use fixtures, loop devices, disposable virtual disks and fault-injection tests. Never use a developer's system disk as an automated test target. Test permission failures, unplug events, full destination, read errors, cancellation and power-loss recovery.

## Reporting

Separate observed facts from interpretation. Include timestamp, device identifiers, tool versions, test duration, errors and unsupported fields. Do not present a single SMART attribute or opaque score as a guaranteed prediction of failure.
