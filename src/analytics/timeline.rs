use std::collections::BTreeMap;

use crate::models::{CommitRecord, TimelinePoint};

pub fn build_timeline(commits: &[CommitRecord]) -> Vec<TimelinePoint> {
    let mut by_day = BTreeMap::<String, u64>::new();
    for commit in commits {
        let day = commit.committed_at.format("%Y-%m-%d").to_string();
        *by_day.entry(day).or_default() += 1;
    }

    by_day
        .into_iter()
        .map(|(date, commits)| TimelinePoint { date, commits })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::build_timeline;
    use crate::models::{CommitRecord, FileDelta};

    #[test]
    fn timeline_buckets_commits_by_day() {
        let now = Utc::now();
        let commits = vec![
            CommitRecord {
                id: "1".into(),
                summary: "a".into(),
                author_name: "x".into(),
                author_email: "x@x".into(),
                committed_at: now,
                files: vec![FileDelta {
                    path: "a.rs".into(),
                    additions: 1,
                    deletions: 1,
                }],
            },
            CommitRecord {
                id: "2".into(),
                summary: "b".into(),
                author_name: "x".into(),
                author_email: "x@x".into(),
                committed_at: now - Duration::hours(1),
                files: vec![FileDelta {
                    path: "b.rs".into(),
                    additions: 1,
                    deletions: 1,
                }],
            },
        ];

        let timeline = build_timeline(&commits);
        assert_eq!(timeline.iter().map(|t| t.commits).sum::<u64>(), 2);
    }
}
