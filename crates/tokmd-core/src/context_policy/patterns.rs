//! Path pattern tables and path-focused predicates for context policy.

use tokmd_scan::normalize_slashes as normalize_path;

pub(crate) const LOCKFILES: &[&str] = &[
    "Cargo.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "poetry.lock",
    "Pipfile.lock",
    "go.sum",
    "composer.lock",
    "Gemfile.lock",
];

pub(crate) const GENERATED_PATTERNS: &[&str] = &[
    "node-types.json",
    "grammar.json",
    ".generated.",
    ".pb.go",
    ".pb.rs",
    "_pb2.py",
    ".g.dart",
    ".freezed.dart",
];

pub(crate) const VENDORED_DIRS: &[&str] =
    &["vendor/", "third_party/", "third-party/", "node_modules/"];
pub(crate) const FIXTURE_DIRS: &[&str] = &[
    "fixtures/",
    "testdata/",
    "test_data/",
    "__snapshots__/",
    "golden/",
];

const SMART_EXCLUDE_SUFFIXES: &[(&str, &str)] = &[
    (".min.js", "minified"),
    (".min.css", "minified"),
    (".js.map", "sourcemap"),
    (".css.map", "sourcemap"),
];

const SPINE_PATTERNS: &[&str] = &[
    "README.md",
    "README",
    "README.rst",
    "README.txt",
    "ROADMAP.md",
    "docs/ROADMAP.md",
    "CONTRIBUTING.md",
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "go.mod",
    "docs/architecture.md",
    "docs/design.md",
    "tokmd.toml",
    "cockpit.toml",
];

/// Returns the smart-exclude reason for a path, if any.
///
/// Reasons:
/// - `lockfile`
/// - `minified`
/// - `sourcemap`
#[must_use]
pub fn smart_exclude_reason(path: &str) -> Option<&'static str> {
    let basename = basename(path);

    if LOCKFILES.contains(&basename) {
        return Some("lockfile");
    }

    for &(suffix, reason) in SMART_EXCLUDE_SUFFIXES {
        if basename.ends_with(suffix) {
            return Some(reason);
        }
    }

    None
}

/// Returns `true` when a path matches a "spine" file that should be prioritized.
#[must_use]
pub fn is_spine_file(path: &str) -> bool {
    let normalized = normalize_path(path);
    let basename = basename(&normalized);

    for &pattern in SPINE_PATTERNS {
        if pattern.contains('/') {
            if normalized == pattern || normalized.ends_with(&format!("/{pattern}")) {
                return true;
            }
        } else if basename == pattern {
            return true;
        }
    }

    false
}

#[must_use]
pub(crate) fn basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}
