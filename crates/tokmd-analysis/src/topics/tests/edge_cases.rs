//! Edge-case BDD tests for topic-cloud extraction.

use crate::topics::build_topic_clouds;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

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

fn make_export(rows: Vec<FileRow>, module_roots: Vec<&str>) -> ExportData {
    ExportData {
        rows,
        module_roots: module_roots.into_iter().map(String::from).collect(),
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ── Scenario: multi-language files share topic space ─────────────────

#[test]
fn given_files_in_different_languages_when_extracted_then_topics_are_language_agnostic() {
    let rows = vec![
        make_row("app/controller.rs", "app", "Rust", 50),
        make_row("app/controller.py", "app", "Python", 50),
        make_row("app/controller.js", "app", "JavaScript", 50),
    ];
    let export = make_export(rows, vec!["app"]);
    let clouds = build_topic_clouds(&export);

    // "controller" should appear as a term regardless of language
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(
        terms.contains(&"controller"),
        "expected 'controller' across languages, got {terms:?}"
    );
}

// ── Scenario: language extensions as stopwords for all langs ─────────

#[test]
fn given_python_file_extension_when_extracted_then_py_is_stopped() {
    let rows = vec![make_row("app/handler.py", "app", "Python", 50)];
    let export = make_export(rows, vec!["app"]);
    let clouds = build_topic_clouds(&export);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(!terms.contains(&"py"), "'py' should be a stopword");
    assert!(terms.contains(&"handler"));
}

#[test]
fn given_go_file_extension_when_extracted_then_go_is_stopped() {
    let rows = vec![make_row("cmd/server.go", "cmd", "Go", 50)];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(!terms.contains(&"go"), "'go' should be a stopword");
    assert!(terms.contains(&"server"));
    assert!(terms.contains(&"cmd"));
}

// ── Scenario: single row with only stopword path ────────────────────

#[test]
fn given_row_with_only_stopwords_when_extracted_then_no_topics() {
    let rows = vec![make_row("src/lib/mod.rs", "src/lib", "Rust", 50)];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);
    assert!(
        clouds.overall.is_empty(),
        "all-stopword path should produce no topics"
    );
}

// ── Scenario: very long filename with many separators ────────────────

#[test]
fn given_long_compound_filename_when_extracted_then_all_parts_tokenized() {
    let rows = vec![make_row(
        "services/user_auth-handler.v3.test.rs",
        "services",
        "Rust",
        50,
    )];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(terms.contains(&"user"));
    assert!(terms.contains(&"auth"));
    assert!(terms.contains(&"handler"));
    assert!(terms.contains(&"v3"));
    assert!(terms.contains(&"services"));
    // "test" and "rs" are stopwords
    assert!(!terms.contains(&"rs"));
}

// ── Scenario: module_roots with multiple roots ──────────────────────

#[test]
fn given_multiple_module_roots_when_extracted_then_all_are_stopwords() {
    let rows = vec![
        make_row("crates/auth/login.rs", "crates/auth", "Rust", 50),
        make_row("packages/ui/button.ts", "packages/ui", "TypeScript", 50),
    ];
    let export = make_export(rows, vec!["crates", "packages"]);
    let clouds = build_topic_clouds(&export);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(
        !terms.contains(&"crates"),
        "'crates' is a module root stopword"
    );
    assert!(
        !terms.contains(&"packages"),
        "'packages' is a module root stopword"
    );
    assert!(terms.contains(&"auth") || terms.contains(&"login"));
}

// ── Scenario: weight for zero-token rows ────────────────────────────

#[test]
fn given_zero_token_row_when_extracted_then_weight_is_clamped_to_one() {
    let rows = vec![make_row("app/feature.rs", "app", "Rust", 0)];
    let export = make_export(rows, vec!["app"]);
    let clouds = build_topic_clouds(&export);
    let feature = clouds.overall.iter().find(|t| t.term == "feature");
    assert!(feature.is_some(), "term should exist even with 0 tokens");
    assert!(feature.unwrap().tf >= 1, "tf should be at least 1");
}

// ── Scenario: df tracks per-file occurrences across modules ─────────

#[test]
fn given_term_in_three_files_across_two_modules_when_extracted_then_df_is_three() {
    let rows = vec![
        make_row("mod_a/shared.rs", "mod_a", "Rust", 50),
        make_row("mod_a/shared_util.rs", "mod_a", "Rust", 50),
        make_row("mod_b/shared.rs", "mod_b", "Rust", 50),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);
    let shared = clouds.overall.iter().find(|t| t.term == "shared");
    assert!(shared.is_some());
    assert_eq!(
        shared.unwrap().df,
        3,
        "df should count all files containing the term"
    );
}

// ── Scenario: determinism with multi-module input ───────────────────

#[test]
fn given_multi_module_input_when_extracted_twice_then_identical_output() {
    let make = || {
        let rows = vec![
            make_row("svc/auth/login.rs", "svc/auth", "Rust", 100),
            make_row("svc/auth/logout.rs", "svc/auth", "Rust", 50),
            make_row("svc/billing/invoice.rs", "svc/billing", "Rust", 200),
            make_row("svc/billing/payment.rs", "svc/billing", "Rust", 150),
            make_row("lib/utils/format.rs", "lib/utils", "Rust", 30),
        ];
        make_export(rows, vec!["svc", "lib"])
    };

    let a = build_topic_clouds(&make());
    let b = build_topic_clouds(&make());

    assert_eq!(a.overall.len(), b.overall.len());
    for (ta, tb) in a.overall.iter().zip(b.overall.iter()) {
        assert_eq!(ta.term, tb.term);
        assert_eq!(ta.tf, tb.tf);
        assert_eq!(ta.df, tb.df);
        assert!((ta.score - tb.score).abs() < f64::EPSILON);
    }
    assert_eq!(
        a.per_module.keys().collect::<Vec<_>>(),
        b.per_module.keys().collect::<Vec<_>>()
    );
}

// ── Scenario: overall truncation to TOP_K=8 ─────────────────────────

#[test]
fn given_many_unique_terms_across_modules_when_extracted_then_overall_at_most_8() {
    let rows: Vec<FileRow> = (0..30)
        .map(|i| {
            make_row(
                &format!("mod_{}/unique_term_{}.rs", i % 5, i),
                &format!("mod_{}", i % 5),
                "Rust",
                50,
            )
        })
        .collect();
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);
    assert!(
        clouds.overall.len() <= 8,
        "overall should be truncated to 8, got {}",
        clouds.overall.len()
    );
}
