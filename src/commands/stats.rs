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
        return emit_json(&snapshot.stats);
    }

    let stats = &snapshot.stats;

    let rows = vec![
        vec!["Commits".into(), stats.commit_count.to_string()],
        vec!["Lines added".into(), stats.lines_added.to_string()],
        vec!["Lines deleted".into(), stats.lines_deleted.to_string()],
        vec![
            "Repository age (days)".into(),
            stats.repository_age_days.to_string(),
        ],
        vec!["Bus factor".into(), stats.bus_factor.bus_factor.to_string()],
        vec![
            "Largest commit (churn)".into(),
            stats
                .largest_commits
                .first()
                .map(|c| c.churn.to_string())
                .unwrap_or_else(|| "0".to_string()),
        ],
    ];
    println!("{}", render_table(&["Metric", "Value"], &rows));

    if !stats.language_breakdown.is_empty() {
        println!();
        let lang_rows = stats
            .language_breakdown
            .iter()
            .map(|l| vec![l.language.clone(), l.churn.to_string()])
            .collect::<Vec<_>>();
        println!("{}", render_table(&["Language", "Churn"], &lang_rows));
    }

    if !stats.largest_commits.is_empty() {
        println!();
        let commit_rows = stats
            .largest_commits
            .iter()
            .map(|c| {
                vec![
                    c.id.clone(),
                    c.author.clone(),
                    c.churn.to_string(),
                    c.summary.clone(),
                ]
            })
            .collect::<Vec<_>>();
        println!(
            "{}",
            render_table(&["Commit", "Author", "Churn", "Summary"], &commit_rows)
        );
    }

    Ok(())
}
