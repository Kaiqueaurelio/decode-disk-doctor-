# Operation safety contract

## Read-only default

Discovery, SMART, NVMe inspection and diagnostic reads must not modify source media. No format, partition, repair, sanitize, secure erase, firmware update, zero-fill or raw-device write is implicit.

## Before clone or restore

1. Resolve source and destination to stable physical identities.
2. Show model, serial/WWN/EUI where available, capacity, bus, current path and mount/system status.
3. Reject ambiguous or missing identity.
4. Reject same-device aliases, including symlink/by-id aliases.
5. Reject system, mounted or read-only destinations.
6. Verify destination capacity and sector-size compatibility.
7. Create a fingerprinted immutable operation plan.
8. Require a confirmation phrase derived from that exact plan.
9. Re-enumerate and revalidate immediately before opening the destination for writing.
10. Persist an audit event before execution and completion/failure events after it.

## Cancellation and resume

A canceled operation must remain marked incomplete. Never report a partial image as verified. Preserve logs and ddrescue mapfiles. Resume only when source identity, destination identity, operation parameters and artifact metadata match the original plan.

## Recovery

Never recover files onto the source device. For failing media, image first with ddrescue and recover from the copy. Mapfile storage must be on healthy, separate storage. Verify destination mount ancestry at runtime; a different pathname does not prove a different physical device.

## Testing

Unit tests use in-memory readers, fixtures and temporary regular files. Integration tests must use disposable loop devices or virtual disks. No CI job may write to host block devices. Physical hardware qualification is a separate, explicitly documented release activity.


## Physical identity requirements

A sysfs path, /dev/sdX name, mount point or Windows PNP/topology identifier is not sufficient proof of physical-device identity for a destructive operation. Discovery only marks a hardware serial as a write-grade stable identifier. If a serial is unavailable, the device may still be inspected, but clone/restore authorization must fail closed until a platform adapter can resolve a stronger identifier such as WWN/EUI/NGUID. The same physical serial must be treated as the same device even when two OS paths or topology IDs differ.
