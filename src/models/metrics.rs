use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSnapshot {
    pub generated_at: DateTime<Utc>,
    pub stats: RepoStats,
    pub contributors: Vec<ContributorStats>,
    pub file_churn: Vec<FileChurn>,
    pub timeline: Vec<TimelinePoint>,
    pub hotspots: Vec<Hotspot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStats {
    pub commit_count: u64,
    pub lines_added: u64,
    pub lines_deleted: u64,
    pub repository_age_days: u64,
    pub largest_commits: Vec<CommitSummary>,
    pub language_breakdown: Vec<LanguageStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSummary {
    pub id: String,
    pub author: String,
    pub summary: String,
    pub churn: u64,
    pub committed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorStats {
    pub name: String,
    pub commit_count: u64,
    pub lines_added: u64,
    pub lines_deleted: u64,
    pub files_touched: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChurn {
    pub path: String,
    pub commit_count: u64,
    pub additions: u64,
    pub deletions: u64,
    pub contributor_count: u64,
    pub last_modified: DateTime<Utc>,
}

impl FileChurn {
    pub fn total_churn(&self) -> u64 {
        self.additions + self.deletions
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStat {
    pub language: String,
    pub churn: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    pub date: String,
    pub commits: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotspot {
    pub path: String,
    pub score: f64,
    pub commit_count: u64,
    pub churn: u64,
    pub contributor_count: u64,
}

#[derive(Debug, Clone)]
pub struct CommitRecord {
    pub id: String,
    pub summary: String,
    pub author_name: String,
    pub author_email: String,
    pub committed_at: DateTime<Utc>,
    pub files: Vec<FileDelta>,
}

#[derive(Debug, Clone)]
pub struct FileDelta {
    pub path: String,
    pub additions: u64,
    pub deletions: u64,
}
