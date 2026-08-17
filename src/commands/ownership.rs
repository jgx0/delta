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
        return emit_json(&snapshot.ownership);
    }

    let rows = snapshot
        .ownership
        .iter()
        .take(30)
        .map(|o| {
            vec![
                o.path.clone(),
                o.dominant_author.clone(),
                format!("{:.0}%", o.dominant_share * 100.0),
                o.total_churn.to_string(),
                o.contributor_count.to_string(),
            ]
        })
        .collect::<Vec<_>>();

    println!(
        "{}",
        render_table(
            &["File", "Dominant author", "Share", "Churn", "Contrib"],
            &rows
        )
    );
    Ok(())
}
