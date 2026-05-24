use std::collections::HashMap;
use std::env;
use std::process::exit;

use delta::{
    Metrics, QueryOptions, collect_metrics, json_escape, parse_git_log_output, run_git_log,
};

#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Text,
    Json,
    Csv,
}

#[derive(Debug, Clone, Copy)]
enum CommandName {
    Status,
    Churn,
    Hotspots,
    Contributors,
    Timeline,
}

fn main() {
    let (command, options, output) = match parse_cli_args(env::args().collect()) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{}", usage());
            exit(1);
        }
    };

    let log_output = match run_git_log(&options) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("Failed to query git history: {error}");
            exit(1);
        }
    };

    let commits = parse_git_log_output(&log_output);
    let metrics = collect_metrics(&commits, options.file_type.as_deref());

    let rendered = match command {
        CommandName::Status => render_status(&metrics, output),
        CommandName::Churn => render_churn(&metrics, output),
        CommandName::Hotspots => render_hotspots(&metrics, output),
        CommandName::Contributors => render_contributors(&metrics, output),
        CommandName::Timeline => render_timeline(&metrics, output),
    };

    println!("{rendered}");
}

fn usage() -> &'static str {
    "Usage: delta <status|churn|hotspots|contributors|timeline> [--repo <path>] [--author <author>] [--since <date>] [--until <date>] [--branch <branch>] [--file-type <ext>] [--output <text|json|csv>]"
}

fn parse_cli_args(args: Vec<String>) -> Result<(CommandName, QueryOptions, OutputFormat), String> {
    if args.len() < 2 {
        return Err("Missing command.".to_string());
    }

    let command = match args[1].as_str() {
        "status" => CommandName::Status,
        "churn" => CommandName::Churn,
        "hotspots" => CommandName::Hotspots,
        "contributors" => CommandName::Contributors,
        "timeline" => CommandName::Timeline,
        "--help" | "-h" => return Err("".to_string()),
        other => return Err(format!("Unknown command '{other}'.")),
    };

    let mut options = QueryOptions::default();
    let mut output = OutputFormat::Text;

    let mut idx = 2;
    while idx < args.len() {
        let flag = &args[idx];
        let value = args
            .get(idx + 1)
            .ok_or_else(|| format!("Flag '{flag}' requires a value."))?
            .clone();

        match flag.as_str() {
            "--repo" => options.repo = value,
            "--author" => options.author = Some(value),
            "--since" => options.since = Some(value),
            "--until" => options.until = Some(value),
            "--branch" => options.branch = Some(value),
            "--file-type" => options.file_type = Some(value),
            "--output" => {
                output = match value.as_str() {
                    "text" => OutputFormat::Text,
                    "json" => OutputFormat::Json,
                    "csv" => OutputFormat::Csv,
                    _ => return Err(format!("Unsupported output format '{value}'.")),
                }
            }
            other => return Err(format!("Unknown flag '{other}'.")),
        }
        idx += 2;
    }

    Ok((command, options, output))
}

fn render_status(metrics: &Metrics, output: OutputFormat) -> String {
    let mut file_rows: Vec<(&str, u64)> = metrics
        .file_stats
        .iter()
        .map(|(path, stats)| (path.as_str(), stats.modifications))
        .collect();
    file_rows.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    let most_modified_file = file_rows.first().map(|(path, _)| (*path).to_string());

    let mut language_rows: Vec<(&str, u64)> = metrics
        .language_distribution
        .iter()
        .map(|(lang, churn)| (lang.as_str(), *churn))
        .collect();
    language_rows.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    match output {
        OutputFormat::Text => {
            let top_languages = language_rows
                .iter()
                .take(5)
                .map(|(lang, churn)| format!("{lang}:{churn}"))
                .collect::<Vec<String>>()
                .join(", ");
            format!(
                "Repository status\n  commits: {}\n  lines added: {}\n  lines deleted: {}\n  contributors: {}\n  most modified file: {}\n  language distribution (top): {}",
                metrics.total_commits,
                metrics.total_lines_added,
                metrics.total_lines_deleted,
                metrics.contributor_stats.len(),
                most_modified_file.unwrap_or_else(|| "n/a".to_string()),
                if top_languages.is_empty() {
                    "n/a".to_string()
                } else {
                    top_languages
                }
            )
        }
        OutputFormat::Json => {
            let languages = language_rows
                .iter()
                .map(|(lang, churn)| {
                    format!(
                        "{{\"language\":\"{}\",\"churn\":{}}}",
                        json_escape(lang),
                        churn
                    )
                })
                .collect::<Vec<String>>()
                .join(",");
            format!(
                "{{\"commits\":{},\"lines_added\":{},\"lines_deleted\":{},\"contributors\":{},\"most_modified_file\":{},\"language_distribution\":[{}]}}",
                metrics.total_commits,
                metrics.total_lines_added,
                metrics.total_lines_deleted,
                metrics.contributor_stats.len(),
                most_modified_file
                    .map(|v| format!("\"{}\"", json_escape(&v)))
                    .unwrap_or_else(|| "null".to_string()),
                languages
            )
        }
        OutputFormat::Csv => {
            format!(
                "metric,value\ncommits,{}\nlines_added,{}\nlines_deleted,{}\ncontributors,{}\nmost_modified_file,{}",
                metrics.total_commits,
                metrics.total_lines_added,
                metrics.total_lines_deleted,
                metrics.contributor_stats.len(),
                most_modified_file.unwrap_or_default()
            )
        }
    }
}

fn render_churn(metrics: &Metrics, output: OutputFormat) -> String {
    let mut rows: Vec<(&str, u64, u64, u64)> = metrics
        .file_stats
        .iter()
        .map(|(path, stats)| {
            (
                path.as_str(),
                stats.modifications,
                stats.lines_added,
                stats.lines_deleted,
            )
        })
        .collect();
    rows.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    rows.truncate(20);

    match output {
        OutputFormat::Text => {
            if rows.is_empty() {
                return "No churn data found.".to_string();
            }

            let body = rows
                .iter()
                .map(|(path, modifications, added, deleted)| {
                    format!("  {path} | edits={modifications} added={added} deleted={deleted}")
                })
                .collect::<Vec<String>>()
                .join("\n");
            format!("File churn rankings\n{body}")
        }
        OutputFormat::Json => {
            let values = rows
                .iter()
                .map(|(path, modifications, added, deleted)| {
                    format!(
                        "{{\"file\":\"{}\",\"modifications\":{},\"lines_added\":{},\"lines_deleted\":{}}}",
                        json_escape(path),
                        modifications,
                        added,
                        deleted
                    )
                })
                .collect::<Vec<String>>()
                .join(",");
            format!("[{values}]")
        }
        OutputFormat::Csv => {
            let mut csv = String::from("file,modifications,lines_added,lines_deleted");
            for (path, modifications, added, deleted) in rows {
                csv.push_str(&format!(
                    "\n\"{}\",{},{},{}",
                    path.replace('"', "\"\""),
                    modifications,
                    added,
                    deleted
                ));
            }
            csv
        }
    }
}

fn render_hotspots(metrics: &Metrics, output: OutputFormat) -> String {
    let mut rows: Vec<(&str, u64, u64)> = metrics
        .file_stats
        .iter()
        .map(|(path, stats)| {
            let churn = stats.lines_added + stats.lines_deleted;
            let score = stats.modifications * churn.max(1);
            (path.as_str(), score, stats.modifications)
        })
        .collect();
    rows.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    rows.truncate(20);

    match output {
        OutputFormat::Text => {
            if rows.is_empty() {
                return "No hotspot data found.".to_string();
            }
            let body = rows
                .iter()
                .map(|(path, score, modifications)| {
                    format!("  {path} | score={score} edits={modifications}")
                })
                .collect::<Vec<String>>()
                .join("\n");
            format!("Hotspot analysis\n{body}")
        }
        OutputFormat::Json => {
            let values = rows
                .iter()
                .map(|(path, score, modifications)| {
                    format!(
                        "{{\"file\":\"{}\",\"score\":{},\"modifications\":{}}}",
                        json_escape(path),
                        score,
                        modifications
                    )
                })
                .collect::<Vec<String>>()
                .join(",");
            format!("[{values}]")
        }
        OutputFormat::Csv => {
            let mut csv = String::from("file,score,modifications");
            for (path, score, modifications) in rows {
                csv.push_str(&format!(
                    "\n\"{}\",{},{}",
                    path.replace('"', "\"\""),
                    score,
                    modifications
                ));
            }
            csv
        }
    }
}

fn render_contributors(metrics: &Metrics, output: OutputFormat) -> String {
    let mut rows: Vec<(&str, u64, u64, u64)> = metrics
        .contributor_stats
        .iter()
        .map(|(name, stats)| {
            (
                name.as_str(),
                stats.commits,
                stats.lines_added,
                stats.lines_deleted,
            )
        })
        .collect();
    rows.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    let bus_factor = estimate_bus_factor(&metrics.contributor_stats);

    match output {
        OutputFormat::Text => {
            if rows.is_empty() {
                return "No contributor activity found.".to_string();
            }
            let body = rows
                .iter()
                .map(|(name, commits, added, deleted)| {
                    format!("  {name} | commits={commits} added={added} deleted={deleted}")
                })
                .collect::<Vec<String>>()
                .join("\n");
            format!("Contributor activity (bus factor: {bus_factor})\n{body}")
        }
        OutputFormat::Json => {
            let values = rows
                .iter()
                .map(|(name, commits, added, deleted)| {
                    format!(
                        "{{\"author\":\"{}\",\"commits\":{},\"lines_added\":{},\"lines_deleted\":{}}}",
                        json_escape(name),
                        commits,
                        added,
                        deleted
                    )
                })
                .collect::<Vec<String>>()
                .join(",");
            format!("{{\"bus_factor\":{bus_factor},\"contributors\":[{values}]}}")
        }
        OutputFormat::Csv => {
            let mut csv =
                format!("bus_factor,{bus_factor}\nauthor,commits,lines_added,lines_deleted");
            for (name, commits, added, deleted) in rows {
                csv.push_str(&format!(
                    "\n\"{}\",{},{},{}",
                    name.replace('"', "\"\""),
                    commits,
                    added,
                    deleted
                ));
            }
            csv
        }
    }
}

fn render_timeline(metrics: &Metrics, output: OutputFormat) -> String {
    let rows: Vec<(&str, u64)> = metrics
        .commit_frequency
        .iter()
        .map(|(date, count)| (date.as_str(), *count))
        .collect();

    match output {
        OutputFormat::Text => {
            if rows.is_empty() {
                return "No timeline data found.".to_string();
            }
            let max_count = rows.iter().map(|(_, count)| *count).max().unwrap_or(1);
            let body = rows
                .iter()
                .map(|(date, count)| format!("  {date} {} ({count})", sparkline(*count, max_count)))
                .collect::<Vec<String>>()
                .join("\n");
            format!("Commit timeline\n{body}")
        }
        OutputFormat::Json => {
            let values = rows
                .iter()
                .map(|(date, count)| {
                    format!("{{\"date\":\"{}\",\"commits\":{count}}}", json_escape(date))
                })
                .collect::<Vec<String>>()
                .join(",");
            format!("[{values}]")
        }
        OutputFormat::Csv => {
            let mut csv = String::from("date,commits");
            for (date, count) in rows {
                csv.push_str(&format!("\n{date},{count}"));
            }
            csv
        }
    }
}

fn estimate_bus_factor(contributors: &HashMap<String, delta::ContributorStats>) -> u64 {
    if contributors.is_empty() {
        return 0;
    }

    let total_commits: u64 = contributors.values().map(|stats| stats.commits).sum();
    if total_commits == 0 {
        return 0;
    }

    let mut commits: Vec<u64> = contributors.values().map(|stats| stats.commits).collect();
    commits.sort_unstable_by(|a, b| b.cmp(a));

    let mut running = 0u64;
    for (index, commit_count) in commits.iter().enumerate() {
        running += *commit_count;
        if running * 100 >= total_commits * 50 {
            return (index + 1) as u64;
        }
    }

    commits.len() as u64
}

fn sparkline(value: u64, max: u64) -> &'static str {
    if max == 0 {
        return "▁";
    }
    let ratio = value as f64 / max as f64;
    if ratio < 0.15 {
        "▁"
    } else if ratio < 0.30 {
        "▂"
    } else if ratio < 0.45 {
        "▃"
    } else if ratio < 0.60 {
        "▄"
    } else if ratio < 0.75 {
        "▅"
    } else if ratio < 0.90 {
        "▆"
    } else if ratio < 0.98 {
        "▇"
    } else {
        "█"
    }
}
