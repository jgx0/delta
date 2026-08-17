# Architecture

`delta` is a Rust binary + library. `src/main.rs` is a thin entry point that
delegates to `delta::app::run()`.

## Module layout

```
src/
├── main.rs            entry point
├── lib.rs             crate root, declares modules
├── cli/               clap command and argument definitions
├── app/               wires CLI parsing to command dispatch
├── commands/          one module per subcommand
├── git/               git2 repository discovery and history walking
├── analytics/         metric computation (engine, hotspots, timeline, coupling)
├── models/            serializable data structures and analysis options
├── cache/             SQLite snapshot cache
├── exporters/         JSON / CSV / Markdown rendering
├── tui/               ratatui dashboard
└── utils/             table rendering, glob matching, date parsing
```

## Data flow

```
CLI (clap)
  └─ commands::dispatch
       └─ commands::load_or_analyze
            ├─ cache::sqlite::load_latest   (cache hit)
            └─ analytics::engine::analyze   (cache miss)
                 ├─ git::discover::open_repository
                 ├─ git::history::walk_commits   (filters applied here)
                 ├─ analytics::{hotspots, timeline, coupling}
                 └─ cache::sqlite::store
```

## Key types

- `RepoSnapshot` — the single serializable result of an analysis. It holds
  stats, contributors, file churn, timeline, hotspots, ownership, coupling,
  and the raw commit records.
- `AnalysisOptions` — the inputs that shape an analysis: commit limit, date
  window, include/exclude globs, and timeline granularity. This is what the
  cache key is derived from.
- `CommitRecord` / `FileDelta` — the raw per-commit, per-file data produced by
  the history walker and consumed by the analytics engine.

## Design notes

- The analytics engine aggregates into private "builder" structs
  (`ContributorBuilder`, `FileChurnBuilder`) before producing the public
  model types. Builders carry extra state (e.g. per-author churn per file)
  needed only during aggregation.
- Filters (date, include/exclude) are applied during the history walk so the
  whole downstream pipeline only sees the requested data.
- Command modules are thin: they load a snapshot and render it, either as a
  Unicode table or as JSON.
