use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "delta", version, about = "Terminal-first Git analytics platform")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Analyze repository and persist a snapshot cache
    Scan {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value_t = 5000)]
        limit: usize,
        #[arg(long)]
        refresh_cache: bool,
        #[arg(long)]
        tui: bool,
    },
    /// Show high-level repository statistics
    Stats {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value_t = 5000)]
        limit: usize,
        #[arg(long)]
        refresh_cache: bool,
    },
    /// Show top contributors
    Contributors {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value_t = 5000)]
        limit: usize,
        #[arg(long)]
        refresh_cache: bool,
    },
    /// Show file churn rankings
    Churn {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value_t = 5000)]
        limit: usize,
        #[arg(long)]
        refresh_cache: bool,
    },
    /// Show hotspot rankings
    Hotspots {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value_t = 5000)]
        limit: usize,
        #[arg(long)]
        refresh_cache: bool,
    },
    /// Show commit activity timeline
    Timeline {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value_t = 5000)]
        limit: usize,
        #[arg(long)]
        refresh_cache: bool,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
