# delta

Understand how your code changes. A terminal-first Git analytics platform that
turns commit history into churn, hotspot, ownership, coupling, and bus-factor
insights.

## Install

```bash
cargo build --release
./target/release/delta --help
```

## Commands

```bash
delta scan [path] [--tui] [--json]
delta stats [path] [--json]
delta contributors [path] [--json]
delta churn [path] [--json]
delta hotspots [path] [--json]
delta timeline [path] [--granularity day|week|month|year] [--json]
delta languages [path] [--json]
delta log [path] [--json]
delta bus-factor [path] [--json]
delta coupling [path] [--json]
delta ownership [path] [--json]
delta export [path] --format json|csv|markdown [--output FILE]
```

### Shared options

Every command accepts these flags:

| Flag | Description |
| --- | --- |
| `[path]` | Repository to analyze (defaults to `.`) |
| `--limit N` | Maximum number of commits to analyze (default `5000`) |
| `--refresh-cache` | Ignore the snapshot cache and re-analyze |
| `--since DATE` | Only commits on/after a date (`YYYY-MM-DD` or RFC 3339) |
| `--until DATE` | Only commits on/before a date |
| `--include GLOB` | Only files matching the glob (repeatable, e.g. `*.rs`) |
| `--exclude GLOB` | Exclude files matching the glob (repeatable) |
| `--json` | Emit machine-readable JSON instead of a table |

## What is implemented

- Modular Rust architecture: CLI, git parsing, analytics engine, TUI, cache,
  models, exporters, and utils
- `git2`-based commit history walking and per-file diff churn analysis
- Contributor rankings, file churn, hotspot scoring, timeline aggregation
- Logical coupling (files that change together), per-file ownership, and bus
  factor / at-risk file detection
- Language churn breakdown and a recent-commit log
- Date and path (glob) filtering across all analyses
- `ratatui` dashboard (`delta scan --tui`, quit with `q`/`Esc`)
- SQLite snapshot cache at `<repo>/.delta/cache.db`, keyed by analysis options
- JSON, CSV, and Markdown exporters wired through `delta export`

## Documentation

See [`docs/`](docs/README.md) for the full reference:

- [Commands](docs/commands.md)
- [Metrics](docs/metrics.md)
- [Architecture](docs/architecture.md)
- [Caching](docs/caching.md)
- [Exporters](docs/exporters.md)
- [Development](docs/development.md)
