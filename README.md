# delta
Understand how your code changes.

## MVP commands

```bash
delta scan [path] [--limit N] [--refresh-cache] [--tui]
delta stats [path]
delta contributors [path]
delta churn [path]
delta hotspots [path]
delta timeline [path]
```

## What is implemented

- Rust modular architecture for CLI, git analysis, analytics engine, TUI, cache, models, exporters, and utils
- `git2`-based commit history parsing and per-file diff churn analysis
- contributor rankings, file churn rankings, hotspot scoring, timeline aggregation
- terminal Unicode table rendering for command output
- basic `ratatui` dashboard (`delta scan --tui`, quit with `q`/`Esc`)
- SQLite snapshot cache at `<repo>/.delta/cache.db`
- JSON/CSV/Markdown exporter modules for future command wiring
