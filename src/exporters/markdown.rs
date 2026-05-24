use crate::models::RepoSnapshot;

pub fn export(snapshot: &RepoSnapshot) -> String {
    let mut out = String::new();
    out.push_str("# Delta Report\n\n");
    out.push_str("## Repository Stats\n\n");
    out.push_str(&format!("- Commits: {}\n", snapshot.stats.commit_count));
    out.push_str(&format!("- Lines added: {}\n", snapshot.stats.lines_added));
    out.push_str(&format!("- Lines deleted: {}\n", snapshot.stats.lines_deleted));
    out.push_str("\n## Top Contributors\n\n");

    for c in snapshot.contributors.iter().take(10) {
        out.push_str(&format!(
            "- {}: {} commits (+{} / -{})\n",
            c.name, c.commit_count, c.lines_added, c.lines_deleted
        ));
    }

    out
}
