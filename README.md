# delta
Understand how your code changes.

`delta` is a fully local Git analytics CLI focused on repository health, churn, contributor behavior, and development velocity.

## Commands

```bash
delta status
delta churn
delta hotspots
delta contributors
delta timeline
```

## Filters

All commands support:

- `--repo <path>`
- `--author <name>`
- `--since <YYYY-MM-DD>`
- `--until <YYYY-MM-DD>`
- `--branch <branch>`
- `--file-type <ext>`
- `--output <text|json|csv>`

## Examples

```bash
delta status --repo . --output json
delta churn --since 2026-01-01 --file-type rs
delta contributors --author "Jane"
delta timeline --branch main --output csv
```
