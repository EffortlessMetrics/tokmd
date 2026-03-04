//! Wave-42 deep tests for topic-cloud extraction.
//!
//! Tests topic extraction from paths, stopword filtering, TF-IDF scoring,
//! determinism, serde roundtrip, and edge cases.

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

// ── 1. Stopwords are filtered out ───────────────────────────────

#[test]
fn stopwords_filtered_from_topics() {
    let data = export(vec![row("src/lib/auth/handler.rs", "auth", 50)], &[]);
    let terms = overall_terms(&data);
    // "src", "lib", "rs" are all stopwords
    assert!(
        !terms.contains(&"src".to_string()),
        "src should be a stopword"
    );
    assert!(
        !terms.contains(&"lib".to_string()),
        "lib should be a stopword"
    );
    assert!(
        !terms.contains(&"rs".to_string()),
        "rs should be a stopword"
    );
    assert!(terms.contains(&"auth".to_string()));
    assert!(terms.contains(&"handler".to_string()));
}

// ── 2. Module roots are stopwords ───────────────────────────────

#[test]
fn module_roots_are_stopwords() {
    let data = export(
        vec![row("crates/auth/login.rs", "crates/auth", 50)],
        &["crates"],
    );
    let terms = overall_terms(&data);
    assert!(
        !terms.contains(&"crates".to_string()),
        "module root 'crates' should be a stopword"
    );
    assert!(terms.contains(&"auth".to_string()));
}

// ── 3. Deterministic output across repeated calls ───────────────

#[test]
fn topic_clouds_deterministic() {
    let data = export(
        vec![
            row("api/auth/login.rs", "api/auth", 50),
            row("api/auth/token.rs", "api/auth", 50),
            row("api/payments/stripe.rs", "api/payments", 50),
        ],
        &[],
    );
    let clouds1 = build_topic_clouds(&data);
    let clouds2 = build_topic_clouds(&data);
    assert_eq!(clouds1.overall.len(), clouds2.overall.len());
    for (a, b) in clouds1.overall.iter().zip(clouds2.overall.iter()) {
        assert_eq!(a.term, b.term);
        assert_eq!(a.tf, b.tf);
    }
}

// ── 4. Empty export produces empty topics ───────────────────────

#[test]
fn empty_export_produces_empty_topics() {
    let data = export(vec![], &[]);
    let clouds = build_topic_clouds(&data);
    assert!(clouds.overall.is_empty());
    assert!(clouds.per_module.is_empty());
}

// ── 5. Child rows are excluded ──────────────────────────────────

#[test]
fn child_rows_excluded_from_topics() {
    let mut r = row("api/handler.rs", "api", 50);
    r.kind = FileKind::Child;
    let data = export(vec![r], &[]);
    let clouds = build_topic_clouds(&data);
    assert!(clouds.overall.is_empty());
}

// ── 6. Per-module clouds are separate ───────────────────────────

#[test]
fn per_module_clouds_separate() {
    let data = export(
        vec![
            row("auth/login.rs", "auth", 50),
            row("auth/signup.rs", "auth", 50),
            row("billing/invoice.rs", "billing", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    assert!(clouds.per_module.contains_key("auth"));
    assert!(clouds.per_module.contains_key("billing"));
    let auth_terms: Vec<_> = clouds.per_module["auth"].iter().map(|t| &t.term).collect();
    let billing_terms: Vec<_> = clouds.per_module["billing"]
        .iter()
        .map(|t| &t.term)
        .collect();
    assert!(auth_terms.contains(&&"login".to_string()));
    assert!(billing_terms.contains(&&"invoice".to_string()));
}

// ── 7. Serde roundtrip for TopicClouds ──────────────────────────

#[test]
fn topic_clouds_serde_roundtrip() {
    let data = export(
        vec![
            row("api/handler.rs", "api", 50),
            row("api/router.rs", "api", 40),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let json = serde_json::to_string(&clouds).unwrap();
    let deser: tokmd_analysis_types::TopicClouds = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.overall.len(), clouds.overall.len());
    for (a, b) in deser.overall.iter().zip(clouds.overall.iter()) {
        assert_eq!(a.term, b.term);
    }
}

// ── 8. Overall limited to TOP_K=8 entries ───────────────────────

#[test]
fn overall_capped_at_top_k() {
    // Create many distinct terms
    let rows: Vec<FileRow> = (0..20)
        .map(|i| row(&format!("unique_term_{i}/file.rs"), "mod", 50))
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);
    assert!(
        clouds.overall.len() <= 8,
        "overall should be capped at TOP_K=8, got {}",
        clouds.overall.len()
    );
}

// ── 9. Scores are positive ──────────────────────────────────────

#[test]
fn all_scores_positive() {
    let data = export(
        vec![
            row("api/handler.rs", "api", 50),
            row("core/engine.rs", "core", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    for term in &clouds.overall {
        assert!(term.score > 0.0, "score should be positive: {:?}", term);
    }
}

// ── 10. Underscore and hyphen splitting ─────────────────────────

#[test]
fn underscore_and_hyphen_splitting() {
    let data = export(vec![row("api/my_cool-handler.rs", "api", 50)], &[]);
    let terms = overall_terms(&data);
    assert!(terms.contains(&"my".to_string()));
    assert!(terms.contains(&"cool".to_string()));
    assert!(terms.contains(&"handler".to_string()));
}

// ── 11. Backslash paths normalised ──────────────────────────────

#[test]
fn backslash_paths_normalised() {
    let data = export(vec![row("api\\handler\\auth.rs", "api", 50)], &[]);
    let terms = overall_terms(&data);
    assert!(terms.contains(&"handler".to_string()));
    assert!(terms.contains(&"auth".to_string()));
}
