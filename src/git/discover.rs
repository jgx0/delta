use std::path::Path;

use anyhow::{Context, Result};
use git2::Repository;

pub fn open_repository(path: &Path) -> Result<Repository> {
    Repository::discover(path)
        .with_context(|| format!("failed to discover git repository from {}", path.display()))
}
