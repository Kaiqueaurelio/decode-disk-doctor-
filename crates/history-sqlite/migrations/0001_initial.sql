PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS storage_devices (
    id TEXT PRIMARY KEY,
    stable_identity TEXT,
    device_path TEXT NOT NULL,
    model TEXT NOT NULL,
    serial TEXT,
    capacity_bytes INTEGER NOT NULL,
    first_seen_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS operation_jobs (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    state TEXT NOT NULL,
    source_device_id TEXT,
    destination_device_id TEXT,
    plan_fingerprint TEXT,
    started_at TEXT,
    finished_at TEXT,
    error_code TEXT,
    error_message TEXT,
    FOREIGN KEY(source_device_id) REFERENCES storage_devices(id),
    FOREIGN KEY(destination_device_id) REFERENCES storage_devices(id)
);

CREATE TABLE IF NOT EXISTS operation_audit_events (
    id TEXT PRIMARY KEY,
    operation_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    details_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(operation_id) REFERENCES operation_jobs(id)
);

CREATE TABLE IF NOT EXISTS backup_snapshots (
    id TEXT PRIMARY KEY,
    source_device_id TEXT,
    mode TEXT NOT NULL,
    parent_snapshot_id TEXT,
    manifest_path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    verified INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(source_device_id) REFERENCES storage_devices(id),
    FOREIGN KEY(parent_snapshot_id) REFERENCES backup_snapshots(id)
);

CREATE TABLE IF NOT EXISTS backup_files (
    snapshot_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    sha256 TEXT NOT NULL,
    modified_at TEXT,
    PRIMARY KEY(snapshot_id, relative_path),
    FOREIGN KEY(snapshot_id) REFERENCES backup_snapshots(id)
);

CREATE TABLE IF NOT EXISTS image_manifests (
    id TEXT PRIMARY KEY,
    source_identity TEXT NOT NULL,
    image_path TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    total_bytes INTEGER NOT NULL,
    manifest_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    verified INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS image_chunks (
    manifest_id TEXT NOT NULL,
    chunk_index INTEGER NOT NULL,
    offset_bytes INTEGER NOT NULL,
    length_bytes INTEGER NOT NULL,
    sha256 TEXT NOT NULL,
    PRIMARY KEY(manifest_id, chunk_index),
    FOREIGN KEY(manifest_id) REFERENCES image_manifests(id)
);

CREATE INDEX IF NOT EXISTS idx_operation_jobs_state ON operation_jobs(state);
CREATE INDEX IF NOT EXISTS idx_operation_jobs_started ON operation_jobs(started_at);
CREATE INDEX IF NOT EXISTS idx_audit_operation ON operation_audit_events(operation_id, created_at);
CREATE INDEX IF NOT EXISTS idx_backup_snapshots_source ON backup_snapshots(source_device_id, created_at);
