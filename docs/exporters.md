# Exporters

Exporters render a `RepoSnapshot` into different formats. They live in
`src/exporters/` and are exposed through `delta export`.

## JSON

`exporters::json::export` serializes the entire snapshot as pretty-printed
JSON. This is the default `delta export --format json` output and matches the
`--json` flag's payload on individual commands.

## CSV

`exporters::csv` provides two tables:

- `export_file_churn` — the file-churn table (path, commits, additions,
  deletions, churn, contributor count). This is what `delta export
  --format csv` emits.
- `export_contributors` — the contributor table (name, commits, added,
  deleted, files touched). Available to callers of the library.

## Markdown

`exporters::markdown::export` renders a human-readable report with sections
for repository stats, language breakdown, top contributors, hotspots, file
churn, logical coupling, and at-risk files. Suitable for pasting into GitHub
or a wiki.

```bash
delta export --format markdown --output report.md
```

## Adding a format

1. Add a function in `src/exporters/` that takes `&RepoSnapshot` and returns
   `String` (or `anyhow::Result<String>`).
2. Add a variant to `cli::ExportFormat`.
3. Handle the variant in `commands::export::run`.
