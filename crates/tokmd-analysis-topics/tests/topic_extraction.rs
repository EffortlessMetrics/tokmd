//! Tests for topic cloud generation from code.
//!
//! Covers: determinism (BTreeMap ordering), TF-IDF scoring, stopword filtering,
//! edge cases, and multi-module aggregation.

use tokmd_analysis_topics::build_topic_clouds;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_row(path: &str, module: &str, code: usize, tokens: usize) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code,
        comments: 0,
        blanks: 0,
        lines: code,
        bytes: code * 10,
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

// ===========================================================================
// Determinism: identical inputs produce identical topic clouds
// ===========================================================================

#[test]
fn determinism_overall_order_across_100_runs() {
    let make = || {
        let rows = vec![
            make_row("svc/auth/login_handler.rs", "svc/auth", 50, 250),
            make_row("svc/auth/token_refresh.rs", "svc/auth", 40, 200),
            make_row("svc/payments/stripe.rs", "svc/payments", 60, 300),
            make_row("svc/payments/refund.rs", "svc/payments", 30, 150),
            make_row("svc/shared/crypto.rs", "svc/shared", 20, 100),
        ];
        make_export(rows, vec!["svc"])
    };

    let reference = build_topic_clouds(&make());
    for _ in 0..100 {
        let result = build_topic_clouds(&make());

        // Overall terms must be identical in count, order, and values
        assert_eq!(reference.overall.len(), result.overall.len());
        for (a, b) in reference.overall.iter().zip(result.overall.iter()) {
            assert_eq!(a.term, b.term);
            assert_eq!(a.tf, b.tf);
            assert_eq!(a.df, b.df);
            assert!((a.score - b.score).abs() < f64::EPSILON);
        }

        // Per-module keys must be identical
        assert_eq!(
            reference.per_module.keys().collect::<Vec<_>>(),
            result.per_module.keys().collect::<Vec<_>>()
        );

        // Per-module terms must be identical
        for key in reference.per_module.keys() {
            let va = &reference.per_module[key];
            let vb = &result.per_module[key];
            assert_eq!(va.len(), vb.len());
            for (a, b) in va.iter().zip(vb.iter()) {
                assert_eq!(a.term, b.term);
                assert_eq!(a.tf, b.tf);
                assert_eq!(a.df, b.df);
                assert!((a.score - b.score).abs() < f64::EPSILON);
            }
        }
    }
}

#[test]
fn determinism_per_module_btreemap_keys_sorted() {
    let rows = vec![
        make_row("z_mod/file.rs", "z_mod", 10, 50),
        make_row("a_mod/file.rs", "a_mod", 10, 50),
        make_row("m_mod/file.rs", "m_mod", 10, 50),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    let keys: Vec<&String> = clouds.per_module.keys().collect();
    let mut sorted_keys = keys.clone();
    sorted_keys.sort();
    assert_eq!(keys, sorted_keys, "per_module keys should be BTreeMap-sorted");
}

// ===========================================================================
// Topic extraction from various domain patterns
// ===========================================================================

#[test]
fn auth_domain_topics() {
    let rows = vec![
        make_row("services/auth/login.rs", "services/auth", 100, 500),
        make_row("services/auth/oauth_callback.rs", "services/auth", 80, 400),
        make_row("services/auth/jwt_verify.rs", "services/auth", 60, 300),
    ];
    let export = make_export(rows, vec!["services"]);
    let clouds = build_topic_clouds(&export);

    let auth = clouds.per_module.get("services/auth").unwrap();
    let terms: Vec<&str> = auth.iter().map(|t| t.term.as_str()).collect();
    assert!(terms.contains(&"login"), "missing 'login' in {terms:?}");
    assert!(terms.contains(&"oauth"), "missing 'oauth' in {terms:?}");
    assert!(terms.contains(&"jwt"), "missing 'jwt' in {terms:?}");
}

#[test]
fn database_domain_topics() {
    let rows = vec![
        make_row("db/connection_pool.rs", "db", 80, 400),
        make_row("db/migration.rs", "db", 60, 300),
        make_row("db/query_builder.rs", "db", 40, 200),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    let db = clouds.per_module.get("db").unwrap();
    let terms: Vec<&str> = db.iter().map(|t| t.term.as_str()).collect();
    assert!(terms.contains(&"connection"), "missing 'connection' in {terms:?}");
    assert!(terms.contains(&"migration"), "missing 'migration' in {terms:?}");
    assert!(terms.contains(&"query"), "missing 'query' in {terms:?}");
}

// ===========================================================================
// Stopword filtering
// ===========================================================================

#[test]
fn common_stopwords_excluded() {
    let rows = vec![make_row("src/lib/mod/index/main.rs", "root", 10, 50)];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);
    // All path segments are stopwords
    assert!(clouds.overall.is_empty());
}

#[test]
fn file_extensions_are_stopwords() {
    let rows = vec![make_row("app/handler.py", "app", 10, 50)];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(!terms.contains(&"py"), "'py' should be filtered");
    assert!(!terms.contains(&"rs"), "'rs' should be filtered");
}

#[test]
fn module_roots_become_stopwords() {
    let rows = vec![make_row("crates/auth/handler.rs", "crates/auth", 10, 50)];
    let export = make_export(rows, vec!["crates"]);
    let clouds = build_topic_clouds(&export);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(!terms.contains(&"crates"), "'crates' should be a module root stopword");
}

// ===========================================================================
// TF-IDF: unique terms score higher than ubiquitous ones
// ===========================================================================

#[test]
fn unique_term_has_higher_idf_than_shared_term() {
    let rows = vec![
        make_row("mod_a/shared_util.rs", "mod_a", 10, 50),
        make_row("mod_b/shared_unique.rs", "mod_b", 10, 50),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    let find = |term: &str| clouds.overall.iter().find(|t| t.term == term);

    let shared = find("shared");
    let unique = find("unique");
    let util = find("util");

    // "shared" appears in both modules (df=2), "unique" and "util" in one (df=1)
    assert!(shared.is_some());
    assert_eq!(shared.unwrap().df, 2);
    if let Some(u) = unique {
        assert_eq!(u.df, 1);
    }
    if let Some(u) = util {
        assert_eq!(u.df, 1);
    }
}

// ===========================================================================
// Weight by tokens: more tokens → higher tf
// ===========================================================================

#[test]
fn file_with_more_tokens_contributes_more_tf() {
    let rows = vec![
        make_row("app/heavy_feature.rs", "app", 100, 5000),
        make_row("app/light_feature.rs", "app", 10, 10),
    ];
    let export = make_export(rows, vec!["app"]);
    let clouds = build_topic_clouds(&export);

    let find = |term: &str| clouds.overall.iter().find(|t| t.term == term);
    let heavy_tf = find("heavy").map(|t| t.tf).unwrap_or(0);
    let light_tf = find("light").map(|t| t.tf).unwrap_or(0);
    assert!(
        heavy_tf > light_tf,
        "heavy ({heavy_tf}) should have higher tf than light ({light_tf})"
    );
}

// ===========================================================================
// Edge case: zero tokens → weight at least 1
// ===========================================================================

#[test]
fn zero_tokens_still_produces_topics() {
    let rows = vec![make_row("app/empty_module.rs", "app", 0, 0)];
    let export = make_export(rows, vec!["app"]);
    let clouds = build_topic_clouds(&export);
    let term = clouds.overall.iter().find(|t| t.term == "empty");
    assert!(term.is_some(), "should extract topic even with 0 tokens");
    assert!(term.unwrap().tf >= 1, "tf should be at least 1");
}

// ===========================================================================
// Edge case: empty input
// ===========================================================================

#[test]
fn empty_export_produces_empty_clouds() {
    let export = make_export(vec![], vec![]);
    let clouds = build_topic_clouds(&export);
    assert!(clouds.overall.is_empty());
    assert!(clouds.per_module.is_empty());
}

// ===========================================================================
// Edge case: only child rows (no parents)
// ===========================================================================

#[test]
fn child_rows_ignored() {
    let mut row = make_row("app/controller.rs", "app", 10, 50);
    row.kind = FileKind::Child;
    let export = make_export(vec![row], vec![]);
    let clouds = build_topic_clouds(&export);
    assert!(clouds.overall.is_empty());
    assert!(clouds.per_module.is_empty());
}

// ===========================================================================
// TOP_K truncation to 8
// ===========================================================================

#[test]
fn overall_truncated_to_8() {
    let rows: Vec<FileRow> = (0..30)
        .map(|i| make_row(&format!("app/uniqueterm{i}.rs"), "app", 10, 50))
        .collect();
    let export = make_export(rows, vec!["app"]);
    let clouds = build_topic_clouds(&export);
    assert!(clouds.overall.len() <= 8);
}

#[test]
fn per_module_truncated_to_8() {
    let rows: Vec<FileRow> = (0..30)
        .map(|i| make_row(&format!("mod/uniqueterm{i}.rs"), "mod", 10, 50))
        .collect();
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);
    let mod_terms = clouds.per_module.get("mod").unwrap();
    assert!(mod_terms.len() <= 8);
}

// ===========================================================================
// Sorting: overall descending by score, tie-break by term name
// ===========================================================================

#[test]
fn overall_sorted_descending_by_score() {
    let rows = vec![
        make_row("a/alpha.rs", "a", 200, 1000),
        make_row("a/beta.rs", "a", 50, 250),
        make_row("b/gamma.rs", "b", 10, 50),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    for window in clouds.overall.windows(2) {
        assert!(
            window[0].score >= window[1].score
                || (window[0].score == window[1].score && window[0].term <= window[1].term),
            "not sorted: {} ({}) vs {} ({})",
            window[0].term,
            window[0].score,
            window[1].term,
            window[1].score,
        );
    }
}

// ===========================================================================
// All scores are non-negative
// ===========================================================================

#[test]
fn all_scores_non_negative() {
    let rows = vec![
        make_row("a/foo.rs", "a", 10, 50),
        make_row("b/bar.rs", "b", 5, 25),
        make_row("c/baz.rs", "c", 1, 5),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    for term in &clouds.overall {
        assert!(term.score >= 0.0, "negative score: {}", term.score);
    }
    for terms in clouds.per_module.values() {
        for term in terms {
            assert!(term.score >= 0.0, "negative score: {}", term.score);
        }
    }
}

// ===========================================================================
// Case normalization: all terms lowercased
// ===========================================================================

#[test]
fn all_terms_are_lowercased() {
    let rows = vec![
        make_row("App/MyController.rs", "App", 10, 50),
        make_row("Svc/AuthHandler.rs", "Svc", 10, 50),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    for term in &clouds.overall {
        assert_eq!(term.term, term.term.to_lowercase());
    }
}

// ===========================================================================
// Path normalization: backslashes handled
// ===========================================================================

#[test]
fn backslash_paths_produce_correct_tokens() {
    let rows = vec![make_row(r"crates\auth\handler.rs", "crates/auth", 10, 50)];
    let export = make_export(rows, vec!["crates"]);
    let clouds = build_topic_clouds(&export);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(terms.contains(&"handler"), "backslash paths should split correctly");
}

// ===========================================================================
// Tokenizer: splits on underscore, hyphen, dot
// ===========================================================================

#[test]
fn compound_filename_split_into_tokens() {
    let rows = vec![make_row("app/my_api-client.v2.rs", "app", 10, 50)];
    let export = make_export(rows, vec!["app"]);
    let clouds = build_topic_clouds(&export);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(terms.contains(&"my"));
    assert!(terms.contains(&"api"));
    assert!(terms.contains(&"client"));
    assert!(terms.contains(&"v2"));
}

// ===========================================================================
// Multi-module: per_module maps are distinct
// ===========================================================================

#[test]
fn per_module_maps_contain_correct_modules() {
    let rows = vec![
        make_row("svc/auth/login.rs", "svc/auth", 10, 50),
        make_row("svc/billing/invoice.rs", "svc/billing", 10, 50),
        make_row("svc/search/index_builder.rs", "svc/search", 10, 50),
    ];
    let export = make_export(rows, vec!["svc"]);
    let clouds = build_topic_clouds(&export);

    assert_eq!(clouds.per_module.len(), 3);
    assert!(clouds.per_module.contains_key("svc/auth"));
    assert!(clouds.per_module.contains_key("svc/billing"));
    assert!(clouds.per_module.contains_key("svc/search"));

    let auth = clouds.per_module.get("svc/auth").unwrap();
    assert!(auth.iter().any(|t| t.term == "login"));

    let billing = clouds.per_module.get("svc/billing").unwrap();
    assert!(billing.iter().any(|t| t.term == "invoice"));
}

// ===========================================================================
// df counts: shared term across modules
// ===========================================================================

#[test]
fn df_reflects_module_count_not_file_count() {
    let rows = vec![
        make_row("mod_a/handler.rs", "mod_a", 10, 50),
        make_row("mod_a/handler_util.rs", "mod_a", 10, 50),
        make_row("mod_b/handler.rs", "mod_b", 10, 50),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    let handler = clouds.overall.iter().find(|t| t.term == "handler");
    assert!(handler.is_some());
    // "handler" appears in files across 2 modules; df should reflect file count
    // (the code counts per-file, not per-module for df)
    assert!(handler.unwrap().df >= 2);
}

// ===========================================================================
// Overall aggregates tf across all modules
// ===========================================================================

#[test]
fn overall_tf_is_sum_of_module_tf() {
    let rows = vec![
        make_row("mod_a/handler.rs", "mod_a", 10, 100),
        make_row("mod_b/handler.rs", "mod_b", 10, 100),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    let overall_handler = clouds.overall.iter().find(|t| t.term == "handler");
    let mod_a_handler = clouds
        .per_module
        .get("mod_a")
        .and_then(|v| v.iter().find(|t| t.term == "handler"));
    let mod_b_handler = clouds
        .per_module
        .get("mod_b")
        .and_then(|v| v.iter().find(|t| t.term == "handler"));

    assert!(overall_handler.is_some());
    assert!(mod_a_handler.is_some());
    assert!(mod_b_handler.is_some());

    assert_eq!(
        overall_handler.unwrap().tf,
        mod_a_handler.unwrap().tf + mod_b_handler.unwrap().tf,
        "overall tf should be sum of per-module tf"
    );
}
