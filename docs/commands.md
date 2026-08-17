# Commands

All commands analyze a repository (defaulting to the current directory) and
cache the resulting snapshot in `<repo>/.delta/cache.db`. They share a common
set of options described below.

## Shared options

```
[path]                     Repository to analyze (default ".")
--limit <N>                Maximum commits to analyze (default 5000)
--refresh-cache            Re-analyze instead of loading the cache
--since <DATE>             Only commits on/after this date
--until <DATE>             Only commits on/before this date
--include <GLOB>           Only files matching this glob (repeatable)
--exclude <GLOB>           Exclude files matching this glob (repeatable)
--json                     Print machine-readable JSON instead of a table
```

Dates accept `YYYY-MM-DD` (interpreted as UTC midnight) or a full RFC 3339
timestamp such as `2024-01-15T00:00:00Z`.

Globs support `*` and `?`. A pattern without `/` (e.g. `*.rs`) matches against
the file's basename, so `--include "*.rs"` selects all Rust files regardless
of directory. A pattern with `/` (e.g. `src/**`) matches the full path.

## `scan`

Analyzes and persists a snapshot. Without flags it prints a one-line summary.

```bash
delta scan                      # analyze and summarize
delta scan --tui                # open the interactive dashboard
delta scan --json               # print the full snapshot as JSON
```

## `stats`

High-level repository statistics: commit count, lines added/deleted,
repository age, bus factor, largest commits, and the language breakdown.

```bash
delta stats
```

## `contributors`

Top contributors ranked by commit count, with lines added/deleted, total
churn, and files touched.

```bash
delta contributors
```

## `churn`

Files ranked by total churn (additions + deletions), with commit count and
number of contributors.

```bash
delta churn
```

## `hotspots`

Files ranked by hotspot score (see [Metrics](metrics.md#hotspots)).

```bash
delta hotspots
```

## `timeline`

Commit activity bucketed by time. The `--granularity` flag controls the
bucket size (`day` is the default).

```bash
delta timeline --granularity week
delta timeline --granularity month
```

## `languages`

Churn broken down by file extension, with a share bar.

```bash
delta languages
```

## `log`

The most recent commits (newest first) with hash, date, author, churn, file
count, and summary.

```bash
delta log
```

## `bus-factor`

The repository's bus factor and the files whose knowledge is concentrated in
one author.

```bash
delta bus-factor
```

## `coupling`

Pairs of files that frequently change together in the same commit (logical
coupling).

```bash
delta coupling
```

## `ownership`

Per-file dominant author, their share of the file's churn, total churn, and
contributor count.

```bash
delta ownership
```

## `export`

Render the snapshot to a file or stdout.

```bash
delta export --format json
delta export --format markdown --output report.md
delta export --format csv --output churn.csv
```

`--format csv` exports the file-churn table; `--format json` and
`--format markdown` export the full snapshot. See
[Exporters](exporters.md).
