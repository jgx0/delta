use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::{
    cli::{CommonArgs, ExportFormat},
    commands::load_or_analyze,
    exporters::{csv, json, markdown},
    models::TimelineGranularity,
};

pub fn run(common: &CommonArgs, format: ExportFormat, output: Option<PathBuf>) -> Result<()> {
    let snapshot = load_or_analyze(common, TimelineGranularity::Day)?;

    let rendered = match format {
        ExportFormat::Json => json::export(&snapshot)?,
        ExportFormat::Csv => csv::export_file_churn(&snapshot)?,
        ExportFormat::Markdown => markdown::export(&snapshot),
    };

    match output {
        Some(path) => fs::write(&path, rendered)?,
        None => print!("{rendered}"),
    }

    Ok(())
}
