use std::collections::BTreeMap;
use std::path::Path;

use super::classify::{ClassKind, FileKind, classify_row, load_gitattributes, tag_name};
use tokmd_analysis_types::normalize_path;
use tokmd_analysis_types::{EffortSizeBasis, EffortTagSizeRow};
use tokmd_types::ExportData;

#[derive(Debug)]
pub struct SizeBasisResult {
    pub basis: EffortSizeBasis,
    pub source_confidence: f64,
}

pub fn build_size_basis(root: &Path, export: &ExportData) -> SizeBasisResult {
    let rules = load_gitattributes(root);
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
