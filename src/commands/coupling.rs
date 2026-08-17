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
        return emit_json(&snapshot.coupling);
    }

    let rows = snapshot
        .coupling
        .iter()
        .take(40)
        .map(|c| vec![c.file_a.clone(), c.file_b.clone(), c.co_changes.to_string()])
        .collect::<Vec<_>>();

    println!(
        "{}",
        render_table(&["File A", "File B", "Co-changes"], &rows)
    );
    Ok(())
}
