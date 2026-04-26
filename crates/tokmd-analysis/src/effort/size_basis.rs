use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{fs, io::BufRead, io::BufReader};

use tokmd_analysis_types::normalize_path;
use tokmd_analysis_types::{EffortSizeBasis, EffortTagSizeRow};
use tokmd_types::{ExportData, FileRow};

#[derive(Debug, Clone, Copy)]
enum FileKind {
    Core,
    Infra,
    Build,
    Docs,
    Tests,
    Generated,
    Vendored,
    Api,
    Ffi,
    Ui,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ClassKind {
    Unknown,
    Generated,
    Vendored,
}

impl ClassKind {
    #[allow(dead_code)]
    fn confidence_boost(self) -> f64 {
        match self {
            Self::Generated | Self::Vendored => 1.0,
            Self::Unknown => 0.0,
        }
    }
}

#[derive(Debug)]
struct GitAttrRule {
    kind: ClassKind,
    pattern: String,
    #[allow(dead_code)]
    source: String,
}

#[derive(Debug)]
pub struct SizeBasisResult {
    pub basis: EffortSizeBasis,
    pub source_confidence: f64,
}

fn has_host_root(root: &Path) -> bool {
    !root.as_os_str().is_empty()
}

pub fn build_size_basis(root: &Path, export: &ExportData) -> SizeBasisResult {
    let rules = if has_host_root(root) {
        load_gitattributes(root)
    } else {
        Vec::new()
    };
    let mut total_lines = 0usize;
    let mut generated_lines = 0usize;
    let mut vendored_lines = 0usize;
    let mut by_tag: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut unknown_lines = 0usize;

    for row in &export.rows {
        let normalized = normalize_path(&row.path, root);
        let (class, tag) = classify_row(root, &normalized, &rules, row);

        let code = row.code;
        total_lines = total_lines.saturating_add(code);

        let mut authored = code;
        let mut warning: Option<String> = None;

        let generated = matches!(class, ClassKind::Generated);
        let vendored = matches!(class, ClassKind::Vendored);
        if generated {
            generated_lines = generated_lines.saturating_add(code);
            authored = 0;
        }
        if vendored {
            vendored_lines = vendored_lines.saturating_add(code);
            authored = 0;
        }

        if class == ClassKind::Unknown {
            unknown_lines = unknown_lines.saturating_add(code);
            if code > 0 {
                warning = Some("heuristic-only classification used".to_string());
            }
        }

        if class == ClassKind::Unknown && matches!(tag, FileKind::Generated | FileKind::Vendored) {
            unknown_lines = unknown_lines.saturating_sub(code);
        }

        if generated {
            accumulate_tag(&mut by_tag, "generated", code, authored);
        } else if vendored {
            accumulate_tag(&mut by_tag, "vendored", code, authored);
        } else {
            let tag_name = tag_name(&tag);
            accumulate_tag(&mut by_tag, tag_name, code, authored);
        }

        let _ = warning;
    }

    let authored_lines = total_lines.saturating_sub(generated_lines + vendored_lines);
    let kloc_total = (total_lines as f64) / 1000.0;
    let kloc_authored = (authored_lines as f64) / 1000.0;

    let generated_pct = ratio(generated_lines, total_lines);
    let vendored_pct = ratio(vendored_lines, total_lines);

    let by_tag_rows = by_tag
        .into_iter()
        .map(|(tag, (lines, authored))| EffortTagSizeRow {
            tag,
            lines,
            authored_lines: authored,
            pct_of_total: ratio(lines, total_lines),
        })
        .collect::<Vec<_>>();

    let confidence_from_rules = if !rules.is_empty() { 0.75 } else { 0.55 };

    let confidence_heuristic = if total_lines == 0 {
        0.0
    } else {
        1.0 - (unknown_lines as f64) / (total_lines as f64)
    };
    let classification_confidence =
        ((0.4 * confidence_from_rules) + (0.6 * confidence_heuristic)).clamp(0.0, 1.0);

    let warnings = if unknown_lines > 0 {
        vec!["heuristic classification used for some files".to_string()]
    } else {
        Vec::new()
    };

    let basis = EffortSizeBasis {
        total_lines,
        authored_lines,
        generated_lines,
        vendored_lines,
        kloc_total,
        kloc_authored,
        generated_pct,
        vendored_pct,
        classification_confidence: if classification_confidence >= 0.75 {
            tokmd_analysis_types::EffortConfidenceLevel::High
        } else if classification_confidence >= 0.55 {
            tokmd_analysis_types::EffortConfidenceLevel::Medium
        } else {
            tokmd_analysis_types::EffortConfidenceLevel::Low
        },
        warnings,
        by_tag: by_tag_rows,
    };

    SizeBasisResult {
        basis,
        source_confidence: classification_confidence,
    }
}

fn classify_row(
    root: &Path,
    path: &str,
    rules: &[GitAttrRule],
    row: &FileRow,
) -> (ClassKind, FileKind) {
    let _lower = path.to_lowercase();

    for rule in rules {
        if matches_path_pattern(path, root, &rule.pattern) {
            return (
                rule.kind,
                match rule.kind {
                    ClassKind::Generated => FileKind::Generated,
                    ClassKind::Vendored => FileKind::Vendored,
                    ClassKind::Unknown => FileKind::Core,
                },
            );
        }
    }

    let kind = if looks_generated_dir(path) || has_generated_sentinel(root, path) {
        FileKind::Generated
    } else if looks_vendored_dir(path) {
        FileKind::Vendored
    } else if looks_test_path(path) {
        FileKind::Tests
    } else if looks_doc_path(path, row) {
        FileKind::Docs
    } else if looks_api_path(path) {
        FileKind::Api
    } else if looks_ffi_path(path) {
        FileKind::Ffi
    } else if looks_ui_path(path) {
        FileKind::Ui
    } else if looks_data_path(path) {
        FileKind::Data
    } else if looks_build_path(path) {
        FileKind::Build
    } else if looks_infra_path(row) {
        FileKind::Infra
    } else {
        FileKind::Core
    };

    let class = match kind {
        FileKind::Generated => ClassKind::Generated,
        FileKind::Vendored => ClassKind::Vendored,
        _ => ClassKind::Unknown,
    };

    (class, kind)
}

fn tag_name(kind: &FileKind) -> &str {
    match kind {
        FileKind::Core => "core",
        FileKind::Infra => "infra",
        FileKind::Build => "build",
        FileKind::Docs => "docs",
        FileKind::Tests => "tests",
        FileKind::Generated => "generated",
        FileKind::Vendored => "vendored",
        FileKind::Api => "api",
        FileKind::Ffi => "ffi",
        FileKind::Ui => "ui",
        FileKind::Data => "data",
    }
}

fn accumulate_tag(
    map: &mut BTreeMap<String, (usize, usize)>,
    tag: &str,
    lines: usize,
    authored: usize,
) {
    let entry = map.entry(tag.to_string()).or_insert((0, 0));
    entry.0 = entry.0.saturating_add(lines);
    entry.1 = entry.1.saturating_add(authored);
}

fn ratio(num: usize, den: usize) -> f64 {
    if den == 0 {
        0.0
    } else {
        (num as f64) / (den as f64)
    }
}

fn has_generated_sentinel(root: &Path, path: &str) -> bool {
    if !has_host_root(root) {
        return false;
    }

    let full = root.join(PathBuf::from(path));
    let file = match fs::File::open(&full) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let reader = BufReader::new(file);
    for line in reader.lines().take(40).flatten() {
        if is_generated_sentinel(&line) {
            return true;
        }
    }
    false
}

fn is_generated_sentinel(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("generated by")
        || lower.contains("do not edit")
        || lower.contains("auto-generated")
        || lower.contains("autogenerated")
}

fn looks_generated_dir(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("/generated/")
        || lower.contains("/dist/")
        || lower.contains("/build/")
        || lower.contains("/target/")
        || lower.ends_with(".min.js")
        || lower.ends_with(".map")
}

fn looks_vendored_dir(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("/vendor/")
        || lower.contains("/third_party/")
        || lower.contains("/node_modules/")
        || lower.contains("/deps/")
}

fn looks_test_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("/test/")
        || lower.contains("/tests/")
        || lower.contains("__tests__")
        || lower.contains("/spec/")
        || lower.contains("/specs/")
        || lower.ends_with("_test.rs")
}

fn looks_doc_path(path: &str, row: &FileRow) -> bool {
    let lower = path.to_lowercase();
    row.lang.to_lowercase() == "markdown"
        || lower.contains("/docs/")
        || lower.ends_with("readme.md")
}

fn looks_api_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("/api/") || lower.ends_with(".proto") || lower.ends_with(".openapi.json")
}

fn looks_ffi_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("/ffi/") || lower.contains("/bindings/") || lower.ends_with("_ffi.rs")
}

fn looks_ui_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("/ui/") || lower.contains("/web/") || lower.contains("/frontend/")
}

fn looks_data_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("/data/") || lower.contains("/resources/") || lower.contains("/assets/")
}

fn looks_build_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".yml")
        || lower.ends_with(".yaml")
        || lower.ends_with(".toml")
        || lower.contains("/ci/")
        || lower.contains("/.github/")
}

fn looks_infra_path(row: &FileRow) -> bool {
    tokmd_analysis_types::is_infra_lang(&row.lang)
}

fn load_gitattributes(root: &Path) -> Vec<GitAttrRule> {
    if !has_host_root(root) {
        return Vec::new();
    }

    let path = root.join(".gitattributes");
    let file = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) => return Vec::new(),
    };

    let mut rules = Vec::new();
    for raw in file.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace().map(str::trim).collect::<Vec<_>>();
        if parts.len() < 2 {
            continue;
        }

        let pattern = parts.remove(0).to_string();
        let flags = parts.join(" ");

        let kind = if flags.contains("linguist-generated") {
            ClassKind::Generated
        } else if flags.contains("linguist-vendored") {
            ClassKind::Vendored
        } else {
            ClassKind::Unknown
        };

        if !matches!(kind, ClassKind::Unknown) {
            rules.push(GitAttrRule {
                kind,
                pattern,
                source: raw.to_string(),
            });
        }
    }

    rules
}

fn matches_path_pattern(path: &str, root: &Path, pattern: &str) -> bool {
    let path_lower = path.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    if pattern_lower.is_empty() {
        return false;
    }

    if pattern_lower.starts_with("*") {
        let suffix = pattern_lower.trim_start_matches('*');
        if suffix.is_empty() {
            return false;
        }
        return path_lower.ends_with(suffix);
    }

    if pattern_lower.ends_with("/") {
        return path_lower.starts_with(&pattern_lower);
    }

    if path_lower.contains(&pattern_lower) {
        return true;
    }

    if !has_host_root(root) {
        return false;
    }

    let full = root.join(PathBuf::from(path));
    full.ends_with(&pattern_lower)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct RestoreCurrentDir(PathBuf);

    impl Drop for RestoreCurrentDir {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.0);
        }
    }

    fn with_current_dir<T>(path: &Path, f: impl FnOnce() -> T) -> T {
        let _lock = CWD_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("cwd lock");
        let original = env::current_dir().expect("current dir");
        env::set_current_dir(path).expect("set current dir");
        let _restore = RestoreCurrentDir(original);
        f()
    }

    #[test]
    fn size_basis_detects_generated_sentinels() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        let generated = src.join("gen.min.js");
        let mut f = File::create(&generated).unwrap();
        writeln!(f, "// Generated by build pipeline").unwrap();
        writeln!(f, "const x = 1;").unwrap();

        let export = ExportData {
            rows: vec![tokmd_types::FileRow {
                path: "src/gen.min.js".to_string(),
                module: "src".to_string(),
                lang: "JavaScript".to_string(),
                kind: tokmd_types::FileKind::Parent,
                code: 12,
                comments: 0,
                blanks: 0,
                lines: 12,
                bytes: 120,
                tokens: 30,
            }],
            module_roots: vec!["src".to_string()],
            module_depth: 1,
            children: tokmd_types::ChildIncludeMode::Separate,
        };

        let res = build_size_basis(dir.path(), &export);
        assert_eq!(res.basis.generated_lines, 12);
        assert_eq!(res.basis.authored_lines, 0);
    }

    #[test]
    fn size_basis_uses_gitattributes_over_heuristics() {
        let dir = tempdir().unwrap();
        let mut ga = File::create(dir.path().join(".gitattributes")).unwrap();
        writeln!(ga, "src/lib.rs linguist-generated").unwrap();

        let export = ExportData {
            rows: vec![tokmd_types::FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: tokmd_types::FileKind::Parent,
                code: 40,
                comments: 0,
                blanks: 0,
                lines: 40,
                bytes: 300,
                tokens: 20,
            }],
            module_roots: vec!["src".to_string()],
            module_depth: 1,
            children: tokmd_types::ChildIncludeMode::Separate,
        };

        let res = build_size_basis(dir.path(), &export);
        assert_eq!(res.basis.generated_lines, 40);
    }

    #[test]
    fn size_basis_with_empty_root_does_not_read_current_dir_metadata() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        let mut ga = File::create(dir.path().join(".gitattributes")).unwrap();
        writeln!(ga, "src/lib.rs linguist-generated").unwrap();
        let mut source = File::create(dir.path().join("src/lib.rs")).unwrap();
        writeln!(source, "// Generated by host workspace").unwrap();
        writeln!(source, "pub fn host_only() {{}}").unwrap();

        let export = ExportData {
            rows: vec![tokmd_types::FileRow {
                path: "src/lib.rs".to_string(),
                module: "src".to_string(),
                lang: "Rust".to_string(),
                kind: tokmd_types::FileKind::Parent,
                code: 40,
                comments: 0,
                blanks: 0,
                lines: 40,
                bytes: 300,
                tokens: 20,
            }],
            module_roots: vec!["src".to_string()],
            module_depth: 1,
            children: tokmd_types::ChildIncludeMode::Separate,
        };

        let res = with_current_dir(dir.path(), || build_size_basis(Path::new(""), &export));

        assert_eq!(res.basis.generated_lines, 0);
        assert_eq!(res.basis.vendored_lines, 0);
        assert_eq!(res.basis.authored_lines, 40);
    }
}
