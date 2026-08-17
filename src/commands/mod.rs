use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::{
    cache::sqlite::{SnapshotCache, cache_key},
    cli::{Cli, Command, CommonArgs},
    models::{AnalysisOptions, RepoSnapshot, TimelineGranularity},
};

mod bus_factor;
mod churn;
mod contributors;
mod coupling;
mod export;
mod hotspots;
mod languages;
mod log;
mod ownership;
mod scan;
mod stats;
mod timeline;

pub fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Scan { common, tui } => scan::run(&common, tui),
        Command::Stats { common } => stats::run(&common),
        Command::Contributors { common } => contributors::run(&common),
        Command::Churn { common } => churn::run(&common),
        Command::Hotspots { common } => hotspots::run(&common),
        Command::Timeline { common, granularity } => timeline::run(&common, granularity),
        Command::Languages { common } => languages::run(&common),
        Command::Log { common } => log::run(&common),
        Command::BusFactor { common } => bus_factor::run(&common),
        Command::Coupling { common } => coupling::run(&common),
        Command::Ownership { common } => ownership::run(&common),
        Command::Export {
            common,
            format,
            output,
        } => export::run(&common, format, output),
    }
}

/// Load a cached snapshot matching the given options, or analyze from scratch
/// and cache the result.
pub(crate) fn load_or_analyze(
    common: &CommonArgs,
    granularity: TimelineGranularity,
) -> Result<RepoSnapshot> {
    let options = common.to_options(granularity)?;
    load_or_analyze_with_options(&common.path, &options, common.refresh_cache)
}

pub(crate) fn load_or_analyze_with_options(
    path: &Path,
    options: &AnalysisOptions,
    refresh_cache: bool,
) -> Result<RepoSnapshot> {
    let cache = SnapshotCache::new(path)?;
    let key = cache_key(path, options);
    if !refresh_cache && let Some(snapshot) = cache.load_latest(path, &key)? {
        return Ok(snapshot);
    }

    let snapshot = crate::analytics::engine::analyze(path, options)?;
    cache.store(path, &key, &snapshot)?;
    Ok(snapshot)
}

/// Print a value as pretty JSON.
pub(crate) fn emit_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
