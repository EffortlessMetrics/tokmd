//! Wave-56 depth tests for `tokmd-analysis-topics`.
//!
//! Covers topic extraction, cloud generation, keyword frequency analysis,
//! deduplication/normalization, language-specific detection, edge cases,
//! and deterministic output guarantees.

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

fn row_lang(path: &str, module: &str, lang: &str, tokens: usize) -> FileRow {
    FileRow {
        lang: lang.to_string(),
        ..row(path, module, tokens)
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
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ── 1. Topic extraction from code file paths ────────────────────

#[test]
fn extracts_terms_from_simple_paths() {
    let data = export(
        vec![
            row("api/auth/handler.rs", "api/auth", 100),
            row("api/auth/middleware.rs", "api/auth", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let module_topics = clouds.per_module.get("api/auth").unwrap();
    let terms: Vec<&str> = module_topics.iter().map(|t| t.term.as_str()).collect();
    assert!(terms.contains(&"api"));
    assert!(terms.contains(&"auth"));
}

#[test]
fn extracts_terms_from_nested_directory_structure() {
    let data = export(
        vec![
            row(
                "services/payment/stripe/checkout.rs",
                "services/payment",
                200,
            ),
            row("services/payment/stripe/refund.rs", "services/payment", 150),
            row(
                "services/payment/paypal/process.rs",
                "services/payment",
                100,
            ),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let overall_terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(overall_terms.contains(&"payment"));
    assert!(overall_terms.contains(&"stripe"));
}

#[test]
fn extracts_terms_split_on_underscores_and_hyphens() {
    let data = export(
        vec![row("user_profile/account-settings.rs", "user_profile", 100)],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(terms.contains(&"user"));
    assert!(terms.contains(&"profile"));
    assert!(terms.contains(&"account"));
    assert!(terms.contains(&"settings"));
}

// ── 2. Topic cloud generation ───────────────────────────────────

#[test]
fn overall_cloud_aggregates_across_modules() {
    let data = export(
        vec![
            row("api/router.rs", "api", 100),
            row("db/connection.rs", "db", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    assert_eq!(clouds.per_module.len(), 2);
    assert!(!clouds.overall.is_empty());
}

#[test]
fn per_module_cloud_limited_to_top_k() {
    // Create many distinct path segments to exceed TOP_K=8
    let rows: Vec<FileRow> = (0..20)
        .map(|i| row(&format!("mod/unique_segment_{i}.rs"), "mod", 50))
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);
    let mod_terms = clouds.per_module.get("mod").unwrap();
    assert!(
        mod_terms.len() <= 8,
        "per-module terms should be capped at TOP_K=8"
    );
}

#[test]
fn overall_cloud_also_limited_to_top_k() {
    let rows: Vec<FileRow> = (0..30)
        .map(|i| row(&format!("m/distinct_word_{i}.rs"), "m", 50))
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);
    assert!(clouds.overall.len() <= 8);
}

#[test]
fn modules_with_higher_token_weights_rank_higher() {
    let data = export(
        vec![
            row("web/heavy_controller.rs", "web", 10000),
            row("web/light_util.rs", "web", 1),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let terms = clouds.per_module.get("web").unwrap();
    // "heavy" should have higher tf*idf because its file has many more tokens
    let heavy = terms.iter().find(|t| t.term == "heavy");
    let light = terms.iter().find(|t| t.term == "light");
    if let (Some(h), Some(l)) = (heavy, light) {
        assert!(h.tf > l.tf, "heavy file should have higher tf");
    }
}

// ── 3. Keyword frequency analysis ──────────────────────────────

#[test]
fn term_frequency_accumulates_across_files_in_module() {
    let data = export(
        vec![
            row("core/parser.rs", "core", 100),
            row("core/parser_util.rs", "core", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let terms = clouds.per_module.get("core").unwrap();
    let parser = terms.iter().find(|t| t.term == "parser").unwrap();
    // "parser" appears in both paths, so tf should reflect combined weight
    assert!(
        parser.tf >= 200,
        "tf should accumulate weights from both files"
    );
}

#[test]
fn document_frequency_counts_distinct_files() {
    let data = export(
        vec![
            row("a/shared_name.rs", "a", 50),
            row("b/shared_name.rs", "b", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let shared = clouds.overall.iter().find(|t| t.term == "shared").unwrap();
    assert_eq!(shared.df, 2, "term in two files should have df=2");
}

#[test]
fn idf_penalizes_ubiquitous_terms() {
    let data = export(
        vec![
            row("a/common.rs", "a", 100),
            row("b/common.rs", "b", 100),
            row("c/common.rs", "c", 100),
            row("a/unique.rs", "a", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let common = clouds.overall.iter().find(|t| t.term == "common");
    let unique = clouds.overall.iter().find(|t| t.term == "unique");
    if let (Some(c), Some(u)) = (common, unique) {
        // "unique" appears in only one module, so its IDF should be higher
        assert!(u.df < c.df);
    }
}

#[test]
fn score_is_positive_for_valid_terms() {
    let data = export(vec![row("project/feature.rs", "project", 100)], &[]);
    let clouds = build_topic_clouds(&data);
    for term in &clouds.overall {
        assert!(term.score > 0.0, "score should be positive: {}", term.term);
    }
}

// ── 4. Topic deduplication and normalization ────────────────────

#[test]
fn terms_are_lowercased() {
    let data = export(vec![row("Modules/MyController.rs", "Modules", 100)], &[]);
    let clouds = build_topic_clouds(&data);
    for term in &clouds.overall {
        assert_eq!(
            term.term,
            term.term.to_lowercase(),
            "term should be lowercase"
        );
    }
}

#[test]
fn stopwords_are_filtered_out() {
    let data = export(
        vec![
            row("src/lib/index.rs", "src/lib", 100),
            row("src/test/main.rs", "src/test", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(!terms.contains(&"src"), "src should be a stopword");
    assert!(!terms.contains(&"lib"), "lib should be a stopword");
    assert!(!terms.contains(&"test"), "test should be a stopword");
    assert!(!terms.contains(&"main"), "main should be a stopword");
}

#[test]
fn file_extensions_are_filtered_as_stopwords() {
    let data = export(vec![row("code/widget.rs", "code", 100)], &[]);
    let clouds = build_topic_clouds(&data);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(
        !terms.contains(&"rs"),
        "file extensions should be stopwords"
    );
}

#[test]
fn module_roots_are_filtered_as_stopwords() {
    let data = export(
        vec![row("crates/auth/handler.rs", "crates/auth", 100)],
        &["crates"],
    );
    let clouds = build_topic_clouds(&data);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(
        !terms.contains(&"crates"),
        "module roots should be stopwords"
    );
}

#[test]
fn backslash_paths_are_normalized() {
    let data = export(vec![row("api\\v2\\endpoint.rs", "api\\v2", 100)], &[]);
    let clouds = build_topic_clouds(&data);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(terms.contains(&"api") || terms.contains(&"v2") || terms.contains(&"endpoint"));
}

// ── 5. Language-specific topic detection ────────────────────────

#[test]
fn python_extension_filtered_as_stopword() {
    let data = export(
        vec![row_lang("scripts/deploy.py", "scripts", "Python", 100)],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(!terms.contains(&"py"), "py extension should be stopword");
    assert!(terms.contains(&"deploy"));
}

#[test]
fn typescript_extension_filtered_as_stopword() {
    let data = export(
        vec![row_lang(
            "components/Button.tsx",
            "components",
            "TypeScript",
            100,
        )],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(!terms.contains(&"tsx"));
    assert!(terms.contains(&"button"));
}

#[test]
fn mixed_language_project_produces_meaningful_topics() {
    let data = export(
        vec![
            row_lang("backend/server.rs", "backend", "Rust", 200),
            row_lang("frontend/app.tsx", "frontend", "TypeScript", 200),
            row_lang("scripts/migrate.py", "scripts", "Python", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    assert!(clouds.per_module.len() == 3);
    assert!(!clouds.overall.is_empty());
}

// ── 6. Edge cases ───────────────────────────────────────────────

#[test]
fn empty_export_produces_empty_clouds() {
    let data = export(vec![], &[]);
    let clouds = build_topic_clouds(&data);
    assert!(clouds.overall.is_empty());
    assert!(clouds.per_module.is_empty());
}

#[test]
fn only_child_rows_produce_empty_clouds() {
    let data = export(
        vec![child_row("a/b.rs", "a", 100), child_row("c/d.rs", "c", 100)],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    assert!(clouds.overall.is_empty());
}

#[test]
fn single_file_produces_topics() {
    let data = export(vec![row("project/feature_flag.rs", "project", 100)], &[]);
    let clouds = build_topic_clouds(&data);
    assert!(!clouds.overall.is_empty());
}

#[test]
fn path_with_only_stopwords_produces_no_terms() {
    let data = export(vec![row("src/lib/mod.rs", "src/lib", 100)], &[]);
    let clouds = build_topic_clouds(&data);
    // All segments are stopwords: src, lib, mod, rs
    assert!(clouds.overall.is_empty());
}

#[test]
fn massive_number_of_files_still_capped() {
    let rows: Vec<FileRow> = (0..500)
        .map(|i| {
            row(
                &format!("mod_{}/file_{}.rs", i % 10, i),
                &format!("mod_{}", i % 10),
                50,
            )
        })
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);
    // overall should still be capped at TOP_K=8
    assert!(clouds.overall.len() <= 8);
    for (_, terms) in &clouds.per_module {
        assert!(terms.len() <= 8);
    }
}

#[test]
fn zero_token_files_contribute_with_weight_one() {
    let data = export(
        vec![
            row("feature/zero.rs", "feature", 0),
            row("feature/also_zero.rs", "feature", 0),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    assert!(
        !clouds.overall.is_empty(),
        "zero-token rows should still produce topics"
    );
    for t in &clouds.overall {
        assert!(t.tf >= 1, "tf should be at least 1 for zero-token rows");
    }
}

#[test]
fn very_large_token_count_does_not_overflow() {
    let data = export(vec![row("big/enormous_file.rs", "big", usize::MAX)], &[]);
    let clouds = build_topic_clouds(&data);
    assert!(!clouds.overall.is_empty());
}

// ── 7. Deterministic output ─────────────────────────────────────

#[test]
fn deterministic_across_multiple_runs() {
    let make_data = || {
        export(
            vec![
                row("core/parser.rs", "core", 200),
                row("core/lexer.rs", "core", 150),
                row("api/handler.rs", "api", 300),
                row("api/router.rs", "api", 100),
                row("db/schema.rs", "db", 250),
            ],
            &[],
        )
    };
    let clouds1 = build_topic_clouds(&make_data());
    let clouds2 = build_topic_clouds(&make_data());

    // Overall terms must be identical
    assert_eq!(clouds1.overall.len(), clouds2.overall.len());
    for (a, b) in clouds1.overall.iter().zip(clouds2.overall.iter()) {
        assert_eq!(a.term, b.term);
        assert!((a.score - b.score).abs() < 1e-10);
        assert_eq!(a.tf, b.tf);
        assert_eq!(a.df, b.df);
    }

    // Per-module maps must be identical
    assert_eq!(clouds1.per_module.len(), clouds2.per_module.len());
    for (mod_name, terms1) in &clouds1.per_module {
        let terms2 = clouds2.per_module.get(mod_name).unwrap();
        assert_eq!(terms1.len(), terms2.len());
        for (a, b) in terms1.iter().zip(terms2.iter()) {
            assert_eq!(a.term, b.term);
        }
    }
}

#[test]
fn sort_order_is_score_desc_then_term_asc() {
    let data = export(
        vec![
            row("a/alpha.rs", "a", 100),
            row("a/beta.rs", "a", 100),
            row("a/gamma.rs", "a", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let terms = clouds.per_module.get("a").unwrap();
    for window in terms.windows(2) {
        let is_valid_order = window[0].score > window[1].score
            || (window[0].score == window[1].score && window[0].term <= window[1].term);
        assert!(
            is_valid_order,
            "terms should be sorted by score desc, term asc"
        );
    }
}

#[test]
fn overall_sort_order_is_score_desc_then_term_asc() {
    let data = export(
        vec![
            row("x/one.rs", "x", 50),
            row("y/two.rs", "y", 50),
            row("z/three.rs", "z", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    for window in clouds.overall.windows(2) {
        let is_valid_order = window[0].score > window[1].score
            || ((window[0].score - window[1].score).abs() < 1e-10
                && window[0].term <= window[1].term);
        assert!(
            is_valid_order,
            "overall terms should be sorted by score desc, term asc"
        );
    }
}

#[test]
fn serde_roundtrip_preserves_clouds() {
    let data = export(
        vec![
            row("api/handler.rs", "api", 200),
            row("api/router.rs", "api", 150),
            row("db/query.rs", "db", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let json = serde_json::to_string(&clouds).unwrap();
    let deserialized: tokmd_analysis_types::TopicClouds = serde_json::from_str(&json).unwrap();
    assert_eq!(clouds.overall.len(), deserialized.overall.len());
    assert_eq!(clouds.per_module.len(), deserialized.per_module.len());
    for (a, b) in clouds.overall.iter().zip(deserialized.overall.iter()) {
        assert_eq!(a.term, b.term);
        assert!((a.score - b.score).abs() < 1e-10);
    }
}

// ── 8. Cross-module aggregation ─────────────────────────────────

#[test]
fn term_appearing_in_all_modules_has_high_df() {
    let data = export(
        vec![
            row("a/common_name.rs", "a", 100),
            row("b/common_name.rs", "b", 100),
            row("c/common_name.rs", "c", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let common = clouds.overall.iter().find(|t| t.term == "common").unwrap();
    assert_eq!(common.df, 3, "term in 3 modules should have df=3");
}

#[test]
fn module_specific_term_has_df_one() {
    let data = export(
        vec![
            row("a/unique_only.rs", "a", 100),
            row("b/different.rs", "b", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let unique_t = clouds.overall.iter().find(|t| t.term == "unique");
    if let Some(u) = unique_t {
        assert_eq!(u.df, 1);
    }
}

#[test]
fn empty_path_segments_are_skipped() {
    let data = export(vec![row("a//b///c.rs", "a", 100)], &[]);
    let clouds = build_topic_clouds(&data);
    let terms: Vec<&str> = clouds.overall.iter().map(|t| t.term.as_str()).collect();
    assert!(!terms.contains(&""), "empty segments should be skipped");
}

#[test]
fn single_module_has_idf_of_one() {
    // With only one module, idf = ln((1+1)/(1+1)) + 1 = ln(1) + 1 = 1.0
    let data = export(vec![row("only/feature.rs", "only", 100)], &[]);
    let clouds = build_topic_clouds(&data);
    for t in &clouds.overall {
        // With module_count=1, idf = ln(2/2) + 1 = 1.0; score = tf * 1.0 = tf
        let expected_idf = ((1.0 + 1.0) / (t.df as f64 + 1.0)).ln() + 1.0;
        let expected_score = t.tf as f64 * expected_idf;
        assert!(
            (t.score - expected_score).abs() < 1e-6,
            "score should match tf*idf formula: {} vs {}",
            t.score,
            expected_score,
        );
    }
}
