use anyhow::Result;

use crate::{
    cli::CommonArgs,
    commands::{emit_json, load_or_analyze},
    models::TimelineGranularity,
    utils::table::render_table,
};

pub fn run(common: &CommonArgs) -> Result<()> {
    let snapshot = load_or_analyze(common, TimelineGranularity::Day)?;

    if common.json {
        return emit_json(&snapshot.commits);
    }

    let rows = snapshot
        .commits
        .iter()
        .take(30)
        .map(|c| {
            vec![
                c.id.clone(),
                c.committed_at.format("%Y-%m-%d").to_string(),
                c.author_name.clone(),
                c.total_churn().to_string(),
                c.files.len().to_string(),
                truncate(&c.summary, 60),
            ]
        })
        .collect::<Vec<_>>();

    println!(
        "{}",
        render_table(&["Commit", "Date", "Author", "Churn", "Files", "Summary"], &rows)
    );
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    let chars = s.chars().collect::<Vec<_>>();
    if chars.len() <= max {
        s.to_string()
    } else {
        chars[..max.saturating_sub(1)].iter().collect::<String>() + "…"
    }
}
