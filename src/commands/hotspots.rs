use std::path::Path;

use anyhow::Result;

use crate::{commands::load_or_analyze, utils::table::render_table};

pub fn run(path: &Path, limit: usize, refresh_cache: bool) -> Result<()> {
    let snapshot = load_or_analyze(path, limit, refresh_cache)?;
    let rows = snapshot
        .hotspots
        .iter()
        .take(25)
        .map(|h| {
            vec![
                h.path.clone(),
                format!("{:.3}", h.score),
                h.commit_count.to_string(),
                h.churn.to_string(),
                h.contributor_count.to_string(),
            ]
        })
        .collect::<Vec<_>>();

    println!(
        "{}",
        render_table(
            &["File", "Score", "Commits", "Churn", "Contrib"],
            &rows,
        )
    );
    Ok(())
}
