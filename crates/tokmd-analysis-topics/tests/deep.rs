//! Deep invariant tests for topic-cloud extraction.
//!
//! Focuses on TF-IDF scoring semantics, tokenisation edge cases,
//! TOP_K truncation, determinism, and stopword filtering.

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

// ── 1. Numeric path segments produce valid terms ────────────────

#[test]
fn numeric_path_segments_produce_valid_terms() {
    let data = export(vec![row("v2/api/handler.rs", "v2/api", 50)], &[]);
    let terms = overall_terms(&data);
    // "v2" is split into "v2" (single token, no further splitting)
    assert!(
        terms.contains(&"v2".to_string()),
        "numeric path segments should be valid terms: {terms:?}"
    );
}

// ── 2. Single file: overall == per_module ───────────────────────

#[test]
fn single_file_overall_equals_per_module() {
    let data = export(vec![row("m/widget.rs", "m", 50)], &[]);
    let clouds = build_topic_clouds(&data);
    let overall = &clouds.overall;
    let per_mod = clouds.per_module.get("m").expect("module 'm' should exist");

    assert_eq!(
        overall.len(),
        per_mod.len(),
        "single-file: overall and per_module should have same length"
    );
    for (o, p) in overall.iter().zip(per_mod.iter()) {
        assert_eq!(o.term, p.term);
        assert!(
            (o.score - p.score).abs() < f64::EPSILON,
            "scores should match"
        );
    }
}

// ── 3. DF counts files, not token frequency ─────────────────────

#[test]
fn df_counts_files_not_token_frequency() {
    // Two files in same module, both containing "widget" in their path
    let data = export(
        vec![
            row("m/widget_a.rs", "m", 50),
            row("m/widget_b.rs", "m", 50),
        ],
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

// ── 4. Term in many modules gets lower IDF boost ────────────────

#[test]
fn ubiquitous_term_has_lower_idf_than_rare_term() {
    // "common" appears in all 5 modules; "rare" appears in only 1.
    // Score = tf * idf. "common" accumulates tf from 5 files, so its total
    // tf is higher. To isolate the IDF effect, compare per-module scores
    // where each module has a single occurrence.
    let mut rows = Vec::new();
    for i in 0..5 {
        rows.push(row(&format!("mod_{i}/common.rs"), &format!("mod_{i}"), 50));
    }
    rows.push(row("mod_0/rare.rs", "mod_0", 50));
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);

    // In mod_0, both "common" and "rare" have tf=50 (same weight).
    // "rare" has df=1 module vs "common" df=5 modules → rare has higher IDF.
    let m0 = clouds.per_module.get("mod_0").expect("mod_0 should exist");
    let common = m0.iter().find(|t| t.term == "common");
    let rare = m0.iter().find(|t| t.term == "rare");

    if let (Some(c), Some(r)) = (common, rare) {
        assert!(
            r.score >= c.score,
            "rare ({}) should score >= common ({}) in same module due to IDF",
            r.score,
            c.score
        );
    }
}

// ── 5. TOP_K preserves highest-scoring terms ────────────────────

#[test]
fn top_k_preserves_highest_scoring_terms() {
    // Create 20 terms with varying weights; only top 8 should survive
    let rows: Vec<FileRow> = (0..20)
        .map(|i| row(&format!("m/term{i}.rs"), "m", (i + 1) * 100))
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);

    let m_terms = clouds.per_module.get("m").expect("module 'm'");
    assert!(m_terms.len() <= 8);

    // The highest-scoring term should be term19 (weight=2000)
    // since all terms have the same IDF (single module)
    assert_eq!(
        m_terms[0].term, "term19",
        "highest-weight term should rank first"
    );
}

// ── 6. Consecutive separators: no empty terms ───────────────────

#[test]
fn consecutive_separators_no_empty_terms() {
    let data = export(
        vec![row("a__b--c..d/file.rs", "a__b--c..d", 50)],
        &[],
    );
    let terms = overall_terms(&data);
    for term in &terms {
        assert!(!term.is_empty(), "no empty terms should be produced");
    }
}

// ── 7. Mixed case normalizes to lowercase ───────────────────────

#[test]
fn mixed_case_normalizes_to_lowercase() {
    let data = export(vec![row("MyModule/MyFile.rs", "MyModule", 50)], &[]);
    let terms = overall_terms(&data);
    for term in &terms {
        assert_eq!(
            *term,
            term.to_lowercase(),
            "all terms should be lowercase"
        );
    }
}

// ── 8. Per-module keys are BTreeMap-sorted ──────────────────────

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

// ── 9. Weight overflow: u32::MAX tokens still works ─────────────

#[test]
fn u32_max_tokens_does_not_overflow() {
    let data = export(vec![row("m/overflow.rs", "m", u32::MAX as usize)], &[]);
    let clouds = build_topic_clouds(&data);
    let overflow = clouds.overall.iter().find(|t| t.term == "overflow");
    assert!(overflow.is_some(), "term should exist with MAX weight");
    assert!(overflow.unwrap().tf > 0, "tf should be positive");
}

// ── 10. Module depth doesn't affect extraction ──────────────────

#[test]
fn deeply_nested_path_extracts_all_non_stopword_segments() {
    // Note: "c" is a stopword (file extension), so skip it in expectations
    let data = export(vec![row("a/b/deep/d/e/f/feature_name.rs", "a/b", 50)], &[]);
    let terms = overall_terms(&data);
    for expected in ["a", "b", "deep", "d", "e", "f", "feature", "name"] {
        assert!(
            terms.contains(&expected.to_string()),
            "missing '{expected}' in {terms:?}"
        );
    }
}

// ── 11. Backslash and forward slash paths produce same terms ────

#[test]
fn backslash_and_forward_slash_paths_equivalent() {
    let data_fwd = export(vec![row("app/auth/handler.rs", "app/auth", 50)], &["app"]);
    let data_bck = export(vec![row(r"app\auth\handler.rs", "app/auth", 50)], &["app"]);

    let terms_fwd = overall_terms(&data_fwd);
    let terms_bck = overall_terms(&data_bck);

    assert_eq!(
        terms_fwd, terms_bck,
        "backslash and forward slash paths should produce identical terms"
    );
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
    // tf should be sum of weights: 100 + 200 = 300
    assert_eq!(widget.tf, 300, "tf should accumulate weights across files");
}

// ── 13. Determinism with many modules ───────────────────────────

#[test]
fn determinism_with_many_modules() {
    let rows: Vec<FileRow> = (0..50)
        .map(|i| row(&format!("mod_{i}/feature.rs"), &format!("mod_{i}"), 50))
        .collect();
    let data = export(rows, &[]);

    let results: Vec<Vec<TopicTerm>> =
        (0..3).map(|_| build_topic_clouds(&data).overall).collect();

    for i in 1..3 {
        assert_eq!(
            results[0].len(),
            results[i].len(),
            "run count mismatch"
        );
        for (a, b) in results[0].iter().zip(results[i].iter()) {
            assert_eq!(a.term, b.term);
            assert!(
                (a.score - b.score).abs() < f64::EPSILON,
                "score mismatch for {}",
                a.term
            );
        }
    }
}

// ── 14. All documented extensions are stopwords ─────────────────

#[test]
fn all_documented_extensions_are_stopwords() {
    let known_extensions = [
        "rs", "js", "ts", "tsx", "jsx", "py", "go", "java", "kt", "kts", "rb", "php", "c", "cc",
        "cpp", "h", "hpp", "cs", "swift", "m", "mm", "scala", "sql", "toml", "yaml", "yml",
        "json", "md", "markdown", "txt", "lock", "cfg", "ini", "env", "nix", "zig", "dart",
    ];
    for ext in known_extensions {
        // Create a row where the only non-stopword would be the extension
        let data = export(vec![row(&format!("module/{ext}.rs"), "module", 50)], &[]);
        let terms = overall_terms(&data);
        assert!(
            !terms.contains(&ext.to_string()),
            "extension '{ext}' should be a stopword but found in terms: {terms:?}"
        );
    }
}

// ── 15. Base stopwords filter common directories ────────────────

#[test]
fn base_stopwords_filter_common_directories() {
    let base_stops = [
        "src", "lib", "mod", "index", "test", "tests", "impl", "main", "bin", "pkg", "package",
        "target", "build", "dist", "out", "gen", "generated",
    ];
    for stop in base_stops {
        let data = export(
            vec![row(&format!("{stop}/feature.rs"), stop, 50)],
            &[],
        );
        let terms = overall_terms(&data);
        assert!(
            !terms.contains(&stop.to_string()),
            "base stopword '{stop}' should be filtered: {terms:?}"
        );
    }
}
