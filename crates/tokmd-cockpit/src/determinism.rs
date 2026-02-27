//! Shared BLAKE3 hashing helpers for determinism verification.
//!
//! Used by both the `baseline` command (to capture source hashes) and the
//! `cockpit` command (to verify them). Both paths use the same incremental
//! BLAKE3 protocol so that identical source trees produce identical hashes.

use std::path::Path;

use anyhow::{Context, Result};

/// Hash a set of files given their relative paths (from export rows).
///
/// Paths are sorted before hashing for deterministic output.
/// Each file contributes `(normalized_path, file_length_le_bytes, file_bytes)`
/// to an incremental BLAKE3 hasher.
pub fn hash_files_from_paths(root: &Path, paths: &[&str]) -> Result<String> {
    let mut sorted: Vec<&str> = paths.to_vec();
    sorted.sort();
    sorted.dedup();

    let mut hasher = blake3::Hasher::new();

    for rel_path in &sorted {
        let normalized = normalize(rel_path);
        let full = root.join(rel_path);

        // Match walk-based hashing: never include generated/metadata directories,
        // even if they appear in the exported path set.
        if normalized.starts_with(".tokmd/")
            || normalized.starts_with("target/")
            || normalized.starts_with(".git/")
        {
            continue;
        }

        // Skip files that no longer exist (e.g., deleted between scan and baseline)
        match feed_file_streaming(&mut hasher, &normalized, &full) {
            Ok(()) => {}
            Err(_) => continue,
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Hash all tracked files under `root` by walking the directory tree.
///
/// Uses the `ignore` crate to respect `.gitignore` rules, then sorts
/// the discovered paths and hashes them with the same protocol as
/// [`hash_files_from_paths`].
pub fn hash_files_from_walk(root: &Path, exclude_rel: &[&str]) -> Result<String> {
    let mut paths: Vec<String> = Vec::new();

    let walker = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(true)
        .build();

    for entry in walker {
        let entry = entry.context("failed to walk directory entry")?;
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        if let Ok(rel) = entry.path().strip_prefix(root) {
            let normalized = normalize(&rel.to_string_lossy());

            // Hard-skip generated directories + explicit exclusions
            if normalized.starts_with(".tokmd/")
                || normalized.starts_with("target/")
                || normalized.starts_with(".git/")
                || exclude_rel.iter().any(|ex| normalized == *ex)
            {
                continue;
            }

            paths.push(normalized);
        }
    }

    paths.sort();
    paths.dedup();

    let mut hasher = blake3::Hasher::new();

    for rel_path in &paths {
        let full = root.join(rel_path);
        match feed_file_streaming(&mut hasher, rel_path, &full) {
            Ok(()) => {}
            Err(_) => continue,
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Hash the `Cargo.lock` file at `root`, if present.
///
/// Returns `None` if no `Cargo.lock` exists.
pub fn hash_cargo_lock(root: &Path) -> Result<Option<String>> {
    let lock_path = root.join("Cargo.lock");
    if !lock_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read(&lock_path)
        .with_context(|| format!("failed to read {}", lock_path.display()))?;

    let hash = blake3::hash(&content);
    Ok(Some(hash.to_hex().to_string()))
}

/// Feed a single file into the incremental hasher using streaming I/O (O(1) memory).
///
/// Protocol: `hasher.update(path_bytes); hasher.update(len_le_bytes); hasher.update(content)`.
fn feed_file_streaming(
    hasher: &mut blake3::Hasher,
    normalized_path: &str,
    full_path: &Path,
) -> std::io::Result<()> {
    let metadata = std::fs::metadata(full_path)?;
    let len = metadata.len();
    hasher.update(normalized_path.as_bytes());
    hasher.update(&len.to_le_bytes());
    let file = std::fs::File::open(full_path)?;
    let mut reader = std::io::BufReader::new(file);
    std::io::copy(&mut reader, hasher)?;
    Ok(())
}

/// Normalize a relative path to use forward slashes.
fn normalize(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_hash_files_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("b.rs"), "fn test() {}").unwrap();

        let paths = vec!["a.rs", "b.rs"];
        let h1 = hash_files_from_paths(dir.path(), &paths).unwrap();
        let h2 = hash_files_from_paths(dir.path(), &paths).unwrap();

        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // BLAKE3 hex digest
    }

    #[test]
    fn test_hash_files_order_independent() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("b.rs"), "fn test() {}").unwrap();

        let h1 = hash_files_from_paths(dir.path(), &["a.rs", "b.rs"]).unwrap();
        let h2 = hash_files_from_paths(dir.path(), &["b.rs", "a.rs"]).unwrap();

        assert_eq!(h1, h2, "hash should be order-independent");
    }

    #[test]
    fn test_hash_files_changes_on_modification() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();

        let h1 = hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();

        fs::write(dir.path().join("a.rs"), "fn main() { panic!(); }").unwrap();

        let h2 = hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();

        assert_ne!(h1, h2, "hash should change when file content changes");
    }

    #[test]
    fn test_hash_cargo_lock_present() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.lock"),
            "[[package]]\nname = \"test\"",
        )
        .unwrap();

        let result = hash_cargo_lock(dir.path()).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 64);
    }

    #[test]
    fn test_hash_cargo_lock_absent() {
        let dir = tempfile::tempdir().unwrap();

        let result = hash_cargo_lock(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_hash_files_from_walk_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("b.rs"), "fn test() {}").unwrap();

        let h1 = hash_files_from_walk(dir.path(), &[]).unwrap();
        let h2 = hash_files_from_walk(dir.path(), &[]).unwrap();

        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_walk_and_paths_produce_same_hash() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("b.rs"), "fn test() {}").unwrap();
        // Create .git marker so ignore crate works properly
        fs::create_dir_all(dir.path().join(".git")).unwrap();

        let from_paths = hash_files_from_paths(dir.path(), &["a.rs", "b.rs"]).unwrap();
        let from_walk = hash_files_from_walk(dir.path(), &[]).unwrap();

        assert_eq!(
            from_paths, from_walk,
            "walk and explicit paths should produce same hash for same files"
        );
    }

    #[test]
    fn test_walk_excludes_specified_paths() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("b.rs"), "fn test() {}").unwrap();
        // Create .git marker so ignore crate works properly
        fs::create_dir_all(dir.path().join(".git")).unwrap();

        // Walk excluding b.rs should match paths-only hash of just a.rs
        let walk_excluded = hash_files_from_walk(dir.path(), &["b.rs"]).unwrap();
        let paths_only = hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();

        assert_eq!(
            walk_excluded, paths_only,
            "excluding b.rs from walk should match paths-only a.rs"
        );
    }

    #[test]
    fn test_walk_excludes_tokmd_directory() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();
        // Create .git marker so ignore crate works properly
        fs::create_dir_all(dir.path().join(".git")).unwrap();
        // Create .tokmd directory with a baseline file -- should be auto-excluded
        fs::create_dir_all(dir.path().join(".tokmd")).unwrap();
        fs::write(dir.path().join(".tokmd/baseline.json"), "{}").unwrap();

        let with_tokmd = hash_files_from_walk(dir.path(), &[]).unwrap();
        let paths_only = hash_files_from_paths(dir.path(), &["a.rs"]).unwrap();

        assert_eq!(
            with_tokmd, paths_only,
            ".tokmd/ directory should be auto-excluded from walk hash"
        );
    }
}
