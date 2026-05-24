use chrono::Utc;

use crate::models::{FileChurn, Hotspot};

pub fn build_hotspots(files: &[FileChurn]) -> Vec<Hotspot> {
    if files.is_empty() {
        return Vec::new();
    }

    let max_commits = files.iter().map(|f| f.commit_count).max().unwrap_or(1) as f64;
    let max_churn = files.iter().map(|f| f.total_churn()).max().unwrap_or(1) as f64;
    let max_contrib = files.iter().map(|f| f.contributor_count).max().unwrap_or(1) as f64;

    let now = Utc::now();
    let mut hotspots = files
        .iter()
        .map(|f| {
            let freq = f.commit_count as f64 / max_commits;
            let churn = f.total_churn() as f64 / max_churn;
            let contributors = f.contributor_count as f64 / max_contrib;
            let days_since = (now - f.last_modified).num_days().max(0) as f64;
            let recency = 1.0 / (1.0 + days_since / 30.0);
            let density = if days_since <= 1.0 {
                f.commit_count as f64
            } else {
                f.commit_count as f64 / days_since
            };
            let density_norm = (density / max_commits).min(1.0);

            let score = (freq * 0.30)
                + (churn * 0.25)
                + (contributors * 0.15)
                + (recency * 0.20)
                + (density_norm * 0.10);

            Hotspot {
                path: f.path.clone(),
                score,
                commit_count: f.commit_count,
                churn: f.total_churn(),
                contributor_count: f.contributor_count,
            }
        })
        .collect::<Vec<_>>();

    hotspots.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    hotspots
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::build_hotspots;
    use crate::models::FileChurn;

    #[test]
    fn hotspots_rank_more_active_files_higher() {
        let now = Utc::now();
        let files = vec![
            FileChurn {
                path: "a.rs".into(),
                commit_count: 100,
                additions: 200,
                deletions: 50,
                contributor_count: 5,
                last_modified: now,
            },
            FileChurn {
                path: "b.rs".into(),
                commit_count: 1,
                additions: 2,
                deletions: 1,
                contributor_count: 1,
                last_modified: now - Duration::days(300),
            },
        ];

        let hotspots = build_hotspots(&files);
        assert_eq!(hotspots[0].path, "a.rs");
    }
}
