use std::collections::BTreeMap;

use crate::models::{CommitRecord, TimelineGranularity, TimelinePoint};

pub fn build_timeline(commits: &[CommitRecord], granularity: TimelineGranularity) -> Vec<TimelinePoint> {
    let mut by_bucket = BTreeMap::<String, u64>::new();
    for commit in commits {
        let key = bucket_key(commit.committed_at, granularity);
        *by_bucket.entry(key).or_default() += 1;
    }

    by_bucket
        .into_iter()
        .map(|(date, commits)| TimelinePoint { date, commits })
        .collect()
}

fn bucket_key(dt: chrono::DateTime<chrono::Utc>, granularity: TimelineGranularity) -> String {
    use chrono::Datelike;
    match granularity {
        TimelineGranularity::Day => dt.format("%Y-%m-%d").to_string(),
        TimelineGranularity::Week => {
            let week = dt.iso_week();
            format!("{}-W{:02}", week.year(), week.week())
        }
        TimelineGranularity::Month => dt.format("%Y-%m").to_string(),
        TimelineGranularity::Year => dt.year().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::build_timeline;
    use crate::models::{CommitRecord, FileDelta, TimelineGranularity};

    fn commit_at(id: &str, at: chrono::DateTime<Utc>) -> CommitRecord {
        CommitRecord {
            id: id.into(),
            summary: "s".into(),
            author_name: "x".into(),
            author_email: "x@x".into(),
            committed_at: at,
            files: vec![FileDelta {
                path: "a.rs".into(),
                additions: 1,
                deletions: 1,
            }],
        }
    }

    #[test]
    fn timeline_buckets_commits_by_day() {
        let now = Utc::now();
        let commits = vec![commit_at("1", now), commit_at("2", now - Duration::hours(1))];

        let timeline = build_timeline(&commits, TimelineGranularity::Day);
        assert_eq!(timeline.iter().map(|t| t.commits).sum::<u64>(), 2);
    }

    #[test]
    fn timeline_groups_across_days_into_one_month_bucket() {
        let base = chrono::NaiveDate::from_ymd_opt(2024, 3, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let commits = vec![
            commit_at("1", base),
            commit_at("2", base + Duration::days(20)),
        ];

        let timeline = build_timeline(&commits, TimelineGranularity::Month);
        assert_eq!(timeline.len(), 1);
        assert_eq!(timeline[0].commits, 2);
        assert_eq!(timeline[0].date, "2024-03");
    }
}
