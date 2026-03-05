//! Deep W68 tests for topic-cloud extraction.
//!
//! Covers: single-language repos, multi-language repos, TF-IDF scoring,
//! deterministic ordering, stopword filtering, edge cases (empty, single file),
//! module-level isolation, TOP_K truncation, and token weighting.

use tokmd_analysis_topics::build_topic_clouds;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── Helpers ─────────────────────────────────────────────────────

fn row(path: &str, module: &str, lang: &str, tokens: usize) -> FileRow {
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

fn child_row(path: &str, module: &str) -> FileRow {
    FileRow {
        path: path.to_string(),
        module: module.to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Child,
        code: 5,
        comments: 0,
        blanks: 0,
        lines: 5,
        bytes: 50,
        tokens: 25,
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

fn overall_terms(data: &ExportData) -> Vec<String> {
    build_topic_clouds(data)
        .overall
        .iter()
        .map(|t| t.term.clone())
        .collect()
}

// ── 1. Empty export produces empty clouds ───────────────────────

#[test]
fn empty_export_produces_empty_clouds() {
    let data = export(vec![], &[]);
    let clouds = build_topic_clouds(&data);
    assert!(clouds.overall.is_empty());
    assert!(clouds.per_module.is_empty());
}

// ── 2. Single file single module ────────────────────────────────

#[test]
fn single_file_produces_topic() {
    let data = export(
        vec![row("auth/login_handler.rs", "auth", "Rust", 100)],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    assert!(!clouds.overall.is_empty());
    let terms = overall_terms(&data);
    assert!(terms.contains(&"login".to_string()));
    assert!(terms.contains(&"handler".to_string()));
}

// ── 3. Stopwords are filtered ───────────────────────────────────

#[test]
fn stopwords_filtered_from_topics() {
    let data = export(
        vec![row("src/lib/mod/main/index.rs", "app", "Rust", 50)],
        &[],
    );
    let terms = overall_terms(&data);
    assert!(!terms.contains(&"src".to_string()));
    assert!(!terms.contains(&"lib".to_string()));
    assert!(!terms.contains(&"mod".to_string()));
    assert!(!terms.contains(&"main".to_string()));
    assert!(!terms.contains(&"index".to_string()));
}

// ── 4. File extensions are filtered ─────────────────────────────

#[test]
fn file_extensions_filtered() {
    let data = export(
        vec![row("utils/parser.rs", "utils", "Rust", 50)],
        &[],
    );
    let terms = overall_terms(&data);
    assert!(!terms.contains(&"rs".to_string()));
    assert!(terms.contains(&"parser".to_string()));
}

// ── 5. Module roots become stopwords ────────────────────────────

#[test]
fn module_roots_become_stopwords() {
    let data = export(
        vec![row("crates/auth/login.rs", "crates/auth", "Rust", 50)],
        &["crates"],
    );
    let terms = overall_terms(&data);
    assert!(!terms.contains(&"crates".to_string()));
    assert!(terms.contains(&"auth".to_string()));
}

// ── 6. Deterministic ordering across runs ───────────────────────

#[test]
fn topic_ordering_is_deterministic() {
    let rows = vec![
        row("api/routes/users.rs", "api", "Rust", 100),
        row("api/routes/orders.rs", "api", "Rust", 100),
        row("api/routes/products.rs", "api", "Rust", 100),
    ];
    let data = export(rows, &[]);
    let t1 = build_topic_clouds(&data);
    let t2 = build_topic_clouds(&data);

    let terms1: Vec<String> = t1.overall.iter().map(|t| t.term.clone()).collect();
    let terms2: Vec<String> = t2.overall.iter().map(|t| t.term.clone()).collect();
    assert_eq!(terms1, terms2);
}

// ── 7. Multi-module topics are isolated ─────────────────────────

#[test]
fn per_module_topics_isolated() {
    let data = export(
        vec![
            row("auth/login.rs", "auth", "Rust", 50),
            row("auth/token.rs", "auth", "Rust", 50),
            row("billing/invoice.rs", "billing", "Rust", 50),
            row("billing/payment.rs", "billing", "Rust", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let auth_terms: Vec<String> = clouds
        .per_module
        .get("auth")
        .unwrap()
        .iter()
        .map(|t| t.term.clone())
        .collect();
    let billing_terms: Vec<String> = clouds
        .per_module
        .get("billing")
        .unwrap()
        .iter()
        .map(|t| t.term.clone())
        .collect();

    assert!(auth_terms.contains(&"login".to_string()));
    assert!(auth_terms.contains(&"token".to_string()));
    assert!(billing_terms.contains(&"invoice".to_string()));
    assert!(billing_terms.contains(&"payment".to_string()));
    // Cross-module terms should not leak
    assert!(!auth_terms.contains(&"invoice".to_string()));
    assert!(!billing_terms.contains(&"login".to_string()));
}

// ── 8. Child rows are excluded ──────────────────────────────────

#[test]
fn child_rows_excluded_from_topics() {
    let data = export(
        vec![
            row("app/server.rs", "app", "Rust", 50),
            child_row("app/embedded.js", "app"),
        ],
        &[],
    );
    let terms = overall_terms(&data);
    assert!(terms.contains(&"server".to_string()));
    assert!(!terms.contains(&"embedded".to_string()));
}

// ── 9. Score is positive for valid terms ────────────────────────

#[test]
fn scores_are_positive() {
    let data = export(
        vec![row("core/engine.rs", "core", "Rust", 100)],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    for term in &clouds.overall {
        assert!(term.score > 0.0, "score should be positive: {:?}", term);
    }
}

// ── 10. TF field equals token weight sum ────────────────────────

#[test]
fn tf_reflects_token_weight() {
    let data = export(
        vec![
            row("api/handler.rs", "api", "Rust", 100),
            row("api/handler_utils.rs", "api", "Rust", 200),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let handler_term = clouds
        .overall
        .iter()
        .find(|t| t.term == "handler")
        .expect("handler term should exist");
    // handler appears in both files, tf = weight(file1) + weight(file2) = 100 + 200
    assert_eq!(handler_term.tf, 300);
}

// ── 11. DF counts files where term appears ──────────────────────

#[test]
fn df_counts_files_where_term_appears() {
    let data = export(
        vec![
            row("api/handler.rs", "api", "Rust", 50),
            row("api/handler_v2.rs", "api", "Rust", 50),
            row("core/handler.rs", "core", "Rust", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let handler_term = clouds
        .overall
        .iter()
        .find(|t| t.term == "handler")
        .expect("handler term should exist");
    // "handler" appears in 3 files (df incremented per file)
    assert_eq!(handler_term.df, 3);
}

// ── 12. TOP_K truncation (max 8 terms per module) ───────────────

#[test]
fn top_k_truncation_per_module() {
    let rows: Vec<FileRow> = (0..20)
        .map(|i| row(&format!("mod/term{i}.rs"), "mod", "Rust", 50))
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);
    let mod_terms = clouds.per_module.get("mod").unwrap();
    assert!(mod_terms.len() <= 8, "per-module should be truncated to TOP_K=8");
}

// ── 13. TOP_K truncation on overall ─────────────────────────────

#[test]
fn top_k_truncation_overall() {
    let rows: Vec<FileRow> = (0..20)
        .map(|i| row(&format!("mod/term{i}.rs"), "mod", "Rust", 50))
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);
    assert!(clouds.overall.len() <= 8, "overall should be truncated to TOP_K=8");
}

// ── 14. Multi-language repo ─────────────────────────────────────

#[test]
fn multi_language_repo_produces_topics() {
    let data = export(
        vec![
            row("frontend/app/dashboard.tsx", "frontend", "TypeScript", 200),
            row("backend/api/controller.py", "backend", "Python", 150),
            row("infra/deploy/terraform.tf", "infra", "HCL", 80),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    assert!(clouds.per_module.contains_key("frontend"));
    assert!(clouds.per_module.contains_key("backend"));
    assert!(clouds.per_module.contains_key("infra"));
    assert!(!clouds.overall.is_empty());
}

// ── 15. Underscore and hyphen splitting ─────────────────────────

#[test]
fn underscores_and_hyphens_split_tokens() {
    let data = export(
        vec![row("utils/rate-limiter_config.rs", "utils", "Rust", 50)],
        &[],
    );
    let terms = overall_terms(&data);
    assert!(terms.contains(&"rate".to_string()));
    assert!(terms.contains(&"limiter".to_string()));
    assert!(terms.contains(&"config".to_string()));
}

// ── 16. Backslash paths normalized ──────────────────────────────

#[test]
fn backslash_paths_normalized() {
    let data = export(
        vec![row("utils\\crypto\\hash.rs", "utils", "Rust", 50)],
        &[],
    );
    let terms = overall_terms(&data);
    assert!(terms.contains(&"crypto".to_string()));
    assert!(terms.contains(&"hash".to_string()));
}

// ── 17. Higher token weight increases score ─────────────────────

#[test]
fn higher_token_weight_increases_score() {
    let data = export(
        vec![
            row("a/heavy.rs", "a", "Rust", 10000),
            row("a/light.rs", "a", "Rust", 1),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let heavy = clouds
        .overall
        .iter()
        .find(|t| t.term == "heavy")
        .expect("heavy");
    let light = clouds
        .overall
        .iter()
        .find(|t| t.term == "light")
        .expect("light");
    assert!(
        heavy.score > light.score,
        "heavy ({}) should score higher than light ({})",
        heavy.score,
        light.score
    );
}

// ── 18. Overall scores sorted descending ────────────────────────

#[test]
fn overall_scores_sorted_descending() {
    let rows: Vec<FileRow> = (0..10)
        .map(|i| row(&format!("mod/widget{i}.rs"), "mod", "Rust", (i + 1) * 10))
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);
    for pair in clouds.overall.windows(2) {
        assert!(
            pair[0].score >= pair[1].score
                || (pair[0].score == pair[1].score && pair[0].term <= pair[1].term),
            "overall not sorted: {:?} vs {:?}",
            pair[0],
            pair[1]
        );
    }
}

// ── 19. Per-module scores sorted descending ─────────────────────

#[test]
fn per_module_scores_sorted_descending() {
    let data = export(
        vec![
            row("api/auth.rs", "api", "Rust", 100),
            row("api/cache.rs", "api", "Rust", 200),
            row("api/db.rs", "api", "Rust", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let api_terms = clouds.per_module.get("api").unwrap();
    for pair in api_terms.windows(2) {
        assert!(
            pair[0].score >= pair[1].score
                || (pair[0].score == pair[1].score && pair[0].term <= pair[1].term),
            "per-module not sorted: {:?} vs {:?}",
            pair[0],
            pair[1]
        );
    }
}

// ── 20. IDF boosts rare terms with equal per-module TF ──────────

#[test]
fn idf_boosts_rare_terms_same_tf() {
    // Both terms appear in 1 file each within same module.
    // "rare" appears in 1 file (df=1), "spread" in 2 files (df=2).
    // Same module means same module_count=1 for scoring.
    // Use per-module to compare terms with same tf but different df.
    let data = export(
        vec![
            row("mod_a/rare.rs", "mod_a", "Rust", 50),
            row("mod_a/spread.rs", "mod_a", "Rust", 50),
            row("mod_b/spread.rs", "mod_b", "Rust", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let mod_a = clouds.per_module.get("mod_a").unwrap();
    let rare = mod_a.iter().find(|t| t.term == "rare").expect("rare");
    let spread = mod_a.iter().find(|t| t.term == "spread").expect("spread");
    // rare has df=1, spread has df=2; same tf within mod_a => rare has higher IDF
    assert!(
        rare.score > spread.score,
        "rare ({}) should score higher than spread ({}) in same module due to IDF",
        rare.score,
        spread.score
    );
}
