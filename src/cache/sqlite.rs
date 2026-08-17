use std::{fs, path::Path};

use anyhow::Result;
use rusqlite::{Connection, params};

use crate::models::{AnalysisOptions, RepoSnapshot};

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
            CREATE TABLE IF NOT EXISTS snapshots_v2 (
                id INTEGER PRIMARY KEY,
                repo_path TEXT NOT NULL,
                cache_key TEXT NOT NULL,
                generated_at TEXT NOT NULL,
                payload TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_snapshots_v2_lookup
            ON snapshots_v2 (repo_path, cache_key, generated_at DESC);
            ",
        )?;
        Ok(Self { conn })
    }

    pub fn store(&self, repo_path: &Path, cache_key: &str, snapshot: &RepoSnapshot) -> Result<()> {
        let payload = serde_json::to_string(snapshot)?;
        self.conn.execute(
            "INSERT INTO snapshots_v2 (repo_path, cache_key, generated_at, payload) VALUES (?1, ?2, ?3, ?4)",
            params![
                repo_path.to_string_lossy().to_string(),
                cache_key,
                snapshot.generated_at.to_rfc3339(),
                payload
            ],
        )?;
        Ok(())
    }

    pub fn load_latest(&self, repo_path: &Path, cache_key: &str) -> Result<Option<RepoSnapshot>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT payload
            FROM snapshots_v2
            WHERE repo_path = ?1 AND cache_key = ?2
            ORDER BY generated_at DESC
            LIMIT 1
            ",
        )?;

        let payload = stmt.query_row(
            params![repo_path.to_string_lossy().to_string(), cache_key],
            |row| row.get::<_, String>(0),
        );

        match payload {
            Ok(json) => Ok(Some(serde_json::from_str(&json)?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

/// Build a deterministic cache key that captures everything that influences
/// the analysis result, so two requests with different filters never share a
/// stale snapshot.
pub fn cache_key(repo_path: &Path, options: &AnalysisOptions) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}",
        repo_path.to_string_lossy(),
        options.limit,
        options.since.map(|d| d.timestamp()).unwrap_or(0),
        options.until.map(|d| d.timestamp()).unwrap_or(0),
        options.include.join("\u{1}"),
        options.exclude.join("\u{1}"),
        options.granularity.as_str(),
    )
}
