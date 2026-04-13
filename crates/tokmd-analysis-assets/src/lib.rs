use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_analysis_types::{
    AssetCategoryRow, AssetFileRow, AssetReport, DependencyReport, LockfileReport,
};

const ASSET_TOP_N: usize = 10;

/// Build aggregate asset inventory for files produced by a walk.
pub fn build_assets_report(root: &Path, files: &[PathBuf]) -> Result<AssetReport> {
    let mut categories: BTreeMap<String, (usize, u64, BTreeSet<String>)> = BTreeMap::new();
    let mut top_files: Vec<AssetFileRow> = Vec::new();
    let mut total_files = 0usize;
    let mut total_bytes = 0u64;

    for rel in files {
        let ext = rel
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext.is_empty() {
            continue;
        }
        let category = match asset_category(&ext) {
            Some(cat) => cat,
            None => continue,
        };
        let bytes = tokmd_walk::file_size(root, rel).unwrap_or(0);
        total_files += 1;
        total_bytes += bytes;

        let entry = categories
            .entry(category.to_string())
            .or_insert((0, 0, BTreeSet::new()));
        entry.0 += 1;
        entry.1 += bytes;
        entry.2.insert(ext.clone());

        top_files.push(AssetFileRow {
            path: rel.to_string_lossy().replace('\\', "/"),
            bytes,
            category: category.to_string(),
            extension: ext,
        });
    }

    let mut category_rows: Vec<AssetCategoryRow> = categories
        .into_iter()
        .map(|(category, (files, bytes, exts))| AssetCategoryRow {
            category,
            files,
            bytes,
            extensions: exts.into_iter().collect(),
        })
        .collect();

    category_rows.sort_by(|a, b| {
        b.bytes
            .cmp(&a.bytes)
            .then_with(|| a.category.cmp(&b.category))
    });
    top_files.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(&b.path)));
    top_files.truncate(ASSET_TOP_N);

    Ok(AssetReport {
        total_files,
        total_bytes,
        categories: category_rows,
        top_files,
    })
}

fn asset_category(ext: &str) -> Option<&'static str> {
    match ext {
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "bmp" | "tiff" | "ico" => Some("image"),
        "mp4" | "mov" | "avi" | "mkv" | "webm" | "mpeg" | "mpg" => Some("video"),
        "mp3" | "wav" | "flac" | "ogg" | "aac" | "m4a" => Some("audio"),
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => Some("archive"),
        "exe" | "dll" | "so" | "dylib" | "bin" | "class" | "jar" => Some("binary"),
        "ttf" | "otf" | "woff" | "woff2" => Some("font"),
        _ => None,
    }
}

/// Build dependency lockfile summary from detected lockfile paths.
pub fn build_dependency_report(root: &Path, files: &[PathBuf]) -> Result<DependencyReport> {
    let mut lockfiles: Vec<LockfileReport> = Vec::new();

    for rel in files {
        let name = rel.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let path = root.join(rel);
        let content = std::fs::read_to_string(&path);

        let (kind, count) = match name {
            "Cargo.lock" => content
                .as_deref()
                .map(|c| ("cargo", count_cargo_lock(c)))
                .unwrap_or(("cargo", 0)),
            "package-lock.json" => content
                .as_deref()
                .map(|c| ("npm", count_package_lock(c)))
                .unwrap_or(("npm", 0)),
            "pnpm-lock.yaml" => content
                .as_deref()
                .map(|c| ("pnpm", count_pnpm_lock(c)))
                .unwrap_or(("pnpm", 0)),
            "yarn.lock" => content
                .as_deref()
                .map(|c| ("yarn", count_yarn_lock(c)))
                .unwrap_or(("yarn", 0)),
            "go.sum" => content
                .as_deref()
                .map(|c| ("go", count_go_sum(c)))
                .unwrap_or(("go", 0)),
            "Gemfile.lock" => content
                .as_deref()
                .map(|c| ("bundler", count_gemfile_lock(c)))
                .unwrap_or(("bundler", 0)),
            _ => continue,
        };

        lockfiles.push(LockfileReport {
            path: rel.to_string_lossy().replace('\\', "/"),
            kind: kind.to_string(),
            dependencies: count,
        });
    }

    let total = lockfiles.iter().map(|l| l.dependencies).sum();
    Ok(DependencyReport { total, lockfiles })
}

fn count_cargo_lock(content: &str) -> usize {
    content.matches("[[package]]").count()
}

fn count_package_lock(content: &str) -> usize {
    let parsed: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return 0,
    };
    if let Some(packages) = parsed.get("packages").and_then(|v| v.as_object()) {
        let mut count = packages.len();
        if packages.contains_key("") {
            count = count.saturating_sub(1);
        }
        return count;
    }
    parsed
        .get("dependencies")
        .and_then(|v| v.as_object())
        .map(|o| o.len())
        .unwrap_or(0)
}

fn count_pnpm_lock(content: &str) -> usize {
    content
        .lines()
        .filter(|line| line.trim_start().starts_with("/") && line.contains(':'))
        .count()
}

fn count_yarn_lock(content: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            let line = line.trim();
            !line.is_empty() && !line.starts_with('#') && !line.starts_with("version")
        })
        .filter(|line| !line.starts_with("  ") && line.ends_with(':'))
        .count()
}

fn count_go_sum(content: &str) -> usize {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for line in content.lines() {
        let mut parts = line.split_whitespace();
        let module = match parts.next() {
            Some(v) => v,
            None => continue,
        };
        let version = match parts.next() {
            Some(v) => v,
            None => continue,
        };
        if version.ends_with("/go.mod") {
            continue;
        }
        seen.insert(format!("{}@{}", module, version));
    }
    seen.len()
}

fn count_gemfile_lock(content: &str) -> usize {
    let mut count = 0usize;
    let mut in_specs = false;
    for line in content.lines() {
        if line.trim() == "specs:" {
            in_specs = true;
            continue;
        }
        if in_specs {
            if line.trim().is_empty() || !line.starts_with("    ") {
                if !line.starts_with("    ") {
                    in_specs = false;
                }
                continue;
            }
            if line.contains('(') {
                count += 1;
            }
        }
    }
    count
}
