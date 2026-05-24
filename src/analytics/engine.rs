use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::Result;
use chrono::{Datelike, Utc};

use crate::{
    analytics::{hotspots::build_hotspots, timeline::build_timeline},
    git::{discover::open_repository, history::walk_commits},
    models::{
        CommitSummary, ContributorStats, FileChurn, LanguageStat, RepoSnapshot, RepoStats,
    },
};

pub fn analyze(path: &Path, limit: usize) -> Result<RepoSnapshot> {
    let repo = open_repository(path)?;
    let commits = walk_commits(&repo, limit)?;

    let commit_count = commits.len() as u64;
    let mut lines_added = 0_u64;
    let mut lines_deleted = 0_u64;

    let mut contributors: HashMap<String, ContributorStatsBuilder> = HashMap::new();
    let mut files: HashMap<String, FileChurnBuilder> = HashMap::new();
    let mut largest_commits = Vec::new();

    for commit in &commits {
        let mut commit_add = 0_u64;
        let mut commit_del = 0_u64;

        let c = contributors
            .entry(commit.author_name.clone())
            .or_insert_with(|| ContributorStatsBuilder::new(&commit.author_name));
        c.commit_count += 1;

        for f in &commit.files {
            commit_add += f.additions;
            commit_del += f.deletions;

            c.lines_added += f.additions;
            c.lines_deleted += f.deletions;
            c.files_touched.insert(f.path.clone());

            let file = files
                .entry(f.path.clone())
                .or_insert_with(|| FileChurnBuilder::new(&f.path));
            file.commit_count += 1;
            file.additions += f.additions;
            file.deletions += f.deletions;
            file.last_modified = file.last_modified.max(commit.committed_at);
            file.contributors.insert(commit.author_name.clone());
        }

        lines_added += commit_add;
        lines_deleted += commit_del;

        largest_commits.push(CommitSummary {
            id: commit.id.clone(),
            author: commit.author_name.clone(),
            summary: commit.summary.clone(),
            churn: commit_add + commit_del,
            committed_at: commit.committed_at,
        });
    }

    largest_commits.sort_by_key(|c| std::cmp::Reverse(c.churn));
    largest_commits.truncate(10);

    let mut contributors = contributors
        .into_values()
        .map(ContributorStatsBuilder::build)
        .collect::<Vec<_>>();
    contributors.sort_by_key(|c| std::cmp::Reverse(c.commit_count));

    let mut file_churn = files
        .into_values()
        .map(FileChurnBuilder::build)
        .collect::<Vec<_>>();
    file_churn.sort_by_key(|f| std::cmp::Reverse(f.total_churn()));

    let timeline = build_timeline(&commits);
    let hotspots = build_hotspots(&file_churn);
    let language_breakdown = build_language_breakdown(&file_churn);

    let repository_age_days = commits
        .iter()
        .map(|c| c.committed_at)
        .min()
        .map(|start| (Utc::now() - start).num_days().max(0) as u64)
        .unwrap_or(0);

    Ok(RepoSnapshot {
        generated_at: Utc::now(),
        stats: RepoStats {
            commit_count,
            lines_added,
            lines_deleted,
            repository_age_days,
            largest_commits,
            language_breakdown,
        },
        contributors,
        file_churn,
        timeline,
        hotspots,
    })
}

fn build_language_breakdown(files: &[FileChurn]) -> Vec<LanguageStat> {
    let mut by_lang: HashMap<String, u64> = HashMap::new();
    for f in files {
        let lang = f
            .path
            .rsplit('.')
            .next()
            .filter(|ext| ext.chars().all(|c| c.is_ascii_alphanumeric()))
            .unwrap_or("unknown")
            .to_ascii_lowercase();
        *by_lang.entry(lang).or_default() += f.total_churn();
    }

    let mut out = by_lang
        .into_iter()
        .map(|(language, churn)| LanguageStat { language, churn })
        .collect::<Vec<_>>();
    out.sort_by_key(|l| std::cmp::Reverse(l.churn));
    out.truncate(12);
    out
}

struct ContributorStatsBuilder {
    name: String,
    commit_count: u64,
    lines_added: u64,
    lines_deleted: u64,
    files_touched: HashSet<String>,
}

impl ContributorStatsBuilder {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            commit_count: 0,
            lines_added: 0,
            lines_deleted: 0,
            files_touched: HashSet::new(),
        }
    }

    fn build(self) -> ContributorStats {
        ContributorStats {
            name: self.name,
            commit_count: self.commit_count,
            lines_added: self.lines_added,
            lines_deleted: self.lines_deleted,
            files_touched: self.files_touched.len() as u64,
        }
    }
}

struct FileChurnBuilder {
    path: String,
    commit_count: u64,
    additions: u64,
    deletions: u64,
    last_modified: chrono::DateTime<Utc>,
    contributors: HashSet<String>,
}

impl FileChurnBuilder {
    fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            commit_count: 0,
            additions: 0,
            deletions: 0,
            last_modified: chrono::DateTime::<Utc>::UNIX_EPOCH,
            contributors: HashSet::new(),
        }
    }

    fn build(self) -> FileChurn {
        FileChurn {
            path: self.path,
            commit_count: self.commit_count,
            additions: self.additions,
            deletions: self.deletions,
            contributor_count: self.contributors.len() as u64,
            last_modified: self.last_modified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::build_language_breakdown;
    use crate::models::FileChurn;
    use chrono::Utc;

    #[test]
    fn language_breakdown_groups_extensions() {
        let files = vec![
            FileChurn {
                path: "src/main.rs".into(),
                commit_count: 1,
                additions: 10,
                deletions: 2,
                contributor_count: 1,
                last_modified: Utc::now(),
            },
            FileChurn {
                path: "src/lib.rs".into(),
                commit_count: 1,
                additions: 5,
                deletions: 1,
                contributor_count: 1,
                last_modified: Utc::now(),
            },
            FileChurn {
                path: "README".into(),
                commit_count: 1,
                additions: 4,
                deletions: 1,
                contributor_count: 1,
                last_modified: Utc::now(),
            },
        ];

        let langs = build_language_breakdown(&files);
        assert_eq!(langs[0].language, "rs");
        assert_eq!(langs[0].churn, 18);
    }
}
