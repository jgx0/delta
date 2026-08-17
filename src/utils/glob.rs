//! Lightweight glob matching for path filters (`--include` / `--exclude`).
//!
//! Supports `*` (any run of characters) and `?` (exactly one character).
//! Patterns without a `/` match against the file's basename, which makes
//! `--include "*.rs"` behave the way most users expect.

/// Match a single wildcard pattern against a string.
pub fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();

    let mut pi = 0;
    let mut ti = 0;
    let mut star: Option<usize> = None;
    let mut star_match = 0;

    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            star_match = ti;
            pi += 1;
        } else if let Some(s) = star {
            pi = s + 1;
            star_match += 1;
            ti = star_match;
        } else {
            return false;
        }
    }

    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

/// Match a path against a pattern, falling back to basename matching when the
/// pattern does not contain a directory separator.
pub fn path_matches(pattern: &str, path: &str) -> bool {
    if pattern.contains('/') {
        return wildcard_match(pattern, path);
    }
    let basename = path.rsplit('/').next().unwrap_or(path);
    wildcard_match(pattern, basename) || wildcard_match(pattern, path)
}

/// Whether any pattern in the list matches the path.
pub fn matches_any(patterns: &[String], path: &str) -> bool {
    patterns.iter().any(|p| path_matches(p, path))
}

#[cfg(test)]
mod tests {
    use super::{matches_any, path_matches, wildcard_match};

    #[test]
    fn wildcard_star_and_question() {
        assert!(wildcard_match("*.rs", "main.rs"));
        assert!(wildcard_match("*.rs", "src/main.rs"));
        assert!(!wildcard_match("*.rs", "main.rsx"));
        assert!(wildcard_match("main.?s", "main.rs"));
        assert!(wildcard_match("a*c", "abc"));
        assert!(wildcard_match("*", "anything"));
    }

    #[test]
    fn basename_matching_for_extension_filters() {
        assert!(path_matches("*.rs", "deeply/nested/main.rs"));
        assert!(!path_matches("*.rs", "deeply/nested/main.md"));
        assert!(path_matches("src/**", "src/foo/bar.rs"));
        assert!(!path_matches("tests/**", "src/foo/bar.rs"));
    }

    #[test]
    fn matches_any_applies_whole_list() {
        let patterns = vec!["*.rs".to_string(), "*.md".to_string()];
        assert!(matches_any(&patterns, "README.md"));
        assert!(!matches_any(&patterns, "Cargo.toml"));
    }
}
