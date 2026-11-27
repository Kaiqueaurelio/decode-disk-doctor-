# Physical operations and safety

## Linux prerequisites

Install the external utilities used by the desktop app:

```bash
sudo apt update
sudo apt install smartmontools gddrescue testdisk
```

- `smartctl` is supplied by smartmontools.
- `ddrescue` is supplied by gddrescue.
- `photorec` is supplied by testdisk.

The application currently executes raw block-device operations on Linux only. The current implementation does not include a privilege broker. The process must already have the required read/write permissions for the selected devices; do not solve permission errors by changing device nodes to world-writable.

## Before testing

Use two disposable drives or loop-backed test media. Back up anything important. Never choose the running system disk, a mounted disk, or a disk containing data you need.

The backend rechecks the physical serial against the discovered stable identity, rejects source/destination aliasing, checks actual block-device capacity and checks Linux mount information immediately before destructive writes. Missing or inaccessible hardware serials fail closed.

## Supported workflows

### Physical clone

- Select separate source and destination disks.
- Both disks must be unmounted.
- The destination must be at least as large as the requested source length.
- The destination is overwritten.
- The operation is streamed, SHA-256 hashed and read back for verification.

### Raw image acquisition

- The source disk must be unmounted.
- Select the physical disk that contains the output folder as the destination.
- The image file must not already exist.
- The destination filesystem must have enough free space for the entire source disk.
- The image is hashed and read back after writing.

### Image inspection and restoration

- Use the image inspection control to read the actual image size and SHA-256.
- The image file must be on a disk different from the restore target.
- The target disk and all its partitions must be unmounted.
- Restoration overwrites the target and requires the exact plan-bound confirmation phrase.
- The target capacity is queried from the block device; the image must fit.
- The written data is read back and its hash compared.

### GNU ddrescue

- Use a separate destination disk for both the image and mapfile.
- The first pass uses `--no-scrape`.
- Keep the image and mapfile together. If both exist, the operation can resume from the mapfile.
- If ddrescue exits with a non-zero status, the partial image and mapfile are intentionally retained. Review the exit status and mapfile before deciding what to do next.

### PhotoRec

- Choose a source disk and a different destination disk.
- The output directory is created on the selected destination filesystem after physical-disk verification.
- Recovered files are never written to the source.
- The current desktop workflow runs PhotoRec synchronously and does not yet expose live progress or cancellation.

## Known limitations

- Windows and macOS physical operations are not implemented.
- No physical hardware test has been completed by CI; CI uses stream fixtures and compile checks.
- No automatic privilege elevation/polkit helper is included.
- No persistent operation journal or audit database is wired to these commands yet.
- Recovery ISO, TestDisk partition-write operations, directory backup and incremental backup are not implemented.

