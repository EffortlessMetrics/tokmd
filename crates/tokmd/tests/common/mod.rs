//! Shared test utilities for tokmd integration tests.
//!
//! This module provides hermetic test fixtures that work correctly across all
//! environments, including cargo-mutants which copies the crate to a temp
//! directory without the parent `.git/` marker.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static FIXTURE_ROOT: OnceLock<PathBuf> = OnceLock::new();

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Returns path to a hermetic copy of test fixtures with `.git/` marker.
///
/// The fixture is initialized once per test process using `OnceLock`.
/// This ensures that:
/// 1. The `ignore` crate honors `.gitignore` rules (requires `.git/` marker)
/// 2. Tests work in cargo-mutants environment (no parent `.git/`)
/// 3. Fixture is only copied once for efficiency
pub fn fixture_root() -> &'static Path {
    FIXTURE_ROOT
        .get_or_init(|| {
            let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("data");

            let dst = std::env::temp_dir().join(format!(
                "tokmd-fixtures-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));

            let _ = std::fs::remove_dir_all(&dst);
            copy_dir_recursive(&src, &dst).expect("copy test fixtures");
            std::fs::create_dir_all(dst.join(".git")).expect("create .git marker");

            dst
        })
        .as_path()
}
