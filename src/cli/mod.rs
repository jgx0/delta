use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::models::{AnalysisOptions, TimelineGranularity};
use crate::utils::date::parse_date;

#[derive(Debug, Parser)]
#[command(
    name = "delta",
    version,
    about = "Terminal-first Git analytics platform"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Analyze repository and persist a snapshot cache
    Scan {
        #[command(flatten)]
        common: CommonArgs,
        #[arg(long)]
        tui: bool,
    },
    /// Show high-level repository statistics
    Stats {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Show top contributors
    Contributors {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Show file churn rankings
    Churn {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Show hotspot rankings
    Hotspots {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Show commit activity timeline
    Timeline {
        #[command(flatten)]
        common: CommonArgs,
        #[arg(long, value_enum, default_value_t = TimelineGranularity::Day)]
        granularity: TimelineGranularity,
    },
    /// Show language churn breakdown
    Languages {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Show recent commits
    Log {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Show bus factor and at-risk files
    BusFactor {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Show files that change together
    Coupling {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Show per-file ownership
    Ownership {
        #[command(flatten)]
        common: CommonArgs,
    },
    /// Export the analysis snapshot to a file or stdout
    Export {
        #[command(flatten)]
        common: CommonArgs,
        #[arg(long, value_enum, default_value_t = ExportFormat::Json)]
        format: ExportFormat,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// Options shared by every analysis command.
#[derive(Debug, Args)]
pub struct CommonArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Maximum number of commits to analyze
    #[arg(long, default_value_t = 5000)]
    pub limit: usize,
    /// Ignore the cache and re-analyze from scratch
    #[arg(long)]
    pub refresh_cache: bool,
    /// Only include commits on or after this date (YYYY-MM-DD or RFC 3339)
    #[arg(long)]
    pub since: Option<String>,
    /// Only include commits on or before this date (YYYY-MM-DD or RFC 3339)
    #[arg(long)]
    pub until: Option<String>,
    /// Only include files matching these glob patterns (repeatable)
    #[arg(long)]
    pub include: Vec<String>,
    /// Exclude files matching these glob patterns (repeatable)
    #[arg(long)]
    pub exclude: Vec<String>,
    /// Emit machine-readable JSON instead of a table
    #[arg(long)]
    pub json: bool,
}

impl CommonArgs {
    pub fn to_options(&self, granularity: TimelineGranularity) -> anyhow::Result<AnalysisOptions> {
        Ok(AnalysisOptions {
            limit: self.limit,
            since: self.since.as_deref().map(parse_date).transpose()?,
            until: self.until.as_deref().map(parse_date).transpose()?,
            include: self.include.clone(),
            exclude: self.exclude.clone(),
            granularity,
        })
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportFormat {
    Json,
    Csv,
    Markdown,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Markdown => "markdown",
        })
    }
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
