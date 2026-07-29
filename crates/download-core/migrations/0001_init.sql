CREATE TABLE IF NOT EXISTS downloads (
    id TEXT PRIMARY KEY NOT NULL,
    url TEXT NOT NULL,
    destination_path TEXT NOT NULL,
    part_path TEXT NOT NULL,
    status TEXT NOT NULL,
    bytes_downloaded INTEGER NOT NULL DEFAULT 0,
    total_bytes INTEGER NULL,
    retries INTEGER NOT NULL DEFAULT 0,
    etag TEXT NULL,
    last_error TEXT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS downloads_status_idx ON downloads(status);
