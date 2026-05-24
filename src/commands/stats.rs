use std::path::Path;

use anyhow::Result;

use crate::{commands::load_or_analyze, utils::table::render_table};

pub fn run(path: &Path, limit: usize, refresh_cache: bool) -> Result<()> {
    let snapshot = load_or_analyze(path, limit, refresh_cache)?;
    let rows = vec![
        vec!["Commits".into(), snapshot.stats.commit_count.to_string()],
        vec!["Lines added".into(), snapshot.stats.lines_added.to_string()],
        vec!["Lines deleted".into(), snapshot.stats.lines_deleted.to_string()],
        vec![
            "Repository age (days)".into(),
            snapshot.stats.repository_age_days.to_string(),
        ],
        vec![
            "Largest commit (churn)".into(),
            snapshot
                .stats
                .largest_commits
                .first()
                .map(|c| c.churn.to_string())
                .unwrap_or_else(|| "0".to_string()),
        ],
    ];

    println!("{}", render_table(&["Metric", "Value"], &rows));
    Ok(())
}
