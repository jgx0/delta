use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::Result;
use chrono::Utc;

use crate::{
    analytics::{coupling::build_coupling, hotspots::build_hotspots, timeline::build_timeline},
    git::{discover::open_repository, history::walk_commits},
    models::{
        AnalysisOptions, AtRiskFile, BusFactorInfo, CommitSummary, ContributorStats, FileChurn,
        LanguageStat, Ownership, RepoSnapshot, RepoStats,
    },
};

/// Fraction of a file's churn that makes it "at risk" of single-owner knowledge.
const AT_RISK_SHARE: f64 = 0.85;
const MAX_AT_RISK_FILES: usize = 20;

pub fn analyze(path: &Path, options: &AnalysisOptions) -> Result<RepoSnapshot> {
    let repo = open_repository(path)?;
    let commits = walk_commits(&repo, options)?;

    let commit_count = commits.len() as u64;
    let mut lines_added = 0_u64;
    let mut lines_deleted = 0_u64;

    let mut contributors: HashMap<String, ContributorBuilder> = HashMap::new();
    let mut files: HashMap<String, FileChurnBuilder> = HashMap::new();
    let mut largest_commits = Vec::new();

    for commit in &commits {
        let mut commit_add = 0_u64;
        let mut commit_del = 0_u64;

        let contributor = contributors
            .entry(commit.author_name.clone())
            .or_insert_with(|| ContributorBuilder::new(&commit.author_name));
        contributor.commit_count += 1;

        for f in &commit.files {
            let file_churn = f.additions + f.deletions;
            commit_add += f.additions;
            commit_del += f.deletions;

            contributor.lines_added += f.additions;
            contributor.lines_deleted += f.deletions;
            contributor.files_touched.insert(f.path.clone());

            let file = files
                .entry(f.path.clone())
                .or_insert_with(|| FileChurnBuilder::new(&f.path));
            file.commit_count += 1;
            file.additions += f.additions;
            file.deletions += f.deletions;
            file.last_modified = file.last_modified.max(commit.committed_at);
            file.contributors.insert(commit.author_name.clone());
            *file
                .contributor_churn
                .entry(commit.author_name.clone())
                .or_default() += file_churn;
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

    // Derived metrics computed from the raw builders before they are consumed.
    let ownership = build_ownership(&files);
    let bus_factor = build_bus_factor(&contributors, &ownership);
    let coupling = build_coupling(&commits);

    let mut contributors = contributors
        .into_values()
        .map(ContributorBuilder::build)
        .collect::<Vec<_>>();
    contributors.sort_by_key(|c| std::cmp::Reverse(c.commit_count));

    let mut file_churn = files
        .into_values()
        .map(FileChurnBuilder::build)
        .collect::<Vec<_>>();
    file_churn.sort_by_key(|f| std::cmp::Reverse(f.total_churn()));

    let timeline = build_timeline(&commits, options.granularity);
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
            bus_factor,
        },
        contributors,
        file_churn,
        timeline,
        hotspots,
        ownership,
        coupling,
        commits,
    })
}

fn build_ownership(files: &HashMap<String, FileChurnBuilder>) -> Vec<Ownership> {
    let mut ownership = files
        .values()
        .map(|b| {
            let (dominant_author, dominant_churn) = b
                .contributor_churn
                .iter()
                .max_by_key(|(_, &churn)| churn)
                .map(|(author, &churn)| (author.clone(), churn))
                .unwrap_or_else(|| ("unknown".to_string(), 0));

            let total_churn: u64 = b.contributor_churn.values().sum();

            Ownership {
                path: b.path.clone(),
                dominant_author,
                dominant_share: if total_churn == 0 {
                    0.0
                } else {
                    dominant_churn as f64 / total_churn as f64
                },
                total_churn,
                contributor_count: b.contributors.len() as u64,
            }
        })
        .collect::<Vec<_>>();

    ownership.sort_by_key(|o| std::cmp::Reverse(o.total_churn));
    ownership
}

fn build_bus_factor(
    contributors: &HashMap<String, ContributorBuilder>,
    ownership: &[Ownership],
) -> BusFactorInfo {
    let mut by_churn = contributors
        .values()
        .map(|c| (c.name.clone(), c.lines_added + c.lines_deleted))
        .collect::<Vec<_>>();
    by_churn.sort_by_key(|(_, churn)| std::cmp::Reverse(*churn));

    let total_churn: u64 = by_churn.iter().map(|(_, churn)| churn).sum();

    let bus_factor = if total_churn == 0 {
        0
    } else {
        let mut cumulative = 0_u64;
        let mut factor = by_churn.len() as u64;
        for (i, (_, churn)) in by_churn.iter().enumerate() {
            cumulative += *churn;
            if cumulative * 2 >= total_churn {
                factor = (i + 1) as u64;
                break;
            }
        }
        factor
    };

    let mut at_risk_files = ownership
        .iter()
        .filter(|o| o.contributor_count == 1 || o.dominant_share >= AT_RISK_SHARE)
        .map(|o| AtRiskFile {
            path: o.path.clone(),
            author: o.dominant_author.clone(),
            share: o.dominant_share,
            churn: o.total_churn,
        })
        .collect::<Vec<_>>();
    at_risk_files.sort_by_key(|f| std::cmp::Reverse(f.churn));
    at_risk_files.truncate(MAX_AT_RISK_FILES);

    BusFactorInfo {
        bus_factor,
        at_risk_files,
    }
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

struct ContributorBuilder {
    name: String,
    commit_count: u64,
    lines_added: u64,
    lines_deleted: u64,
    files_touched: HashSet<String>,
}

impl ContributorBuilder {
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
    contributor_churn: HashMap<String, u64>,
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
            contributor_churn: HashMap::new(),
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
    use std::collections::HashMap;

    use super::{build_bus_factor, build_language_breakdown, build_ownership, ContributorBuilder, FileChurnBuilder};
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

    fn file_builder(path: &str, author: &str, churn: u64) -> FileChurnBuilder {
        let mut b = FileChurnBuilder::new(path);
        b.contributors.insert(author.to_string());
        b.contributor_churn.insert(author.to_string(), churn);
        b.additions = churn;
        b.deletions = 0;
        b
    }

    #[test]
    fn ownership_reports_dominant_share() {
        let mut files = HashMap::new();
        let mut b = file_builder("a.rs", "alice", 90);
        b.contributors.insert("bob".to_string());
        b.contributor_churn.insert("bob".to_string(), 10);
        files.insert("a.rs".into(), b);

        let ownership = build_ownership(&files);
        assert_eq!(ownership[0].dominant_author, "alice");
        assert!((ownership[0].dominant_share - 0.9).abs() < 1e-9);
        assert_eq!(ownership[0].contributor_count, 2);
    }

    #[test]
    fn bus_factor_counts_contributors_covering_half_the_churn() {
        let mut contributors = HashMap::new();
        for (name, churn) in [("a", 60_u64), ("b", 30), ("c", 10)] {
            let mut c = ContributorBuilder::new(name);
            c.lines_added = churn;
            contributors.insert(name.to_string(), c);
        }
        let ownership = vec![];
        let bf = build_bus_factor(&contributors, &ownership);
        assert_eq!(bf.bus_factor, 1);
    }

    #[test]
    fn bus_factor_flags_single_owner_files() {
        let mut contributors = HashMap::new();
        let mut c = ContributorBuilder::new("a");
        c.lines_added = 100;
        contributors.insert("a".to_string(), c);

        let mut files = HashMap::new();
        files.insert("solo.rs".into(), file_builder("solo.rs", "a", 100));

        let ownership = build_ownership(&files);
        let bf = build_bus_factor(&contributors, &ownership);
        assert_eq!(bf.at_risk_files.len(), 1);
        assert_eq!(bf.at_risk_files[0].path, "solo.rs");
    }
}
