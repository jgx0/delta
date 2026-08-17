use anyhow::Result;

use crate::{
    cli::CommonArgs,
    commands::{emit_json, load_or_analyze},
    models::TimelineGranularity,
    utils::table::render_table,
};

fn sparkline(value: u64, max: u64) -> String {
    const BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if max == 0 {
        return "▁".to_string();
    }
    let idx = ((value as f64 / max as f64) * (BLOCKS.len() - 1) as f64).round() as usize;
    BLOCKS[idx.min(BLOCKS.len() - 1)].to_string()
}

pub fn run(common: &CommonArgs, granularity: TimelineGranularity) -> Result<()> {
    let snapshot = load_or_analyze(common, granularity)?;

    if common.json {
        return emit_json(&snapshot.timeline);
    }

    let max = snapshot
        .timeline
        .iter()
        .map(|d| d.commits)
        .max()
        .unwrap_or(0);
    let rows = snapshot
        .timeline
        .iter()
        .rev()
        .take(30)
        .rev()
        .map(|d| {
            vec![
                d.date.clone(),
                d.commits.to_string(),
                sparkline(d.commits, max),
            ]
        })
        .collect::<Vec<_>>();

    println!("{}", render_table(&["Date", "Commits", "Activity"], &rows));
    Ok(())
}
