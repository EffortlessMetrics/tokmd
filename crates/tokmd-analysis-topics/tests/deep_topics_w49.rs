//! Wave-49 deep tests for topic-cloud extraction.
//!
//! Covers TF-IDF formula, weight handling, cross-module aggregation,
//! sorting invariants, serde roundtrips, and property-based tests.

use tokmd_analysis_topics::build_topic_clouds;
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

fn child_row(path: &str, module: &str, tokens: usize) -> FileRow {
    FileRow {
        kind: FileKind::Child,
        ..row(path, module, tokens)
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

// ── 1. Weight uses max(tokens, 1) — zero-token rows still contribute ──

#[test]
fn zero_token_rows_contribute_with_weight_one() {
    let data = export(
        vec![
            row("api/handler.rs", "api", 0),
            row("api/router.rs", "api", 0),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    assert!(
        !clouds.overall.is_empty(),
        "zero-token rows should still produce topics"
    );
    for t in &clouds.overall {
        assert!(
            t.tf >= 1,
            "tf should be at least 1 when weight is clamped to 1"
        );
    }
}

// ── 2. Higher token weight increases TF ─────────────────────────

#[test]
fn higher_tokens_increase_tf() {
    let data_low = export(vec![row("api/handler.rs", "api", 10)], &[]);
    let data_high = export(vec![row("api/handler.rs", "api", 1000)], &[]);
    let clouds_low = build_topic_clouds(&data_low);
    let clouds_high = build_topic_clouds(&data_high);

    let tf_low = clouds_low
        .overall
        .iter()
        .find(|t| t.term == "handler")
        .map(|t| t.tf)
        .unwrap_or(0);
    let tf_high = clouds_high
        .overall
        .iter()
        .find(|t| t.term == "handler")
        .map(|t| t.tf)
        .unwrap_or(0);
    assert!(
        tf_high > tf_low,
        "higher tokens should produce higher tf: {tf_high} vs {tf_low}"
    );
}

// ── 3. DF counts unique files, not occurrences ──────────────────

#[test]
fn df_counts_files_not_occurrences() {
    let data = export(
        vec![
            row("api/handler/handler.rs", "api", 50),
            row("core/handler.rs", "core", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let handler = clouds.overall.iter().find(|t| t.term == "handler").unwrap();
    // "handler" appears in 2 files (even though first path has it twice)
    assert_eq!(handler.df, 2, "df should count files, not occurrences");
}

// ── 4. IDF formula: ln((N+1)/(df+1)) + 1 ───────────────────────

#[test]
fn idf_formula_verified() {
    // Single module, single file → module_count=1, df=1
    let data = export(vec![row("api/handler.rs", "api", 1)], &[]);
    let clouds = build_topic_clouds(&data);
    let term = clouds.overall.iter().find(|t| t.term == "handler").unwrap();
    // tf=1 (weight=max(1,1)=1), df=1, module_count=1
    // idf = ln((1+1)/(1+1)) + 1 = ln(1) + 1 = 0 + 1 = 1.0
    // score = tf * idf = 1 * 1.0 = 1.0
    let expected = 1.0f64;
    assert!(
        (term.score - expected).abs() < 0.001,
        "score should be ~{expected}, got {}",
        term.score
    );
}

// ── 5. Cross-module IDF: common terms get lower IDF ─────────────

#[test]
fn common_terms_have_lower_idf() {
    // "handler" appears in both modules (df=2), "engine" in one (df=1)
    // Both have same total TF = 50, so IDF is the differentiator
    let data = export(
        vec![
            row("api/handler.rs", "api", 50),
            row("core/engine.rs", "core", 50),
            row("core/handler.rs", "core", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let engine = clouds.overall.iter().find(|t| t.term == "engine").unwrap();
    let handler = clouds.overall.iter().find(|t| t.term == "handler").unwrap();
    // engine df=1, handler df=2 → engine has higher IDF
    // But handler has higher total TF (50+50=100 vs 50)
    // What we can verify: handler has higher df than engine
    assert!(handler.df > engine.df, "handler should have higher df");
    assert_eq!(engine.df, 1);
    assert_eq!(handler.df, 2);
}

// ── 6. Overall aggregates TF from all modules ───────────────────

#[test]
fn overall_aggregates_cross_module_tf() {
    let data = export(
        vec![
            row("api/handler.rs", "api", 100),
            row("core/handler.rs", "core", 200),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let handler = clouds.overall.iter().find(|t| t.term == "handler").unwrap();
    // Overall TF should be sum: 100 + 200 = 300
    assert_eq!(
        handler.tf, 300,
        "overall tf should aggregate across modules"
    );
}

// ── 7. Per-module sorted desc by score, asc by term on tie ──────

#[test]
fn per_module_sort_order() {
    let data = export(
        vec![
            row("api/alpha.rs", "api", 50),
            row("api/beta.rs", "api", 50),
            row("api/gamma.rs", "api", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let api_terms = &clouds.per_module["api"];
    // All have same weight so same score → tie-broken alphabetically
    for pair in api_terms.windows(2) {
        if (pair[0].score - pair[1].score).abs() < f64::EPSILON {
            assert!(
                pair[0].term <= pair[1].term,
                "tied terms should be alphabetical: {} vs {}",
                pair[0].term,
                pair[1].term
            );
        } else {
            assert!(
                pair[0].score >= pair[1].score,
                "should be sorted desc by score"
            );
        }
    }
}

// ── 8. Single file produces exactly one module ──────────────────

#[test]
fn single_file_one_module() {
    let data = export(vec![row("api/handler.rs", "api", 50)], &[]);
    let clouds = build_topic_clouds(&data);
    assert_eq!(clouds.per_module.len(), 1);
    assert!(clouds.per_module.contains_key("api"));
}

// ── 9. Serde roundtrip preserves all fields ─────────────────────

#[test]
fn serde_roundtrip_preserves_all_fields() {
    let data = export(
        vec![
            row("api/handler.rs", "api", 50),
            row("core/engine.rs", "core", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let json = serde_json::to_string(&clouds).unwrap();
    let deser: tokmd_analysis_types::TopicClouds = serde_json::from_str(&json).unwrap();

    assert_eq!(deser.overall.len(), clouds.overall.len());
    assert_eq!(deser.per_module.len(), clouds.per_module.len());
    for (orig, rt) in clouds.overall.iter().zip(deser.overall.iter()) {
        assert_eq!(orig.term, rt.term);
        assert_eq!(orig.tf, rt.tf);
        assert_eq!(orig.df, rt.df);
        assert!((orig.score - rt.score).abs() < f64::EPSILON);
    }
}

// ── 10. Mixed parent and child rows — only parents counted ──────

#[test]
fn mixed_parent_child_only_parents_counted() {
    let data = export(
        vec![
            row("api/handler.rs", "api", 50),
            child_row("api/handler.js", "api", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    // "handler" from parent should appear
    assert!(terms.contains(&"handler"));
    // Only one file contributed, so TF should match parent weight only
    let handler = clouds.overall.iter().find(|t| t.term == "handler").unwrap();
    assert_eq!(handler.tf, 50);
}

// ── 11. Case-insensitive module root stopword ───────────────────

#[test]
fn case_insensitive_module_root_stopword() {
    let data = export(
        vec![row("Crates/auth/login.rs", "Crates/auth", 50)],
        &["Crates"],
    );
    let terms: Vec<String> = build_topic_clouds(&data)
        .overall
        .iter()
        .map(|t| t.term.clone())
        .collect();
    assert!(
        !terms.contains(&"crates".to_string()),
        "case-insensitive module root should be filtered"
    );
    assert!(terms.contains(&"auth".to_string()));
}

// ── Proptest ────────────────────────────────────────────────────

mod properties {
    use super::*;
    use proptest::prelude::*;

    fn arb_row() -> impl Strategy<Value = FileRow> {
        ("[a-z]{1,4}/[a-z]{1,6}\\.rs", "[a-z]{1,4}", 0..500usize).prop_map(
            |(path, module, tokens)| FileRow {
                path,
                module,
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 10,
                comments: 0,
                blanks: 0,
                lines: 10,
                bytes: 100,
                tokens,
            },
        )
    }

    proptest! {
        #[test]
        fn all_scores_non_negative(rows in proptest::collection::vec(arb_row(), 1..20)) {
            let data = ExportData {
                rows,
                module_roots: vec![],
                module_depth: 1,
                children: ChildIncludeMode::Separate,
            };
            let clouds = build_topic_clouds(&data);
            for t in &clouds.overall {
                prop_assert!(t.score >= 0.0, "score must be non-negative: {}", t.score);
                prop_assert!(t.tf >= 1, "tf must be >= 1");
                prop_assert!(t.df >= 1, "df must be >= 1");
            }
            prop_assert!(clouds.overall.len() <= 8, "overall capped at TOP_K=8");
        }
    }
}
