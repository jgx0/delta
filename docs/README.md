# delta documentation

`delta` is a terminal-first Git analytics platform. It reads a repository's
commit history and produces churn, hotspot, ownership, coupling, and
bus-factor metrics, caching the result locally so later queries are instant.

## Contents

- [Commands](commands.md) — every subcommand and its flags
- [Metrics](metrics.md) — how each metric is computed
- [Architecture](architecture.md) — module layout and data flow
- [Caching](caching.md) — the SQLite snapshot cache
- [Exporters](exporters.md) — JSON / CSV / Markdown output
- [Development](development.md) — building, testing, and conventions

## Quick start

```bash
cargo build --release

# Analyze the current repository and cache the result
./target/release/delta scan

# High-level overview
./target/release/delta stats

# Find churn hotspots
./target/release/delta hotspots

# Identify knowledge concentration risks
./target/release/delta bus-factor
```
