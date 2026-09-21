//! Determinism gate: Prevent HashMap/HashSet drift in the core pipeline.
//!
//! The `tokmd-types`, `tokmd-scan`, `tokmd-model`, and `tokmd-format` crates
//! are part of the core data/reporting pipeline, which has a strict determinism
//! contract. Using `HashMap` or `HashSet` in these crates often causes
//! snapshot tests to flap and breaks the invariant that outputs are byte-stable
//! across runs. They must use `BTreeMap` and `BTreeSet` instead.

use std::fs;
use std::path::{Path, PathBuf};

fn get_workspace_root() -> PathBuf {
    // When running tests, the current dir is usually the crate root (crates/tokmd).
    // The workspace root is one level up.
    let mut dir = std::env::current_dir().unwrap();
    if dir.ends_with("tokmd") && dir.parent().map(|p| p.ends_with("crates")).unwrap_or(false) {
        dir.pop();
        dir.pop();
    }
    dir
}

fn check_no_hash_maps(dir: &Path, crate_name: &str) {
    let mut violations = Vec::new();

    // Simple recursive file walk
    let mut dirs_to_visit = vec![dir.to_path_buf()];

    while let Some(current_dir) = dirs_to_visit.pop() {
        if let Ok(entries) = fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs_to_visit.push(path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    if let Ok(contents) = fs::read_to_string(&path) {
                        for (i, line) in contents.lines().enumerate() {
                            // Basic heuristic check for HashMap or HashSet. We skip lines with "//" or "///" or "/*" or "!" to avoid matching comments
                            let trimmed = line.trim();
                            if trimmed.starts_with("//")
                                || trimmed.starts_with("/*")
                                || trimmed.starts_with("!")
                                || trimmed.starts_with("*")
                            {
                                continue;
                            }

                            if trimmed.contains("HashMap") || trimmed.contains("HashSet") {
                                let display_path = path
                                    .strip_prefix(get_workspace_root())
                                    .unwrap_or(&path)
                                    .display();
                                violations.push(format!(
                                    "{}:{} - {}",
                                    display_path,
                                    i + 1,
                                    trimmed
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "HashMap or HashSet found in core pipeline crate '{}' (determinism violation)! Use BTreeMap/BTreeSet instead.\nViolations:\n{}",
        crate_name,
        violations.join("\n")
    );
}

#[test]
fn no_hashmap_or_hashset_in_core_pipeline() {
    let root = get_workspace_root();

    let crates_to_check = ["tokmd-types", "tokmd-scan", "tokmd-model", "tokmd-format"];

    for c in crates_to_check {
        let src_dir = root.join("crates").join(c).join("src");
        if src_dir.exists() {
            check_no_hash_maps(&src_dir, c);
        }
    }
}
