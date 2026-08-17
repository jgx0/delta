use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A complete, serializable snapshot of repository analytics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSnapshot {
    pub generated_at: DateTime<Utc>,
    pub stats: RepoStats,
    pub contributors: Vec<ContributorStats>,
    pub file_churn: Vec<FileChurn>,
    pub timeline: Vec<TimelinePoint>,
    pub hotspots: Vec<Hotspot>,
    pub ownership: Vec<Ownership>,
    pub coupling: Vec<Coupling>,
    /// The commit records that were analyzed, newest first.
    pub commits: Vec<CommitRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStats {
    pub commit_count: u64,
    pub lines_added: u64,
    pub lines_deleted: u64,
    pub repository_age_days: u64,
    pub largest_commits: Vec<CommitSummary>,
    pub language_breakdown: Vec<LanguageStat>,
    pub bus_factor: BusFactorInfo,
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

impl ContributorStats {
    /// Total lines added plus deleted by this contributor.
    pub fn total_churn(&self) -> u64 {
        self.lines_added + self.lines_deleted
    }
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

/// Per-file knowledge concentration: who owns a file and how much.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ownership {
    pub path: String,
    pub dominant_author: String,
    /// Fraction (0.0..=1.0) of the file's churn attributed to the dominant author.
    pub dominant_share: f64,
    pub total_churn: u64,
    pub contributor_count: u64,
}

/// A file whose knowledge is concentrated in one person.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtRiskFile {
    pub path: String,
    pub author: String,
    pub share: f64,
    pub churn: u64,
}

/// Bus factor: how many contributors are needed to cover half of all churn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusFactorInfo {
    /// Number of contributors accounting for >= 50% of total churn.
    pub bus_factor: u64,
    pub at_risk_files: Vec<AtRiskFile>,
}

/// Two files that frequently change together in the same commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coupling {
    pub file_a: String,
    pub file_b: String,
    pub co_changes: u64,
}

/// How commit activity is bucketed for the timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum TimelineGranularity {
    Day,
    Week,
    Month,
    Year,
}

impl TimelineGranularity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Year => "year",
        }
    }
}

impl std::fmt::Display for TimelineGranularity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Input parameters that shape an analysis run.
#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    pub limit: usize,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    /// Glob patterns files must match (empty means all files).
    pub include: Vec<String>,
    /// Glob patterns files must not match.
    pub exclude: Vec<String>,
    pub granularity: TimelineGranularity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRecord {
    pub id: String,
    pub summary: String,
    pub author_name: String,
    pub author_email: String,
    pub committed_at: DateTime<Utc>,
    pub files: Vec<FileDelta>,
}

impl CommitRecord {
    pub fn total_churn(&self) -> u64 {
        self.files.iter().map(|f| f.additions + f.deletions).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDelta {
    pub path: String,
    pub additions: u64,
    pub deletions: u64,
}
