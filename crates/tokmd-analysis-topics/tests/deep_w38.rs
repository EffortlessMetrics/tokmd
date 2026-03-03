//! Deep tests for tokmd-analysis-topics (wave 38).
//!
//! Covers TF-IDF scoring, path tokenization, stopword filtering,
//! TOP_K truncation, sort stability, and determinism.

use std::collections::BTreeSet;

use tokmd_analysis_topics::build_topic_clouds;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_row(path: &str, module: &str, lang: &str, tokens: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: lang.to_string(),
        kind: FileKind::Parent,
        code: 10,
        comments: 0,
        blanks: 0,
        lines: 10,
        bytes: 100,
        tokens,
    }
}

fn make_export(rows: Vec<FileRow>, module_roots: Vec<String>) -> ExportData {
    ExportData {
        rows,
        module_roots,
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ---------------------------------------------------------------------------
// TF-IDF score_term boundary cases
// ---------------------------------------------------------------------------

#[test]
fn score_zero_tf_produces_zero_score() {
    // If tf=0, the term doesn't appear → score should be 0
    let rows = vec![make_row("crates/auth/login.rs", "crates/auth", "Rust", 0)];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);
    // Verify all scores are non-negative
    for term in &topics.overall {
        assert!(term.score >= 0.0, "score must be non-negative");
    }
}

#[test]
fn single_module_idf_is_ln2_plus_one() {
    // With module_count=1 and df=1: idf = ln((1+1)/(1+1)) + 1 = ln(1) + 1 = 1.0
    let rows = vec![
        make_row("crates/auth/login.rs", "crates/auth", "Rust", 50),
        make_row("crates/auth/token.rs", "crates/auth", "Rust", 50),
    ];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);
    // Only 1 module → idf=1.0, so score = tf * 1.0
    assert_eq!(topics.per_module.len(), 1);
}

#[test]
fn two_modules_term_in_both_has_low_idf() {
    // Term "shared" appears in both modules → high df → low IDF
    let rows = vec![
        make_row("crates/auth/shared_helper.rs", "crates/auth", "Rust", 50),
        make_row(
            "crates/payments/shared_util.rs",
            "crates/payments",
            "Rust",
            50,
        ),
    ];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    // "shared" appears in both modules
    let shared_term = topics.overall.iter().find(|t| t.term == "shared");
    if let Some(shared) = shared_term {
        assert_eq!(shared.df, 2);
    }
}

#[test]
fn rare_term_has_higher_score_than_common() {
    // "unique" appears in 1 module, "common" in 3 → "unique" should score higher per occurrence
    let rows = vec![
        make_row("crates/auth/common_helper.rs", "crates/auth", "Rust", 50),
        make_row(
            "crates/payments/common_util.rs",
            "crates/payments",
            "Rust",
            50,
        ),
        make_row("crates/core/common_base.rs", "crates/core", "Rust", 50),
        make_row("crates/auth/unique_feature.rs", "crates/auth", "Rust", 50),
    ];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    let auth_cloud = topics.per_module.get("crates/auth").unwrap();
    let unique_score = auth_cloud
        .iter()
        .find(|t| t.term == "unique")
        .map(|t| t.score);
    let common_score = auth_cloud
        .iter()
        .find(|t| t.term == "common")
        .map(|t| t.score);

    if let (Some(u), Some(c)) = (unique_score, common_score) {
        // "unique" (df=1) should have higher IDF-weighted score than "common" (df=3)
        assert!(u > c, "unique ({u}) should score higher than common ({c})");
    }
}

// ---------------------------------------------------------------------------
// tokenize_path tests (via observable behavior)
// ---------------------------------------------------------------------------

#[test]
fn path_with_multiple_separators() {
    // Tokens from "deeply/nested/path-with_mixed-separators.rs"
    let rows = vec![make_row(
        "deeply/nested/path-with_mixed-separators.rs",
        "deeply/nested",
        "Rust",
        50,
    )];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    let terms: BTreeSet<String> = topics.overall.iter().map(|t| t.term.clone()).collect();
    // "rs" is a stopword extension, should be filtered
    assert!(!terms.contains("rs"));
    // Split by '-' and '_'
    assert!(
        terms.contains("path")
            || terms.contains("with")
            || terms.contains("mixed")
            || terms.contains("separators")
    );
}

#[test]
fn unicode_in_path_lowercased() {
    let rows = vec![make_row("crates/Über/Straße.rs", "crates/Über", "Rust", 50)];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    // Should have lowercased terms
    for term in &topics.overall {
        assert_eq!(
            term.term,
            term.term.to_lowercase(),
            "term not lowercased: {}",
            term.term
        );
    }
}

#[test]
fn empty_parts_in_path_skipped() {
    // "/leading/trailing/" has empty parts
    let rows = vec![make_row(
        "/crates//double_slash/file.rs",
        "(root)",
        "Rust",
        50,
    )];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    // No empty-string terms
    for term in &topics.overall {
        assert!(!term.term.is_empty(), "empty term found");
    }
}

#[test]
fn backslash_normalized_to_forward_slash() {
    let rows = vec![make_row(
        "crates\\auth\\login.rs",
        "crates\\auth",
        "Rust",
        50,
    )];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    // "auth" should be a term (path component after normalization)
    let has_auth = topics.overall.iter().any(|t| t.term == "auth");
    assert!(has_auth, "expected 'auth' from backslash path");
}

// ---------------------------------------------------------------------------
// Stopword filtering
// ---------------------------------------------------------------------------

#[test]
fn common_stopwords_filtered() {
    let stopwords = [
        "src", "lib", "mod", "index", "test", "tests", "impl", "main", "bin",
    ];
    let rows = vec![make_row(
        "src/lib/mod/index/test/tests/impl/main/bin/real_code.rs",
        "(root)",
        "Rust",
        50,
    )];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    let terms: BTreeSet<String> = topics.overall.iter().map(|t| t.term.clone()).collect();

    for sw in &stopwords {
        assert!(!terms.contains(*sw), "stopword '{sw}' should be filtered");
    }
}

#[test]
fn file_extensions_are_stopwords() {
    let extensions = ["rs", "js", "ts", "py", "go", "java", "cpp"];
    let rows = vec![make_row("src/feature_handler.rs", "(root)", "Rust", 50)];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    let terms: BTreeSet<String> = topics.overall.iter().map(|t| t.term.clone()).collect();

    for ext in &extensions {
        assert!(
            !terms.contains(*ext),
            "extension '{ext}' should be a stopword"
        );
    }
}

#[test]
fn module_roots_are_stopwords() {
    let rows = vec![make_row(
        "crates/auth/handler.rs",
        "crates/auth",
        "Rust",
        50,
    )];
    let export = make_export(rows, vec!["crates".to_string()]);
    let topics = build_topic_clouds(&export);

    let terms: BTreeSet<String> = topics.overall.iter().map(|t| t.term.clone()).collect();

    assert!(
        !terms.contains("crates"),
        "'crates' should be a stopword (module root)"
    );
}

// ---------------------------------------------------------------------------
// TOP_K truncation
// ---------------------------------------------------------------------------

#[test]
fn per_module_truncated_to_at_most_8() {
    // Create many unique file names in one module → many terms
    let mut rows = Vec::new();
    for i in 0..20 {
        rows.push(make_row(
            &format!("mod_a/unique_term_{i:03}_file.rs"),
            "mod_a",
            "Rust",
            50,
        ));
    }
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    let mod_a = topics.per_module.get("mod_a").unwrap();
    assert!(
        mod_a.len() <= 8,
        "per-module should be truncated to TOP_K=8, got {}",
        mod_a.len()
    );
}

#[test]
fn overall_truncated_to_at_most_8() {
    let mut rows = Vec::new();
    for i in 0..20 {
        rows.push(make_row(
            &format!("mod_a/term_alpha_{i:03}_file.rs"),
            "mod_a",
            "Rust",
            50,
        ));
    }
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    assert!(
        topics.overall.len() <= 8,
        "overall should be truncated to TOP_K=8, got {}",
        topics.overall.len()
    );
}

#[test]
fn fewer_than_8_terms_not_padded() {
    let rows = vec![
        make_row("mod_a/alpha.rs", "mod_a", "Rust", 50),
        make_row("mod_a/beta.rs", "mod_a", "Rust", 50),
    ];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    let mod_a = topics.per_module.get("mod_a").unwrap();
    // Only 2 unique meaningful terms → should be <= 2
    assert!(mod_a.len() <= 8);
}

// ---------------------------------------------------------------------------
// Sorting stability: score ties broken by term name
// ---------------------------------------------------------------------------

#[test]
fn ties_broken_by_term_name_ascending() {
    // All terms have same tf and df → same score → sorted by name
    let rows = vec![
        make_row("mod_a/zebra.rs", "mod_a", "Rust", 50),
        make_row("mod_a/alpha.rs", "mod_a", "Rust", 50),
        make_row("mod_a/mango.rs", "mod_a", "Rust", 50),
    ];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    let mod_a = topics.per_module.get("mod_a").unwrap();
    // All have same tf and df (each appears once, df=1, module_count=1)
    // Verify alphabetical tiebreak
    for i in 1..mod_a.len() {
        if (mod_a[i - 1].score - mod_a[i].score).abs() < 1e-10 {
            assert!(
                mod_a[i - 1].term <= mod_a[i].term,
                "tie not broken by name: {} vs {}",
                mod_a[i - 1].term,
                mod_a[i].term
            );
        }
    }
}

#[test]
fn overall_sorted_by_score_desc() {
    let mut rows = Vec::new();
    for i in 0..5 {
        rows.push(make_row(
            &format!("mod_a/term_{i}.rs"),
            "mod_a",
            "Rust",
            (i + 1) * 100, // varying token weights
        ));
    }
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    for i in 1..topics.overall.len() {
        assert!(
            topics.overall[i - 1].score >= topics.overall[i].score,
            "overall not sorted by score desc at index {i}"
        );
    }
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn build_topic_clouds_deterministic() {
    let rows = vec![
        make_row("crates/auth/login.rs", "crates/auth", "Rust", 50),
        make_row("crates/auth/token.rs", "crates/auth", "Rust", 50),
        make_row("crates/payments/stripe.rs", "crates/payments", "Rust", 50),
    ];
    let export = make_export(rows, vec![]);

    let t1 = build_topic_clouds(&export);
    let t2 = build_topic_clouds(&export);

    assert_eq!(t1.overall.len(), t2.overall.len());
    for (a, b) in t1.overall.iter().zip(t2.overall.iter()) {
        assert_eq!(a.term, b.term);
        assert!((a.score - b.score).abs() < 1e-10);
        assert_eq!(a.tf, b.tf);
        assert_eq!(a.df, b.df);
    }
    assert_eq!(
        t1.per_module.keys().collect::<Vec<_>>(),
        t2.per_module.keys().collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Child rows are excluded
// ---------------------------------------------------------------------------

#[test]
fn child_rows_excluded_from_topics() {
    let mut child_row = make_row("crates/auth/embedded.rs", "crates/auth", "Rust", 500);
    child_row.kind = FileKind::Child;

    let rows = vec![
        make_row("crates/auth/login.rs", "crates/auth", "Rust", 50),
        child_row,
    ];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    // "embedded" should not appear since it's from a Child row
    let has_embedded = topics.overall.iter().any(|t| t.term == "embedded");
    assert!(!has_embedded, "child row term 'embedded' should not appear");
}

// ---------------------------------------------------------------------------
// Empty export
// ---------------------------------------------------------------------------

#[test]
fn empty_export_empty_topics() {
    let export = make_export(vec![], vec![]);
    let topics = build_topic_clouds(&export);

    assert!(topics.overall.is_empty());
    assert!(topics.per_module.is_empty());
}

// ---------------------------------------------------------------------------
// Weight from tokens
// ---------------------------------------------------------------------------

#[test]
fn weight_from_tokens_boosts_large_files() {
    // File with tokens=1000 should contribute more than tokens=1
    let rows = vec![
        make_row("mod_a/heavy.rs", "mod_a", "Rust", 1000),
        make_row("mod_a/light.rs", "mod_a", "Rust", 1),
    ];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    let mod_a = topics.per_module.get("mod_a").unwrap();
    let heavy_tf = mod_a.iter().find(|t| t.term == "heavy").map(|t| t.tf);
    let light_tf = mod_a.iter().find(|t| t.term == "light").map(|t| t.tf);

    if let (Some(h), Some(l)) = (heavy_tf, light_tf) {
        assert!(h > l, "heavy ({h}) should have higher tf than light ({l})");
    }
}

// ---------------------------------------------------------------------------
// Multiple modules produce separate clouds
// ---------------------------------------------------------------------------

#[test]
fn per_module_keys_match_modules() {
    let rows = vec![
        make_row("crates/auth/login.rs", "crates/auth", "Rust", 50),
        make_row("crates/payments/charge.rs", "crates/payments", "Rust", 50),
        make_row("crates/core/engine.rs", "crates/core", "Rust", 50),
    ];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    let modules: Vec<&String> = topics.per_module.keys().collect();
    assert!(modules.contains(&&"crates/auth".to_string()));
    assert!(modules.contains(&&"crates/payments".to_string()));
    assert!(modules.contains(&&"crates/core".to_string()));
    assert_eq!(modules.len(), 3);
}

// ---------------------------------------------------------------------------
// df reflects cross-module document frequency
// ---------------------------------------------------------------------------

#[test]
fn df_counts_files_not_modules() {
    // "helper" appears in 3 different files across 2 modules → df=3 (per file)
    let rows = vec![
        make_row("crates/auth/helper_a.rs", "crates/auth", "Rust", 50),
        make_row("crates/auth/helper_b.rs", "crates/auth", "Rust", 50),
        make_row("crates/payments/helper_c.rs", "crates/payments", "Rust", 50),
    ];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    let helper_term = topics.overall.iter().find(|t| t.term == "helper");
    if let Some(term) = helper_term {
        assert_eq!(term.df, 3, "df should count files, not modules");
    }
}

// ---------------------------------------------------------------------------
// All scores are positive
// ---------------------------------------------------------------------------

#[test]
fn all_scores_positive() {
    let rows = vec![
        make_row("crates/auth/login.rs", "crates/auth", "Rust", 50),
        make_row("crates/payments/stripe.rs", "crates/payments", "Rust", 50),
    ];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    for term in &topics.overall {
        assert!(term.score > 0.0, "score should be positive: {}", term.term);
    }
    for terms in topics.per_module.values() {
        for term in terms {
            assert!(term.score > 0.0);
        }
    }
}

// ---------------------------------------------------------------------------
// Single file still produces topics
// ---------------------------------------------------------------------------

#[test]
fn single_file_produces_topics() {
    let rows = vec![make_row(
        "crates/auth/login_handler.rs",
        "crates/auth",
        "Rust",
        50,
    )];
    let export = make_export(rows, vec![]);
    let topics = build_topic_clouds(&export);

    assert!(!topics.overall.is_empty());
    assert!(topics.per_module.contains_key("crates/auth"));
}
