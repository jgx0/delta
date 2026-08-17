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
        return emit_json(&snapshot.stats.language_breakdown);
    }

    let max = snapshot
        .stats
        .language_breakdown
        .first()
        .map(|l| l.churn)
        .unwrap_or(0);

    let rows = snapshot
        .stats
        .language_breakdown
        .iter()
        .map(|l| vec![l.language.clone(), l.churn.to_string(), bar(l.churn, max)])
        .collect::<Vec<_>>();

    println!("{}", render_table(&["Language", "Churn", "Share"], &rows));
    Ok(())
}

fn bar(value: u64, max: u64) -> String {
    const WIDTH: usize = 20;
    if max == 0 {
        return String::new();
    }
    let filled = ((value as f64 / max as f64) * WIDTH as f64).round() as usize;
    format!("{}{}", "█".repeat(filled), "░".repeat(WIDTH - filled))
}
