use anyhow::Result;

use crate::{
    cli::CommonArgs,
    commands::{emit_json, load_or_analyze},
    models::TimelineGranularity,
    utils::table::render_table,
};

pub fn run(common: &CommonArgs) -> Result<()> {
    let snapshot = load_or_analyze(common, TimelineGranularity::Day)?;
    let info = &snapshot.stats.bus_factor;

    if common.json {
        return emit_json(info);
    }

    println!("Bus factor: {}", info.bus_factor);

    if !info.at_risk_files.is_empty() {
        println!();
        let rows = info
            .at_risk_files
            .iter()
            .map(|f| {
                vec![
                    f.path.clone(),
                    f.author.clone(),
                    format!("{:.0}%", f.share * 100.0),
                    f.churn.to_string(),
                ]
            })
            .collect::<Vec<_>>();

        println!(
            "{}",
            render_table(&["File", "Sole/major author", "Share", "Churn"], &rows)
        );
    }

    Ok(())
}
