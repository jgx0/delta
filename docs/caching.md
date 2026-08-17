# Caching

`delta` caches analysis results so repeated queries are instant.

## Location

The cache is a SQLite database at `<repo>/.delta/cache.db`, created
automatically on first use. Add `.delta/` to your `.gitignore` to avoid
committing it.

## Schema

```
CREATE TABLE snapshots_v2 (
    id           INTEGER PRIMARY KEY,
    repo_path    TEXT NOT NULL,
    cache_key    TEXT NOT NULL,
    generated_at TEXT NOT NULL,
    payload      TEXT NOT NULL     -- serialized RepoSnapshot (JSON)
);
```

The `payload` column stores the full `RepoSnapshot` serialized as JSON.

## Cache key

The cache key captures everything that influences the analysis result:

- repository path
- commit limit
- `--since` / `--until` timestamps
- `--include` / `--exclude` globs
- timeline granularity

Two requests with different filters therefore never share a stale snapshot.
The key is built by `cache::sqlite::cache_key`.

## Invalidation

- `--refresh-cache` forces a re-analysis and a new snapshot insert.
- Because the table is named `snapshots_v2`, snapshots written by older
  versions of `delta` (which used a `snapshots` table) are ignored, avoiding
  schema drift.

## Notes

- `load_latest` returns the newest snapshot matching the cache key, or `None`
  on a miss, in which case the caller analyzes and stores a fresh snapshot.
- The cache is keyed to the *repository*, so analyzing the same repo from
  different working directories still shares a cache (the `repo_path` is
  canonicalized by `git2`'s repository discovery).
