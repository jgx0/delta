# Metrics

This page describes how `delta` computes each metric.

## Churn

Churn is the sum of added and deleted lines for a file, contributor, or commit.

- **File churn** accumulates additions and deletions across every commit that
  touched the file.
- **Contributor churn** is the sum of that contributor's additions and
  deletions.
- **Commit churn** is the total lines changed in a single commit.

Line counts come from `git2`'s patch diff, counting each `+` and `-` line.

## Hotspots

A hotspot is a file that is both frequently changed and recently active. The
score is a weighted combination normalized across the repository:

| Factor | Weight | Meaning |
| --- | --- | --- |
| Frequency | 0.30 | commit count relative to the busiest file |
| Churn | 0.25 | total churn relative to the highest-churn file |
| Contributors | 0.15 | contributor count relative to the widest file |
| Recency | 0.20 | `1 / (1 + days_since_last_change / 30)` |
| Density | 0.10 | commits per day, normalized |

## Logical coupling

Two files are "coupled" when they change in the same commit. The coupling
weight is the number of commits that touched both files. Commits touching more
than 100 files are treated as bulk changes (mass reformats, vendoring, etc.)
and skipped to avoid noise. The top 100 pairs are kept.

## Ownership

For each file, `delta` tracks per-author churn. The dominant author is the one
with the most churn, and `dominant_share` is their fraction of the file's
total churn.

## Bus factor

The bus factor answers: *how many contributors would have to leave before half
of the project's churn is unaccounted for?*

`delta` sorts contributors by total churn and counts how many are needed to
reach 50% of the repository's total churn. A bus factor of 1 means one person
is responsible for half of all changes.

## At-risk files

A file is "at risk" when its knowledge is concentrated in one person:

- exactly one contributor has touched it, **or**
- the dominant author owns >= 85% of its churn.

At-risk files are ranked by churn and the top 20 are reported.

## Language breakdown

File churn is grouped by file extension (e.g. `rs`, `py`, `md`). Files without
an alphanumeric extension are grouped as `unknown`. The top 12 languages by
churn are kept.
