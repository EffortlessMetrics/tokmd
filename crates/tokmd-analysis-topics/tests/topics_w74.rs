//! W74 – Unit tests for tokmd-analysis-topics enricher.

use tokmd_analysis_topics::build_topic_clouds;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helper: build an ExportData from (path, module, lang, tokens) tuples
// ---------------------------------------------------------------------------
fn make_export(files: &[(&str, &str, &str, usize)]) -> ExportData {
    let rows = files
        .iter()
        .map(|(path, module, lang, tokens)| FileRow {
            path: path.to_string(),
            module: module.to_string(),
            lang: lang.to_string(),
            kind: FileKind::Parent,
            code: 50,
            comments: 5,
            blanks: 5,
            lines: 60,
            bytes: 500,
            tokens: *tokens,
        })
        .collect();
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ── Empty input ───────────────────────────────────────────────────────────

#[test]
fn empty_export_yields_empty_topics() {
    let export = make_export(&[]);
    let clouds = build_topic_clouds(&export);
    assert!(clouds.overall.is_empty());
    assert!(clouds.per_module.is_empty());
}

// ── Basic topic extraction ────────────────────────────────────────────────

#[test]
fn topics_from_path_tokens() {
    let export = make_export(&[
        ("crates/auth/login.rs", "crates/auth", "Rust", 100),
        ("crates/auth/token.rs", "crates/auth", "Rust", 100),
    ]);
    let clouds = build_topic_clouds(&export);
    let auth = clouds.per_module.get("crates/auth").unwrap();
    let terms: Vec<&str> = auth.iter().map(|t| t.term.as_str()).collect();
    assert!(terms.contains(&"login"), "expected 'login' in {terms:?}");
    assert!(terms.contains(&"token"), "expected 'token' in {terms:?}");
}

#[test]
fn overall_topics_are_populated() {
    let export = make_export(&[
        ("crates/auth/login.rs", "crates/auth", "Rust", 100),
        ("crates/payments/stripe.rs", "crates/payments", "Rust", 100),
    ]);
    let clouds = build_topic_clouds(&export);
    assert!(!clouds.overall.is_empty());
}

// ── Stopword filtering ───────────────────────────────────────────────────

#[test]
fn file_extensions_are_stopwords() {
    // "rs" should be filtered out as a stopword
    let export = make_export(&[("crates/auth/login.rs", "crates/auth", "Rust", 100)]);
    let clouds = build_topic_clouds(&export);
    let all_terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(!all_terms.contains(&"rs"), "'rs' should be a stopword");
}

#[test]
fn common_dirs_are_stopwords() {
    // "src", "lib", "test" should be filtered
    let export = make_export(&[("src/lib/utils/helpers.rs", "src/lib", "Rust", 100)]);
    let clouds = build_topic_clouds(&export);
    let all_terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(!all_terms.contains(&"src"), "'src' should be a stopword");
    assert!(!all_terms.contains(&"lib"), "'lib' should be a stopword");
}

// ── Per-module isolation ──────────────────────────────────────────────────

#[test]
fn topics_are_grouped_by_module() {
    let export = make_export(&[
        ("crates/auth/login.rs", "crates/auth", "Rust", 100),
        ("crates/payments/invoice.rs", "crates/payments", "Rust", 100),
    ]);
    let clouds = build_topic_clouds(&export);
    assert!(clouds.per_module.contains_key("crates/auth"));
    assert!(clouds.per_module.contains_key("crates/payments"));
}

#[test]
fn module_topics_contain_relevant_terms() {
    let export = make_export(&[
        (
            "crates/payments/stripe_api.rs",
            "crates/payments",
            "Rust",
            100,
        ),
        ("crates/payments/refund.rs", "crates/payments", "Rust", 100),
    ]);
    let clouds = build_topic_clouds(&export);
    let payments = clouds.per_module.get("crates/payments").unwrap();
    let terms: Vec<&str> = payments.iter().map(|t| t.term.as_str()).collect();
    assert!(terms.contains(&"stripe"), "expected 'stripe' in {terms:?}");
    assert!(terms.contains(&"refund"), "expected 'refund' in {terms:?}");
}

// ── Determinism ───────────────────────────────────────────────────────────

#[test]
fn topic_clouds_are_deterministic() {
    let export = make_export(&[
        ("crates/auth/login.rs", "crates/auth", "Rust", 100),
        ("crates/auth/token.rs", "crates/auth", "Rust", 100),
        ("crates/payments/stripe.rs", "crates/payments", "Rust", 100),
    ]);
    let c1 = build_topic_clouds(&export);
    let c2 = build_topic_clouds(&export);
    assert_eq!(c1.overall.len(), c2.overall.len());
    for (a, b) in c1.overall.iter().zip(c2.overall.iter()) {
        assert_eq!(a.term, b.term);
        assert_eq!(a.tf, b.tf);
        assert_eq!(a.df, b.df);
    }
}

// ── TopicTerm fields ──────────────────────────────────────────────────────

#[test]
fn topic_terms_have_positive_scores() {
    let export = make_export(&[("crates/auth/login.rs", "crates/auth", "Rust", 100)]);
    let clouds = build_topic_clouds(&export);
    for term in &clouds.overall {
        assert!(term.score > 0.0, "score should be positive: {:?}", term);
        assert!(term.tf > 0, "tf should be > 0");
    }
}

#[test]
fn top_k_limits_per_module_terms() {
    // Even with many files, per-module topics should be limited to top-K (8)
    let files: Vec<(&str, &str, &str, usize)> = (0..20)
        .map(|i| {
            // Leak strings to get &str with 'static lifetime
            let path: &'static str = Box::leak(format!("mod/file_{i}.rs").into_boxed_str());
            (path, "mod", "Rust", 100usize)
        })
        .collect();
    let export = make_export(&files);
    let clouds = build_topic_clouds(&export);
    if let Some(topics) = clouds.per_module.get("mod") {
        assert!(topics.len() <= 8, "should be capped at top-K=8");
    }
}
