use std::{fs, path::Path};

use anyhow::Result;
use rusqlite::{params, Connection};

use crate::models::RepoSnapshot;

pub struct SnapshotCache {
    conn: Connection,
}

impl SnapshotCache {
    pub fn new(repo_path: &Path) -> Result<Self> {
        let cache_dir = repo_path.join(".delta");
        fs::create_dir_all(&cache_dir)?;
        let conn = Connection::open(cache_dir.join("cache.db"))?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY,
                repo_path TEXT NOT NULL,
                generated_at TEXT NOT NULL,
                payload TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_snapshots_repo_path_generated_at
            ON snapshots (repo_path, generated_at DESC);
            ",
        )?;
        Ok(Self { conn })
    }

    pub fn store(&self, repo_path: &Path, snapshot: &RepoSnapshot) -> Result<()> {
        let payload = serde_json::to_string(snapshot)?;
        self.conn.execute(
            "INSERT INTO snapshots (repo_path, generated_at, payload) VALUES (?1, ?2, ?3)",
            params![
                repo_path.to_string_lossy().to_string(),
                snapshot.generated_at.to_rfc3339(),
                payload
            ],
        )?;
        Ok(())
    }

    pub fn load_latest(&self, repo_path: &Path) -> Result<Option<RepoSnapshot>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT payload
            FROM snapshots
            WHERE repo_path = ?1
            ORDER BY generated_at DESC
            LIMIT 1
            ",
        )?;

        let payload = stmt.query_row(params![repo_path.to_string_lossy().to_string()], |row| {
            row.get::<_, String>(0)
        });

        match payload {
            Ok(json) => Ok(Some(serde_json::from_str(&json)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}
