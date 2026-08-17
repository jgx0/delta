# Development

## Prerequisites

- Rust toolchain (edition 2024; let-chains are used, so Rust 1.88+)
- A Git repository to analyze

## Build & test

```bash
cargo build
cargo test
```

Run against this repository itself:

```bash
cargo run -- stats
cargo run -- hotspots
cargo run -- scan --tui
```

## Conventions

- One module per subcommand under `src/commands/`; each exposes
  `pub fn run(...) -> anyhow::Result<()>`.
- Metric computation lives in `src/analytics/` and is pure where possible
  (taking slices of model types) so it can be unit-tested without a
  repository.
- Command modules render a Unicode table by default and JSON when `--json` is
  passed, via `commands::emit_json`.
- New `RepoSnapshot` fields must be populated in `analytics::engine::analyze`;
  if they change what a snapshot means, the cache key or table name must be
  bumped (see [Caching](caching.md)).
- Unit tests use plain data construction (`CommitRecord`, `FileChurn`) rather
  than real repositories, keeping the suite fast and hermetic.

## Adding a command

1. Add a variant to `cli::Command`.
2. Create `src/commands/<name>.rs` with a `run` function.
3. Register the module in `src/commands/mod.rs` and add a dispatch arm.
4. Document it in `docs/commands.md` and the README.
