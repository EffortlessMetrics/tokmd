use crate::cli::TrimTargetArgs;
use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrimKind {
    IncrementalDir,
    PdbFile,
}

impl TrimKind {
    fn label(self) -> &'static str {
        match self {
            Self::IncrementalDir => "incremental",
            Self::PdbFile => "pdb",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrimEntry {
    kind: TrimKind,
    path: PathBuf,
    bytes: u64,
}

pub fn run(args: TrimTargetArgs) -> Result<()> {
    let metadata = MetadataCommand::new().no_deps().exec()?;
    let target_dir = metadata.target_directory.into_std_path_buf();
    let debug_dir = target_dir.join("debug");
    let entries = collect_trim_entries(&debug_dir, !args.keep_incremental, !args.keep_pdb)?;
    let reclaimable = entries.iter().map(|entry| entry.bytes).sum::<u64>();

    println!("trim-target: target dir {}", target_dir.display());
    println!("trim-target: scope {}", debug_dir.display());

    if entries.is_empty() {
        println!("trim-target: nothing to trim");
        return Ok(());
    }

    println!(
        "trim-target: {} candidate(s), {} reclaimable",
        entries.len(),
        human_bytes(reclaimable)
    );

    for entry in &entries {
        println!(
            "  - {:<11} {:>8}  {}",
            entry.kind.label(),
            human_bytes(entry.bytes),
            display_path(&debug_dir, &entry.path)
        );
    }

    if args.check {
        println!("trim-target: check mode, no files removed");
        return Ok(());
    }

    for entry in &entries {
        match entry.kind {
            TrimKind::IncrementalDir => {
                if entry.path.exists() {
                    fs::remove_dir_all(&entry.path)
                        .with_context(|| format!("failed to remove {}", entry.path.display()))?;
                }
            }
            TrimKind::PdbFile => {
                if entry.path.exists() {
                    fs::remove_file(&entry.path)
                        .with_context(|| format!("failed to remove {}", entry.path.display()))?;
                }
            }
        }
    }

    println!(
        "trim-target: removed {} candidate(s), reclaimed {}",
        entries.len(),
        human_bytes(reclaimable)
    );
    println!("trim-target: rerun `cargo trim-target --check` to confirm the target dir is lean");
    Ok(())
}

fn collect_trim_entries(
    target_dir: &Path,
    include_incremental: bool,
    include_pdb: bool,
) -> Result<Vec<TrimEntry>> {
    let mut entries = Vec::new();
    if target_dir.exists() {
        visit_trim_candidates(target_dir, include_incremental, include_pdb, &mut entries)?;
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(entries)
}

fn visit_trim_candidates(
    dir: &Path,
    include_incremental: bool,
    include_pdb: bool,
    entries: &mut Vec<TrimEntry>,
) -> Result<()> {
    for child in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let child = child?;
        let file_type = child.file_type()?;
        let path = child.path();

        if file_type.is_dir() {
            if include_incremental && child.file_name() == "incremental" {
                entries.push(TrimEntry {
                    kind: TrimKind::IncrementalDir,
                    bytes: dir_size(&path)?,
                    path,
                });
                continue;
            }

            visit_trim_candidates(&path, include_incremental, include_pdb, entries)?;
            continue;
        }

        if include_pdb
            && file_type.is_file()
            && path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("pdb"))
        {
            entries.push(TrimEntry {
                kind: TrimKind::PdbFile,
                bytes: child.metadata()?.len(),
                path,
            });
        }
    }

    Ok(())
}

fn dir_size(path: &Path) -> Result<u64> {
    let mut total = 0u64;
    for child in fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))? {
        let child = child?;
        let metadata = child.metadata()?;
        if metadata.is_dir() {
            total += dir_size(&child.path())?;
        } else {
            total += metadata.len();
        }
    }
    Ok(total)
}

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];

    let mut value = bytes as f64;
    let mut unit_idx = 0usize;
    while value >= 1024.0 && unit_idx + 1 < UNITS.len() {
        value /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{bytes} {}", UNITS[unit_idx])
    } else {
        format!("{value:.1} {}", UNITS[unit_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::{TrimKind, collect_trim_entries, human_bytes};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn collect_trim_entries_finds_incremental_dirs_and_pdb_files() {
        let temp = TestDir::new();
        write_file(temp.path().join("debug/deps/tokmd-test.pdb"), 128);
        write_file(temp.path().join("debug/deps/tokmd-test.exe"), 64);
        write_file(temp.path().join("debug/incremental/unit/hash.bin"), 256);

        let entries = collect_trim_entries(temp.path(), true, true).unwrap();

        assert_eq!(
            entries.len(),
            2,
            "should collect one pdb and one incremental dir"
        );
        assert!(entries.iter().any(|entry| {
            entry.kind == TrimKind::PdbFile
                && entry.path.ends_with(Path::new("debug/deps/tokmd-test.pdb"))
        }));
        assert!(entries.iter().any(|entry| {
            entry.kind == TrimKind::IncrementalDir
                && entry.path.ends_with(Path::new("debug/incremental"))
        }));
    }

    #[test]
    fn collect_trim_entries_respects_keep_flags() {
        let temp = TestDir::new();
        write_file(temp.path().join("debug/deps/tokmd-test.pdb"), 128);
        write_file(temp.path().join("debug/incremental/unit/hash.bin"), 256);

        let pdb_only = collect_trim_entries(temp.path(), false, true).unwrap();
        assert_eq!(pdb_only.len(), 1);
        assert_eq!(pdb_only[0].kind, TrimKind::PdbFile);

        let incremental_only = collect_trim_entries(temp.path(), true, false).unwrap();
        assert_eq!(incremental_only.len(), 1);
        assert_eq!(incremental_only[0].kind, TrimKind::IncrementalDir);
    }

    #[test]
    fn collect_trim_entries_skips_release_tree_when_debug_root_is_used() {
        let temp = TestDir::new();
        write_file(temp.path().join("debug/deps/debug-only.pdb"), 128);
        write_file(temp.path().join("release/deps/release-only.pdb"), 128);

        let entries = collect_trim_entries(&temp.path().join("debug"), false, true).unwrap();

        assert_eq!(entries.len(), 1);
        assert!(
            entries[0]
                .path
                .ends_with(Path::new("debug/deps/debug-only.pdb"))
        );
    }

    #[test]
    fn human_bytes_formats_large_values() {
        assert_eq!(human_bytes(999), "999 B");
        assert_eq!(human_bytes(1024), "1.0 KiB");
        assert_eq!(human_bytes(1024 * 1024), "1.0 MiB");
    }

    fn write_file(path: PathBuf, bytes: usize) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, vec![b'x'; bytes]).unwrap();
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            static NEXT_ID: AtomicU64 = AtomicU64::new(0);
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let unique = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "tokmd-trim-target-test-{}-{}-{}",
                std::process::id(),
                stamp,
                unique
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
