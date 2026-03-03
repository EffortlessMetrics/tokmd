//! Deep invariant tests for topic-cloud extraction.
//!
//! Tests TF-IDF scoring, tokenisation, TOP_K truncation,
//! determinism, stopword filtering, and edge cases.

use tokmd_analysis_topics::build_topic_clouds;
use tokmd_analysis_types::TopicTerm;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn row(path: &str, module: &str, tokens: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 10,
        comments: 0,
        blanks: 0,
        lines: 10,
        bytes: 100,
        tokens,
    }
}

fn export(rows: Vec<FileRow>, roots: &[&str]) -> ExportData {
    ExportData {
        rows,
        module_roots: roots.iter().map(|r| r.to_string()).collect(),
        module_depth: 1,
        children: ChildIncludeMode::Separate,
    }
}

fn overall_terms(data: &ExportData) -> Vec<String> {
    build_topic_clouds(data)
        .overall
        .iter()
        .map(|e| e.term.clone())
        .collect()
}

// ── 1. Empty export produces empty clouds ───────────────────────

#[test]
fn empty_export_yields_empty_clouds() {
    let data = export(vec![], &[]);
    let clouds = build_topic_clouds(&data);
    assert!(clouds.overall.is_empty());
    assert!(clouds.per_module.is_empty());
}

// ── 2. Child rows are excluded from topic extraction ────────────

#[test]
fn child_rows_excluded() {
    let child = FileRow {
        path: "m/unique_child_term.rs".to_string(),
        module: "m".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Child,
        code: 10,
        comments: 0,
        blanks: 0,
        lines: 10,
        bytes: 100,
        tokens: 50,
    };
    let data = export(vec![child], &[]);
    let clouds = build_topic_clouds(&data);
    assert!(clouds.overall.is_empty(), "child rows should be ignored");
}

// ── 3. Numeric path segments are valid terms ────────────────────

#[test]
fn numeric_path_segments_produce_valid_terms() {
    let data = export(vec![row("v2/api/handler.rs", "v2/api", 50)], &[]);
    let terms = overall_terms(&data);
    assert!(
        terms.contains(&"v2".to_string()),
        "numeric segments should be valid: {terms:?}"
    );
}

// ── 4. Single file: overall == per_module ───────────────────────

#[test]
fn single_file_overall_equals_per_module() {
    let data = export(vec![row("m/widget.rs", "m", 50)], &[]);
    let clouds = build_topic_clouds(&data);
    let overall = &clouds.overall;
    let per_mod = clouds.per_module.get("m").expect("module 'm'");

    assert_eq!(overall.len(), per_mod.len());
    for (o, p) in overall.iter().zip(per_mod.iter()) {
        assert_eq!(o.term, p.term);
        assert!((o.score - p.score).abs() < f64::EPSILON);
    }
}

// ── 5. DF counts files, not token frequency ─────────────────────

#[test]
fn df_counts_files_not_token_frequency() {
    let data = export(
        vec![row("m/widget_a.rs", "m", 50), row("m/widget_b.rs", "m", 50)],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let widget = clouds
        .overall
        .iter()
        .find(|t| t.term == "widget")
        .expect("widget should exist");
    assert_eq!(widget.df, 2, "df should count per-file occurrences");
}

// ── 6. Rare term scores higher than ubiquitous term via IDF ─────

#[test]
fn rare_term_scores_higher_via_idf() {
    let mut rows = Vec::new();
    for i in 0..5 {
        rows.push(row(&format!("mod_{i}/common.rs"), &format!("mod_{i}"), 50));
    }
    rows.push(row("mod_0/rare.rs", "mod_0", 50));
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);

    let m0 = clouds.per_module.get("mod_0").expect("mod_0");
    let common = m0.iter().find(|t| t.term == "common");
    let rare = m0.iter().find(|t| t.term == "rare");

    if let (Some(c), Some(r)) = (common, rare) {
        assert!(
            r.score >= c.score,
            "rare ({}) should score >= common ({})",
            r.score,
            c.score
        );
    }
}

// ── 7. TOP_K preserves highest-scoring terms ────────────────────

#[test]
fn top_k_preserves_highest_scoring_terms() {
    let rows: Vec<FileRow> = (0..20)
        .map(|i| row(&format!("m/term{i}.rs"), "m", (i + 1) * 100))
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);

    let m_terms = clouds.per_module.get("m").expect("module 'm'");
    assert!(m_terms.len() <= 8, "TOP_K should be 8");
    assert_eq!(m_terms[0].term, "term19", "highest-weight term first");
}

// ── 8. All terms are lowercase ──────────────────────────────────

#[test]
fn all_terms_are_lowercase() {
    let data = export(vec![row("MOD/CamelCase_Widget.rs", "MOD", 50)], &[]);
    let terms = overall_terms(&data);
    for term in &terms {
        assert_eq!(*term, term.to_lowercase(), "terms must be lowercase");
    }
}

// ── 9. Per-module keys are BTreeMap-sorted ──────────────────────

#[test]
fn per_module_keys_are_lexicographically_sorted() {
    let data = export(
        vec![
            row("zebra/file.rs", "zebra", 50),
            row("alpha/file.rs", "alpha", 50),
            row("middle/file.rs", "middle", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let keys: Vec<&String> = clouds.per_module.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "per_module keys should be sorted");
}

// ── 10. Consecutive separators produce no empty terms ───────────

#[test]
fn consecutive_separators_no_empty_terms() {
    let data = export(vec![row("a//b__c--d..e.rs", "a", 50)], &[]);
    let terms = overall_terms(&data);
    for term in &terms {
        assert!(!term.is_empty(), "empty term found");
    }
}

// ── 11. Backslash paths normalized to forward slash ─────────────

#[test]
fn backslash_paths_equivalent_to_forward_slash() {
    let data_fwd = export(vec![row("app/auth/handler.rs", "app/auth", 50)], &["app"]);
    let data_bck = export(vec![row(r"app\auth\handler.rs", "app/auth", 50)], &["app"]);

    let terms_fwd = overall_terms(&data_fwd);
    let terms_bck = overall_terms(&data_bck);
    assert_eq!(terms_fwd, terms_bck, "slash normalization must match");
}

// ── 12. TF accumulates across files in same module ──────────────

#[test]
fn tf_accumulates_across_files_in_same_module() {
    let data = export(
        vec![
            row("m/widget_a.rs", "m", 100),
            row("m/widget_b.rs", "m", 200),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let m_terms = clouds.per_module.get("m").expect("module 'm'");
    let widget = m_terms.iter().find(|t| t.term == "widget").unwrap();
    assert_eq!(widget.tf, 300, "tf should sum weights: 100 + 200");
}

// ── 13. Determinism with many modules ───────────────────────────

#[test]
fn determinism_with_many_modules() {
    let rows: Vec<FileRow> = (0..50)
        .map(|i| row(&format!("mod_{i}/feature.rs"), &format!("mod_{i}"), 50))
        .collect();
    let data = export(rows, &[]);

    let results: Vec<Vec<TopicTerm>> = (0..3).map(|_| build_topic_clouds(&data).overall).collect();

    for i in 1..3 {
        assert_eq!(results[0].len(), results[i].len());
        for (a, b) in results[0].iter().zip(results[i].iter()) {
            assert_eq!(a.term, b.term);
            assert!((a.score - b.score).abs() < f64::EPSILON);
        }
    }
}

// ── 14. All documented extensions are stopwords ─────────────────

#[test]
fn all_documented_extensions_are_stopwords() {
    let known_extensions = [
        "rs", "js", "ts", "tsx", "jsx", "py", "go", "java", "kt", "kts", "rb", "php", "c", "cc",
        "cpp", "h", "hpp", "cs", "swift", "m", "mm", "scala", "sql", "toml", "yaml", "yml", "json",
        "md", "markdown", "txt", "lock", "cfg", "ini", "env", "nix", "zig", "dart",
    ];
    for ext in known_extensions {
        let data = export(vec![row(&format!("module/{ext}.rs"), "module", 50)], &[]);
        let terms = overall_terms(&data);
        assert!(
            !terms.contains(&ext.to_string()),
            "extension '{ext}' should be a stopword: {terms:?}"
        );
    }
}

// ── 15. Base stopwords filter common directories ────────────────

#[test]
fn base_stopwords_filter_common_directories() {
    let base_stops = [
        "src",
        "lib",
        "mod",
        "index",
        "test",
        "tests",
        "impl",
        "main",
        "bin",
        "pkg",
        "package",
        "target",
        "build",
        "dist",
        "out",
        "gen",
        "generated",
    ];
    for stop in base_stops {
        let data = export(vec![row(&format!("{stop}/feature.rs"), stop, 50)], &[]);
        let terms = overall_terms(&data);
        assert!(
            !terms.contains(&stop.to_string()),
            "'{stop}' should be filtered: {terms:?}"
        );
    }
}
