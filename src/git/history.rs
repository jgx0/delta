use anyhow::Result;
use chrono::{DateTime, Utc};
use git2::{DiffFormat, Oid, Repository};

use crate::{
    models::{AnalysisOptions, CommitRecord, FileDelta},
    utils::glob::matches_any,
};

pub fn walk_commits(repo: &Repository, options: &AnalysisOptions) -> Result<Vec<CommitRecord>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME | git2::Sort::TOPOLOGICAL)?;

    let mut records = Vec::with_capacity(options.limit.min(4096));

    for oid in revwalk {
        if records.len() >= options.limit {
            break;
        }
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        let date = DateTime::from_timestamp(commit.time().seconds(), 0)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(DateTime::<Utc>::UNIX_EPOCH);

        if options.since.is_some_and(|s| date < s) || options.until.is_some_and(|u| date > u) {
            continue;
        }

        let tree = commit.tree()?;
        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

        let mut file_deltas = Vec::new();
        let mut current_file = String::new();
        let mut additions = 0_u64;
        let mut deletions = 0_u64;

        diff.print(DiffFormat::Patch, |delta, _hunk, line| {
            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "<unknown>".to_string());

            if path != current_file {
                if !current_file.is_empty() {
                    file_deltas.push(FileDelta {
                        path: current_file.clone(),
                        additions,
                        deletions,
                    });
                }
                current_file = path;
                additions = 0;
                deletions = 0;
            }

            match line.origin() {
                '+' => additions += 1,
                '-' => deletions += 1,
                _ => {}
            }
            true
        })?;

        if !current_file.is_empty() {
            file_deltas.push(FileDelta {
                path: current_file,
                additions,
                deletions,
            });
        }

        file_deltas.retain(|delta| path_included(delta, options));

        let author = commit.author();
        records.push(CommitRecord {
            id: format_oid(oid),
            summary: commit.summary().unwrap_or("<no message>").to_string(),
            author_name: author.name().unwrap_or("unknown").to_string(),
            author_email: author.email().unwrap_or("unknown").to_string(),
            committed_at: date,
            files: file_deltas,
        });
    }

    Ok(records)
}

fn path_included(delta: &FileDelta, options: &AnalysisOptions) -> bool {
    let included = options.include.is_empty() || matches_any(&options.include, &delta.path);
    let excluded = matches_any(&options.exclude, &delta.path);
    included && !excluded
}

fn format_oid(oid: Oid) -> String {
    oid.to_string().chars().take(12).collect()
}
