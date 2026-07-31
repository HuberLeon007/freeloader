-- Initial schema for Freeloader downloads.
-- Applied by sqlx::migrate!() on first launch and after upgrades.

CREATE TABLE IF NOT EXISTS downloads (
    id          TEXT PRIMARY KEY NOT NULL,
    url         TEXT NOT NULL,
    final_url   TEXT,
    destination TEXT NOT NULL,
    temporary   TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'created',
    downloaded  INTEGER NOT NULL DEFAULT 0,
    total       INTEGER,
    accept_ranges TEXT,
    etag        TEXT,
    last_modified TEXT,
    error_code  TEXT,
    restart_notice TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
