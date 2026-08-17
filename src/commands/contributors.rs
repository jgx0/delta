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
        return emit_json(&snapshot.contributors);
    }

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
                c.total_churn().to_string(),
                c.files_touched.to_string(),
            ]
        })
        .collect::<Vec<_>>();

    println!(
        "{}",
        render_table(&["Contributor", "Commits", "+", "-", "Churn", "Files"], &rows)
    );
    Ok(())
}
