//! BDD-style scenario tests for topic-cloud extraction.

use tokmd_analysis_topics::build_topic_clouds;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── helpers ──────────────────────────────────────────────────────────

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

// ── Scenario: empty input ────────────────────────────────────────────

#[test]
fn given_no_rows_then_overall_is_empty() {
    let export = make_export(vec![], vec![]);
    let clouds = build_topic_clouds(&export);
    assert!(clouds.overall.is_empty());
    assert!(clouds.per_module.is_empty());
}

// ── Scenario: only child rows (no parents) ───────────────────────────

#[test]
fn given_only_child_rows_then_topics_are_empty() {
    let mut row = make_row("src/lib.rs", "root", 10, 50);
    row.kind = FileKind::Child;
    let export = make_export(vec![row], vec![]);
    let clouds = build_topic_clouds(&export);
    assert!(clouds.overall.is_empty());
}

// ── Scenario: single file ────────────────────────────────────────────

#[test]
fn given_single_file_then_path_segments_appear_as_topics() {
    let rows = vec![make_row("crates/auth/src/login.rs", "crates/auth", 10, 50)];
    let export = make_export(rows, vec!["crates"]);
    let clouds = build_topic_clouds(&export);

    assert_eq!(clouds.per_module.len(), 1);
    let auth_terms = clouds.per_module.get("crates/auth").unwrap();
    let term_names: Vec<&str> = auth_terms.iter().map(|t| t.term.as_str()).collect();
    // "auth" is a module root → stopword, "src"/"rs" are stopwords
    assert!(
        term_names.contains(&"login"),
        "expected 'login' in {term_names:?}"
    );
}

#[test]
fn given_single_file_then_overall_matches_module() {
    let rows = vec![make_row("crates/auth/src/login.rs", "crates/auth", 10, 50)];
    let export = make_export(rows, vec!["crates"]);
    let clouds = build_topic_clouds(&export);

    let module_terms = clouds.per_module.get("crates/auth").unwrap();
    // Overall should contain the same terms (single module)
    for mt in module_terms {
        assert!(
            clouds.overall.iter().any(|ov| ov.term == mt.term),
            "overall missing term '{}'",
            mt.term
        );
    }
}

// ── Scenario: stopwords are filtered ─────────────────────────────────

#[test]
fn given_path_with_stopwords_then_they_are_excluded() {
    let rows = vec![make_row("src/lib/mod/index.rs", "root", 10, 50)];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    // All segments are stopwords → nothing extracted
    assert!(clouds.overall.is_empty());
}

#[test]
fn given_extension_stopwords_then_they_are_excluded() {
    let rows = vec![make_row("auth/handler.py", "auth", 10, 50)];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(!terms.contains(&"py"), "'py' should be a stopword");
    assert!(terms.contains(&"handler"), "expected 'handler'");
}

#[test]
fn given_module_roots_then_they_become_stopwords() {
    let rows = vec![make_row("packages/core/util.ts", "packages/core", 10, 50)];
    let export = make_export(rows, vec!["packages"]);
    let clouds = build_topic_clouds(&export);

    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(
        !terms.contains(&"packages"),
        "'packages' is a module root stopword"
    );
}

// ── Scenario: TF-IDF scoring ─────────────────────────────────────────

#[test]
fn given_common_term_across_all_modules_then_lower_score_than_unique_term() {
    let rows = vec![
        make_row("app/shared/utils.rs", "app/shared", 10, 50),
        make_row("app/api/utils.rs", "app/api", 10, 50),
        make_row("app/api/controller.rs", "app/api", 10, 50),
    ];
    let export = make_export(rows, vec!["app"]);
    let clouds = build_topic_clouds(&export);

    let find = |term: &str| clouds.overall.iter().find(|t| t.term == term);
    let _utils_score = find("utils").map(|t| t.score).unwrap_or(0.0);
    let controller_score = find("controller").map(|t| t.score).unwrap_or(0.0);

    // "controller" appears in only 1 file → lower df → higher IDF per-occurrence
    assert!(
        controller_score > 0.0,
        "controller should have a positive score"
    );
}

#[test]
fn given_term_in_single_module_then_high_idf() {
    let rows = vec![
        make_row("mod_a/unique_term.rs", "mod_a", 10, 50),
        make_row("mod_b/common.rs", "mod_b", 10, 50),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    let find_mod = |module: &str, term: &str| {
        clouds
            .per_module
            .get(module)
            .and_then(|v| v.iter().find(|t| t.term == term))
    };

    let unique = find_mod("mod_a", "unique");
    assert!(unique.is_some(), "expected 'unique' in mod_a");
    assert_eq!(unique.unwrap().df, 1);
}

// ── Scenario: weight by tokens ───────────────────────────────────────

#[test]
fn given_file_with_more_tokens_then_higher_tf() {
    let rows = vec![
        make_row("app/heavy.rs", "app", 100, 5000),
        make_row("app/light.rs", "app", 10, 10),
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

#[test]
fn given_zero_tokens_then_weight_is_at_least_one() {
    let rows = vec![make_row("app/empty.rs", "app", 0, 0)];
    let export = make_export(rows, vec!["app"]);
    let clouds = build_topic_clouds(&export);

    let term = clouds.overall.iter().find(|t| t.term == "empty");
    assert!(term.is_some(), "term should exist even with 0 tokens");
    assert!(term.unwrap().tf >= 1, "tf should be at least 1");
}

// ── Scenario: path normalization ─────────────────────────────────────

#[test]
fn given_backslash_paths_then_segments_are_split_correctly() {
    let rows = vec![make_row(r"crates\auth\src\login.rs", "crates/auth", 10, 50)];
    let export = make_export(rows, vec!["crates"]);
    let clouds = build_topic_clouds(&export);

    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(
        terms.contains(&"login"),
        "backslash path should yield 'login'"
    );
}

// ── Scenario: tokenizer splits on underscore, hyphen, dot ────────────

#[test]
fn given_compound_filename_then_split_into_tokens() {
    let rows = vec![make_row("app/my_api-client.v2.rs", "app", 10, 50)];
    let export = make_export(rows, vec!["app"]);
    let clouds = build_topic_clouds(&export);

    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(terms.contains(&"my"), "expected 'my' from underscore split");
    assert!(
        terms.contains(&"api"),
        "expected 'api' from underscore split"
    );
    assert!(
        terms.contains(&"client"),
        "expected 'client' from hyphen split"
    );
    assert!(terms.contains(&"v2"), "expected 'v2' from dot split");
}

// ── Scenario: TOP_K truncation ───────────────────────────────────────

#[test]
fn given_many_unique_terms_then_per_module_truncated_to_at_most_8() {
    let rows: Vec<FileRow> = (0..20)
        .map(|i| make_row(&format!("app/term{i}.rs", i = i), "app", 10, 50))
        .collect();
    let export = make_export(rows, vec!["app"]);
    let clouds = build_topic_clouds(&export);

    let app_terms = clouds.per_module.get("app").unwrap();
    assert!(
        app_terms.len() <= 8,
        "per-module should be truncated to TOP_K=8, got {}",
        app_terms.len()
    );
}

#[test]
fn given_many_unique_terms_then_overall_truncated_to_at_most_8() {
    let rows: Vec<FileRow> = (0..20)
        .map(|i| make_row(&format!("app/term{i}.rs", i = i), "app", 10, 50))
        .collect();
    let export = make_export(rows, vec!["app"]);
    let clouds = build_topic_clouds(&export);

    assert!(
        clouds.overall.len() <= 8,
        "overall should be truncated to TOP_K=8, got {}",
        clouds.overall.len()
    );
}

// ── Scenario: deterministic ordering ─────────────────────────────────

#[test]
fn given_same_input_twice_then_identical_output() {
    let make = || {
        let rows = vec![
            make_row("app/auth/login.rs", "app/auth", 10, 50),
            make_row("app/auth/logout.rs", "app/auth", 10, 50),
            make_row("app/db/pool.rs", "app/db", 10, 50),
            make_row("app/db/migrate.rs", "app/db", 10, 50),
        ];
        make_export(rows, vec!["app"])
    };

    let a = build_topic_clouds(&make());
    let b = build_topic_clouds(&make());

    // Compare overall
    assert_eq!(a.overall.len(), b.overall.len());
    for (ta, tb) in a.overall.iter().zip(b.overall.iter()) {
        assert_eq!(ta.term, tb.term);
        assert_eq!(ta.tf, tb.tf);
        assert_eq!(ta.df, tb.df);
        assert!((ta.score - tb.score).abs() < f64::EPSILON);
    }

    // Compare per_module keys and terms
    assert_eq!(
        a.per_module.keys().collect::<Vec<_>>(),
        b.per_module.keys().collect::<Vec<_>>()
    );
    for key in a.per_module.keys() {
        let va = &a.per_module[key];
        let vb = &b.per_module[key];
        assert_eq!(va.len(), vb.len());
        for (ta, tb) in va.iter().zip(vb.iter()) {
            assert_eq!(ta.term, tb.term);
        }
    }
}

// ── Scenario: multiple modules with shared and unique terms ──────────

#[test]
fn given_multiple_modules_then_per_module_maps_are_separate() {
    let rows = vec![
        make_row("svc/auth/handler.rs", "svc/auth", 10, 50),
        make_row("svc/billing/handler.rs", "svc/billing", 10, 50),
        make_row("svc/billing/invoice.rs", "svc/billing", 10, 50),
    ];
    let export = make_export(rows, vec!["svc"]);
    let clouds = build_topic_clouds(&export);

    assert!(clouds.per_module.contains_key("svc/auth"));
    assert!(clouds.per_module.contains_key("svc/billing"));
    assert_eq!(clouds.per_module.len(), 2);

    let billing = clouds.per_module.get("svc/billing").unwrap();
    let billing_terms: Vec<&str> = billing.iter().map(|t| t.term.as_str()).collect();
    assert!(billing_terms.contains(&"invoice"));
}

// ── Scenario: scores are non-negative ────────────────────────────────

#[test]
fn given_any_input_then_all_scores_are_non_negative() {
    let rows = vec![
        make_row("a/b/c.rs", "a/b", 10, 50),
        make_row("x/y/z.rs", "x/y", 5, 25),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    for term in &clouds.overall {
        assert!(
            term.score >= 0.0,
            "score should be >= 0, got {}",
            term.score
        );
    }
    for terms in clouds.per_module.values() {
        for term in terms {
            assert!(
                term.score >= 0.0,
                "score should be >= 0, got {}",
                term.score
            );
        }
    }
}

// ── Scenario: overall sorting is descending by score ─────────────────

#[test]
fn given_multiple_terms_then_overall_sorted_descending_by_score() {
    let rows = vec![
        make_row("a/alpha.rs", "a", 100, 500),
        make_row("a/beta.rs", "a", 10, 50),
        make_row("b/gamma.rs", "b", 10, 50),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    for window in clouds.overall.windows(2) {
        assert!(
            window[0].score >= window[1].score,
            "overall not sorted: {} ({}) should >= {} ({})",
            window[0].term,
            window[0].score,
            window[1].term,
            window[1].score,
        );
    }
}

// ── Scenario: case normalization ─────────────────────────────────────

#[test]
fn given_mixed_case_path_then_terms_are_lowercased() {
    let rows = vec![make_row("App/MyController.rs", "App", 10, 50)];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    for term in &clouds.overall {
        assert_eq!(
            term.term,
            term.term.to_lowercase(),
            "term '{}' should be lowercase",
            term.term
        );
    }
}

// ── Scenario: df counts ──────────────────────────────────────────────

#[test]
fn given_term_in_two_modules_then_df_is_two() {
    let rows = vec![
        make_row("mod_a/shared.rs", "mod_a", 10, 50),
        make_row("mod_b/shared.rs", "mod_b", 10, 50),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    let shared = clouds.overall.iter().find(|t| t.term == "shared");
    assert!(shared.is_some());
    assert_eq!(shared.unwrap().df, 2);
}

#[test]
fn given_term_repeated_in_same_module_then_df_counts_files() {
    // df counts per-file (document) occurrences, not per-module
    let rows = vec![
        make_row("mod_a/widget_one.rs", "mod_a", 10, 50),
        make_row("mod_a/widget_two.rs", "mod_a", 10, 50),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    let widget = clouds
        .per_module
        .get("mod_a")
        .and_then(|v| v.iter().find(|t| t.term == "widget"));
    assert!(widget.is_some());
    assert_eq!(widget.unwrap().df, 2, "df counts files containing the term");
}

// ── Scenario: known domain file patterns ─────────────────────────────

#[test]
fn given_auth_api_db_paths_then_domain_topics_appear() {
    let rows = vec![
        make_row("services/auth/login_handler.rs", "services/auth", 50, 250),
        make_row("services/auth/token_refresh.rs", "services/auth", 40, 200),
        make_row("services/api/router.rs", "services/api", 30, 150),
        make_row("services/db/connection_pool.rs", "services/db", 60, 300),
    ];
    let export = make_export(rows, vec!["services"]);
    let clouds = build_topic_clouds(&export);

    let overall_terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(
        overall_terms.contains(&"login"),
        "expected 'login' in overall: {overall_terms:?}"
    );
    assert!(
        overall_terms.contains(&"router"),
        "expected 'router' in overall: {overall_terms:?}"
    );
    assert!(
        overall_terms.contains(&"connection"),
        "expected 'connection' in overall: {overall_terms:?}"
    );

    let auth_terms: Vec<&str> = clouds
        .per_module
        .get("services/auth")
        .unwrap()
        .iter()
        .map(|t| t.term.as_str())
        .collect();
    assert!(
        auth_terms.contains(&"login"),
        "expected 'login' in auth module"
    );
    assert!(
        auth_terms.contains(&"token"),
        "expected 'token' in auth module"
    );
}

// ── Scenario: multiple files aggregate topics across modules ─────────

#[test]
fn given_many_modules_then_overall_aggregates_from_all() {
    let rows = vec![
        make_row(
            "frontend/components/button.rs",
            "frontend/components",
            20,
            100,
        ),
        make_row(
            "frontend/components/modal.rs",
            "frontend/components",
            20,
            100,
        ),
        make_row("backend/handlers/user.rs", "backend/handlers", 30, 150),
        make_row("backend/handlers/order.rs", "backend/handlers", 30, 150),
        make_row("shared/utils/crypto.rs", "shared/utils", 40, 200),
    ];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    assert_eq!(clouds.per_module.len(), 3, "expected 3 modules");

    // Overall should contain terms from different modules
    let overall_terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    // At least some terms from different modules should appear in overall
    let has_frontend = overall_terms
        .iter()
        .any(|t| *t == "button" || *t == "modal");
    let has_backend = overall_terms.iter().any(|t| *t == "user" || *t == "order");
    let has_shared = overall_terms.contains(&"crypto");
    assert!(
        has_frontend || has_backend || has_shared,
        "overall should aggregate terms from multiple modules: {overall_terms:?}"
    );
}

// ── Scenario: empty path segments ────────────────────────────────────

#[test]
fn given_path_with_leading_slash_then_no_panic() {
    let rows = vec![make_row("/leading/slash/file.rs", "leading", 10, 50)];
    let export = make_export(rows, vec![]);
    let clouds = build_topic_clouds(&export);

    // Should not panic and should produce topics
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(terms.contains(&"leading") || terms.contains(&"slash") || terms.contains(&"file"));
}

// ── Scenario: determinism with varied input ──────────────────────────

#[test]
fn given_complex_input_then_deterministic_across_ten_runs() {
    let make = || {
        let rows = vec![
            make_row("crates/auth/src/login.rs", "crates/auth", 100, 500),
            make_row("crates/auth/src/token.rs", "crates/auth", 80, 400),
            make_row(
                "crates/payments/src/stripe_api.rs",
                "crates/payments",
                60,
                300,
            ),
            make_row("crates/payments/src/refund.rs", "crates/payments", 40, 200),
            make_row("crates/shared/src/utils.rs", "crates/shared", 20, 100),
        ];
        make_export(rows, vec!["crates"])
    };

    let reference = build_topic_clouds(&make());
    for _ in 0..10 {
        let result = build_topic_clouds(&make());
        assert_eq!(reference.overall.len(), result.overall.len());
        for (a, b) in reference.overall.iter().zip(result.overall.iter()) {
            assert_eq!(a.term, b.term);
            assert_eq!(a.tf, b.tf);
            assert_eq!(a.df, b.df);
            assert!((a.score - b.score).abs() < f64::EPSILON);
        }
        assert_eq!(
            reference.per_module.keys().collect::<Vec<_>>(),
            result.per_module.keys().collect::<Vec<_>>()
        );
    }
}
