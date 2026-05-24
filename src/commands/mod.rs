use std::path::Path;

use anyhow::Result;

use crate::{
    cache::sqlite::SnapshotCache,
    cli::{Cli, Command},
    models::RepoSnapshot,
};

mod churn;
mod contributors;
mod hotspots;
mod scan;
mod stats;
mod timeline;

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Scan {
            path,
            limit,
            refresh_cache,
            tui,
        } => scan::run(&path, limit, refresh_cache, tui),
        Command::Stats {
            path,
            limit,
            refresh_cache,
        } => stats::run(&path, limit, refresh_cache),
        Command::Contributors {
            path,
            limit,
            refresh_cache,
        } => contributors::run(&path, limit, refresh_cache),
        Command::Churn {
            path,
            limit,
            refresh_cache,
        } => churn::run(&path, limit, refresh_cache),
        Command::Hotspots {
            path,
            limit,
            refresh_cache,
        } => hotspots::run(&path, limit, refresh_cache),
        Command::Timeline {
            path,
            limit,
            refresh_cache,
        } => timeline::run(&path, limit, refresh_cache),
    }
}

pub(crate) fn load_or_analyze(path: &Path, limit: usize, refresh_cache: bool) -> Result<RepoSnapshot> {
    let cache = SnapshotCache::new(path)?;
    if !refresh_cache {
        if let Some(snapshot) = cache.load_latest(path)? {
            return Ok(snapshot);
        }
    }

    let snapshot = crate::analytics::engine::analyze(path, limit)?;
    cache.store(path, &snapshot)?;
    Ok(snapshot)
}
