use std::path::Path;

use anyhow::Result;

use crate::{commands::load_or_analyze, utils::table::render_table};

pub fn run(path: &Path, limit: usize, refresh_cache: bool) -> Result<()> {
    let snapshot = load_or_analyze(path, limit, refresh_cache)?;
    let rows = snapshot
        .contributors
        .iter()
        .take(20)
        .map(|c| {
            vec![
                c.name.clone(),
                c.commit_count.to_string(),
                c.lines_added.to_string(),
                c.lines_deleted.to_string(),
                c.files_touched.to_string(),
            ]
        })
        .collect::<Vec<_>>();

    println!(
        "{}",
        render_table(
            &["Contributor", "Commits", "+", "-", "Files"],
            &rows,
        )
    );
    Ok(())
}
