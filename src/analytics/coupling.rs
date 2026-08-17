use std::collections::HashMap;

use crate::models::{CommitRecord, Coupling};

/// Commits touching more files than this are treated as bulk changes
/// (e.g. mass reformats) and skipped, since they would flood the coupling
/// output with noise.
const MAX_FILES_PER_COMMIT: usize = 100;

/// How many top co-changing pairs to retain.
const MAX_PAIRS: usize = 100;

/// Compute logical coupling: pairs of files that change together in the same
/// commit, weighted by how often that co-change happens.
pub fn build_coupling(commits: &[CommitRecord]) -> Vec<Coupling> {
    let mut co_changes: HashMap<(String, String), u64> = HashMap::new();

    for commit in commits {
        if commit.files.len() > MAX_FILES_PER_COMMIT {
            continue;
        }

        let paths: Vec<&str> = commit.files.iter().map(|f| f.path.as_str()).collect();
        for i in 0..paths.len() {
            for j in (i + 1)..paths.len() {
                let (a, b) = ordered(paths[i], paths[j]);
                *co_changes.entry((a, b)).or_default() += 1;
            }
        }
    }

    let mut pairs = co_changes
        .into_iter()
        .map(|((file_a, file_b), co_changes)| Coupling {
            file_a,
            file_b,
            co_changes,
        })
        .collect::<Vec<_>>();

    pairs.sort_by_key(|c| std::cmp::Reverse(c.co_changes));
    pairs.truncate(MAX_PAIRS);
    pairs
}

fn ordered(a: &str, b: &str) -> (String, String) {
    if a <= b {
        (a.to_string(), b.to_string())
    } else {
        (b.to_string(), a.to_string())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::build_coupling;
    use crate::models::{CommitRecord, FileDelta};

    fn commit_with(id: &str, files: &[&str]) -> CommitRecord {
        CommitRecord {
            id: id.into(),
            summary: "s".into(),
            author_name: "x".into(),
            author_email: "x@x".into(),
            committed_at: Utc::now(),
            files: files
                .iter()
                .map(|p| FileDelta {
                    path: (*p).into(),
                    additions: 1,
                    deletions: 1,
                })
                .collect(),
        }
    }

    #[test]
    fn coupling_counts_co_changes() {
        let commits = vec![
            commit_with("1", &["a.rs", "b.rs", "c.rs"]),
            commit_with("2", &["a.rs", "b.rs"]),
            commit_with("3", &["c.rs", "d.rs"]),
        ];

        let pairs = build_coupling(&commits);
        let ab = pairs
            .iter()
            .find(|c| c.file_a == "a.rs" && c.file_b == "b.rs")
            .unwrap();
        assert_eq!(ab.co_changes, 2);
    }

    #[test]
    fn coupling_orders_pairs_lexicographically() {
        let commits = vec![commit_with("1", &["z.rs", "a.rs"])];
        let pairs = build_coupling(&commits);
        assert_eq!(pairs[0].file_a, "a.rs");
        assert_eq!(pairs[0].file_b, "z.rs");
    }

    #[test]
    fn coupling_skips_bulk_commits() {
        let many: Vec<String> = (0..200).map(|i| format!("f{i}.rs")).collect();
        let refs: Vec<&str> = many.iter().map(|s| s.as_str()).collect();
        let commits = vec![commit_with("bulk", &refs)];
        assert!(build_coupling(&commits).is_empty());
    }
}
