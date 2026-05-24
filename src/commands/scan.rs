use std::path::Path;

use anyhow::Result;

use crate::{commands::load_or_analyze, tui::dashboard};

pub fn run(path: &Path, limit: usize, refresh_cache: bool, tui: bool) -> Result<()> {
    let snapshot = load_or_analyze(path, limit, refresh_cache)?;
    if tui {
        dashboard::run(&snapshot)?;
    } else {
        println!(
            "Scanned {} commits | +{} -{} | contributors: {} | files: {}",
            snapshot.stats.commit_count,
            snapshot.stats.lines_added,
            snapshot.stats.lines_deleted,
            snapshot.contributors.len(),
            snapshot.file_churn.len()
        );
    }
    Ok(())
}
