use anyhow::Result;

use crate::{
    cli::CommonArgs,
    commands::{emit_json, load_or_analyze},
    models::TimelineGranularity,
    tui::dashboard,
};

pub fn run(common: &CommonArgs, tui: bool) -> Result<()> {
    let snapshot = load_or_analyze(common, TimelineGranularity::Day)?;

    if tui {
        return dashboard::run(&snapshot);
    }

    if common.json {
        return emit_json(&snapshot);
    }

    println!(
        "Scanned {} commits | +{} -{} | contributors: {} | files: {}",
        snapshot.stats.commit_count,
        snapshot.stats.lines_added,
        snapshot.stats.lines_deleted,
        snapshot.contributors.len(),
        snapshot.file_churn.len()
    );
    Ok(())
}
