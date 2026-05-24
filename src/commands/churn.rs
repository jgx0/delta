use std::path::Path;

use anyhow::Result;

use crate::{commands::load_or_analyze, utils::table::render_table};

pub fn run(path: &Path, limit: usize, refresh_cache: bool) -> Result<()> {
    let snapshot = load_or_analyze(path, limit, refresh_cache)?;
    let rows = snapshot
        .file_churn
        .iter()
        .take(25)
        .map(|f| {
            vec![
                f.path.clone(),
                f.commit_count.to_string(),
                f.additions.to_string(),
                f.deletions.to_string(),
                f.total_churn().to_string(),
            ]
        })
        .collect::<Vec<_>>();

    println!(
        "{}",
        render_table(&["File", "Commits", "+", "-", "Churn"], &rows,)
    );
    Ok(())
}
