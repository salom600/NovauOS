//! Local SQLite cache of package metadata.

use anyhow::Result;
use rusqlite::Connection;
use std::sync::Mutex;

pub struct Cache {
    conn: Mutex<Connection>,
}

impl Cache {
    pub fn open() -> Result<Self> {
        let path = novau_common::paths::cache().join("store.db");
        novau_common::ensure_dir(&path.parent().unwrap())?;
        let conn = Connection::open(&path)?;
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS packages (
                id        TEXT NOT NULL,
                kind      TEXT NOT NULL,
                name      TEXT NOT NULL,
                summary   TEXT,
                icon      TEXT,
                rating    REAL DEFAULT 0,
                installed INTEGER DEFAULT 0,
                updated   INTEGER NOT NULL,
                PRIMARY KEY (id, kind)
            );
            CREATE INDEX IF NOT EXISTS idx_packages_name ON packages(name);
        "#)?;
        Ok(Self { conn: Mutex::new(conn) })
    }
}
