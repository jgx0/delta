use std::collections::{BTreeMap, HashMap};
use std::process::Command;

const COMMIT_PREFIX: &str = "__COMMIT__|";

#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub repo: String,
    pub author: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub branch: Option<String>,
    pub file_type: Option<String>,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            repo: ".".to_string(),
            author: None,
            since: None,
            until: None,
            branch: None,
            file_type: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: String,
    pub added: u64,
    pub deleted: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitRecord {
    pub sha: String,
    pub author: String,
    pub date: String,
    pub file_changes: Vec<FileChange>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContributorStats {
    pub commits: u64,
    pub lines_added: u64,
    pub lines_deleted: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileStats {
    pub modifications: u64,
    pub lines_added: u64,
    pub lines_deleted: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub total_commits: u64,
    pub total_lines_added: u64,
    pub total_lines_deleted: u64,
    pub commit_frequency: BTreeMap<String, u64>,
    pub file_stats: HashMap<String, FileStats>,
    pub contributor_stats: HashMap<String, ContributorStats>,
    pub language_distribution: HashMap<String, u64>,
}

pub fn run_git_log(options: &QueryOptions) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(&options.repo)
        .arg("log")
        .arg("--numstat")
        .arg("--date=short")
        .arg("--pretty=format:__COMMIT__|%H|%an|%ad");

    if let Some(branch) = &options.branch {
        cmd.arg(branch);
    }
    if let Some(author) = &options.author {
        cmd.arg(format!("--author={author}"));
    }
    if let Some(since) = &options.since {
        cmd.arg(format!("--since={since}"));
    }
    if let Some(until) = &options.until {
        cmd.arg(format!("--until={until}"));
    }

    let output = cmd
        .output()
        .map_err(|err| format!("failed to execute git log: {err}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn parse_git_log_output(log_output: &str) -> Vec<CommitRecord> {
    let mut commits = Vec::new();
    let mut current_commit: Option<CommitRecord> = None;

    for line in log_output.lines() {
        if line.starts_with(COMMIT_PREFIX) {
            if let Some(commit) = current_commit.take() {
                commits.push(commit);
            }

            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                current_commit = Some(CommitRecord {
                    sha: parts[1].to_string(),
                    author: parts[2].to_string(),
                    date: parts[3].to_string(),
                    file_changes: Vec::new(),
                });
            }
            continue;
        }

        if line.trim().is_empty() {
            continue;
        }

        let Some(commit) = current_commit.as_mut() else {
            continue;
        };

        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() != 3 {
            continue;
        }

        let added = if parts[0] == "-" {
            0
        } else {
            parts[0].parse::<u64>().unwrap_or(0)
        };
        let deleted = if parts[1] == "-" {
            0
        } else {
            parts[1].parse::<u64>().unwrap_or(0)
        };

        commit.file_changes.push(FileChange {
            path: parts[2].to_string(),
            added,
            deleted,
        });
    }

    if let Some(commit) = current_commit {
        commits.push(commit);
    }

    commits
}

pub fn collect_metrics(commits: &[CommitRecord], file_type: Option<&str>) -> Metrics {
    let normalized_type = file_type.map(normalize_file_type);
    let mut metrics = Metrics::default();

    for commit in commits {
        let relevant_file_changes: Vec<&FileChange> = commit
            .file_changes
            .iter()
            .filter(|change| {
                normalized_type
                    .as_deref()
                    .map(|ext| file_matches_extension(&change.path, ext))
                    .unwrap_or(true)
            })
            .collect();

        if relevant_file_changes.is_empty() {
            continue;
        }

        metrics.total_commits += 1;
        *metrics
            .commit_frequency
            .entry(commit.date.clone())
            .or_insert(0) += 1;
        metrics
            .contributor_stats
            .entry(commit.author.clone())
            .or_default()
            .commits += 1;

        for change in relevant_file_changes {
            metrics.total_lines_added += change.added;
            metrics.total_lines_deleted += change.deleted;

            let contributor = metrics
                .contributor_stats
                .entry(commit.author.clone())
                .or_default();
            contributor.lines_added += change.added;
            contributor.lines_deleted += change.deleted;

            let file = metrics.file_stats.entry(change.path.clone()).or_default();
            file.modifications += 1;
            file.lines_added += change.added;
            file.lines_deleted += change.deleted;

            let language = file_language(&change.path);
            *metrics.language_distribution.entry(language).or_insert(0) +=
                change.added + change.deleted;
        }
    }

    metrics
}

fn normalize_file_type(file_type: &str) -> String {
    file_type.trim_start_matches('.').to_ascii_lowercase()
}

fn file_matches_extension(path: &str, extension: &str) -> bool {
    path.rsplit_once('.')
        .map(|(_, ext)| ext.eq_ignore_ascii_case(extension))
        .unwrap_or(false)
}

fn file_language(path: &str) -> String {
    path.rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn json_escape(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_git_numstat_log() {
        let input = "\
__COMMIT__|aaa111|alice|2026-01-01
10\t2\tsrc/main.rs
-\t-\tassets/logo.png

__COMMIT__|bbb222|bob|2026-01-02
3\t1\tREADME.md
";

        let commits = parse_git_log_output(input);
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].sha, "aaa111");
        assert_eq!(commits[0].author, "alice");
        assert_eq!(commits[0].file_changes.len(), 2);
        assert_eq!(commits[0].file_changes[1].added, 0);
        assert_eq!(commits[0].file_changes[1].deleted, 0);
    }

    #[test]
    fn filters_metrics_by_file_type() {
        let commits = vec![CommitRecord {
            sha: "aaa111".to_string(),
            author: "alice".to_string(),
            date: "2026-01-01".to_string(),
            file_changes: vec![
                FileChange {
                    path: "src/main.rs".to_string(),
                    added: 10,
                    deleted: 3,
                },
                FileChange {
                    path: "README.md".to_string(),
                    added: 4,
                    deleted: 1,
                },
            ],
        }];

        let metrics = collect_metrics(&commits, Some(".rs"));
        assert_eq!(metrics.total_commits, 1);
        assert_eq!(metrics.total_lines_added, 10);
        assert_eq!(metrics.total_lines_deleted, 3);
        assert_eq!(metrics.file_stats.len(), 1);
        assert!(metrics.file_stats.contains_key("src/main.rs"));
    }
}
