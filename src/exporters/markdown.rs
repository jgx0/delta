use crate::models::RepoSnapshot;

pub fn export(snapshot: &RepoSnapshot) -> String {
    let mut out = String::new();
    out.push_str("# Delta Report\n\n");

    out.push_str("## Repository Stats\n\n");
    out.push_str(&format!("- Commits: {}\n", snapshot.stats.commit_count));
    out.push_str(&format!("- Lines added: {}\n", snapshot.stats.lines_added));
    out.push_str(&format!("- Lines deleted: {}\n", snapshot.stats.lines_deleted));
    out.push_str(&format!(
        "- Repository age (days): {}\n",
        snapshot.stats.repository_age_days
    ));
    out.push_str(&format!(
        "- Bus factor: {}\n",
        snapshot.stats.bus_factor.bus_factor
    ));

    if !snapshot.stats.language_breakdown.is_empty() {
        out.push_str("\n## Language Breakdown\n\n");
        out.push_str("| Language | Churn |\n| --- | --- |\n");
        for lang in &snapshot.stats.language_breakdown {
            out.push_str(&format!("| {} | {} |\n", lang.language, lang.churn));
        }
    }

    out.push_str("\n## Top Contributors\n\n");
    out.push_str("| Contributor | Commits | + | - | Files |\n| --- | --- | --- | --- | --- |\n");
    for c in snapshot.contributors.iter().take(10) {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            c.name, c.commit_count, c.lines_added, c.lines_deleted, c.files_touched
        ));
    }

    out.push_str("\n## Hotspots\n\n");
    out.push_str("| File | Score | Commits | Churn |\n| --- | --- | --- | --- |\n");
    for h in snapshot.hotspots.iter().take(10) {
        out.push_str(&format!(
            "| {} | {:.3} | {} | {} |\n",
            h.path, h.score, h.commit_count, h.churn
        ));
    }

    out.push_str("\n## File Churn\n\n");
    out.push_str("| File | Commits | + | - | Churn |\n| --- | --- | --- | --- | --- |\n");
    for f in snapshot.file_churn.iter().take(20) {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            f.path,
            f.commit_count,
            f.additions,
            f.deletions,
            f.total_churn()
        ));
    }

    if !snapshot.coupling.is_empty() {
        out.push_str("\n## Logical Coupling\n\n");
        out.push_str("| File A | File B | Co-changes |\n| --- | --- | --- |\n");
        for c in snapshot.coupling.iter().take(20) {
            out.push_str(&format!(
                "| {} | {} | {} |\n",
                c.file_a, c.file_b, c.co_changes
            ));
        }
    }

    if !snapshot.stats.bus_factor.at_risk_files.is_empty() {
        out.push_str("\n## At-Risk Files\n\n");
        out.push_str("| File | Author | Share |\n| --- | --- | --- |\n");
        for f in &snapshot.stats.bus_factor.at_risk_files {
            out.push_str(&format!(
                "| {} | {} | {:.0}% |\n",
                f.path,
                f.author,
                f.share * 100.0
            ));
        }
    }

    out
}
