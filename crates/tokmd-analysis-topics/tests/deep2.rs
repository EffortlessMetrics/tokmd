//! Additional deep tests for topic-cloud extraction.
//!
//! Covers serialization roundtrips, empty input, single/multi-language repos,
//! keyword frequency accumulation, module root stopwords, and edge cases.

use tokmd_analysis_topics::build_topic_clouds;
use tokmd_analysis_types::{TopicClouds, TopicTerm};
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

// ── 1. Empty export produces empty topic clouds ─────────────────

#[test]
fn empty_export_produces_empty_clouds() {
    let data = export(vec![], &[]);
    let clouds = build_topic_clouds(&data);

    assert!(clouds.overall.is_empty());
    assert!(clouds.per_module.is_empty());
}

// ── 2. TopicClouds serialization roundtrip ──────────────────────

#[test]
fn topic_clouds_serialization_roundtrip() {
    let data = export(
        vec![
            row("auth/login.rs", "auth", "Rust", 50),
            row("auth/token.rs", "auth", "Rust", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let json = serde_json::to_string_pretty(&clouds).unwrap();
    let back: TopicClouds = serde_json::from_str(&json).unwrap();

    assert_eq!(clouds.overall.len(), back.overall.len());
    assert_eq!(clouds.per_module.len(), back.per_module.len());
    for (orig, deser) in clouds.overall.iter().zip(back.overall.iter()) {
        assert_eq!(orig.term, deser.term);
        assert!((orig.score - deser.score).abs() < 1e-10);
        assert_eq!(orig.tf, deser.tf);
        assert_eq!(orig.df, deser.df);
    }
}

// ── 3. TopicTerm serialization roundtrip ────────────────────────

#[test]
fn topic_term_serialization_roundtrip() {
    let term = TopicTerm {
        term: "widget".to_string(),
        score: 42.5,
        tf: 10,
        df: 3,
    };
    let json = serde_json::to_string(&term).unwrap();
    let back: TopicTerm = serde_json::from_str(&json).unwrap();

    assert_eq!(back.term, "widget");
    assert!((back.score - 42.5).abs() < 1e-10);
    assert_eq!(back.tf, 10);
    assert_eq!(back.df, 3);
}

// ── 4. Single language repo has correct per-module entries ──────

#[test]
fn single_language_repo_has_module_entries() {
    let data = export(
        vec![
            row("core/engine.rs", "core", "Rust", 100),
            row("core/parser.rs", "core", "Rust", 200),
            row("utils/helper.rs", "utils", "Rust", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);

    assert!(clouds.per_module.contains_key("core"));
    assert!(clouds.per_module.contains_key("utils"));
    assert_eq!(clouds.per_module.len(), 2);
}

// ── 5. Multi-language repo: terms from all languages ────────────

#[test]
fn multi_language_repo_extracts_terms_from_all_languages() {
    let data = export(
        vec![
            row("backend/api_handler.rs", "backend", "Rust", 100),
            row("frontend/component_view.js", "frontend", "JavaScript", 100),
            row("scripts/deploy_tool.py", "scripts", "Python", 100),
        ],
        &[],
    );
    let terms = overall_terms(&data);

    assert!(terms.contains(&"api".to_string()) || terms.contains(&"handler".to_string()));
    assert!(terms.contains(&"component".to_string()) || terms.contains(&"view".to_string()));
    assert!(terms.contains(&"deploy".to_string()) || terms.contains(&"tool".to_string()));
}

// ── 6. Child rows are excluded from topic extraction ────────────

#[test]
fn child_rows_excluded() {
    let rows = vec![
        row("mod/feature.rs", "mod", "Rust", 100),
        FileRow {
            path: "mod/embedded.js".to_string(),
            module: "mod".to_string(),
            lang: "JavaScript".to_string(),
            kind: FileKind::Child,
            code: 10,
            comments: 0,
            blanks: 0,
            lines: 10,
            bytes: 100,
            tokens: 50,
        },
    ];
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);

    // "embedded" should not appear since it's a Child row
    let all_terms: Vec<String> = clouds.overall.iter().map(|t| t.term.clone()).collect();
    assert!(
        !all_terms.contains(&"embedded".to_string()),
        "child rows should be excluded from topic extraction"
    );
}

// ── 7. Module roots are treated as stopwords ────────────────────

#[test]
fn module_roots_are_stopwords() {
    let data = export(
        vec![row("myproject/auth/login.rs", "myproject/auth", "Rust", 50)],
        &["myproject"],
    );
    let terms = overall_terms(&data);

    assert!(
        !terms.contains(&"myproject".to_string()),
        "module roots should be stopwords: {terms:?}"
    );
}

// ── 8. Weight proportional to token count ───────────────────────

#[test]
fn higher_token_count_produces_higher_tf() {
    let data = export(
        vec![
            row("m/heavy_feature.rs", "m", "Rust", 1000),
            row("m/light_feature.rs", "m", "Rust", 10),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let m_terms = clouds.per_module.get("m").expect("module 'm'");

    let heavy = m_terms.iter().find(|t| t.term == "heavy");
    let light = m_terms.iter().find(|t| t.term == "light");

    if let (Some(h), Some(l)) = (heavy, light) {
        assert!(
            h.tf > l.tf,
            "heavy ({}) should have higher tf than light ({})",
            h.tf,
            l.tf
        );
    }
}

// ── 9. Zero-token file uses weight of 1 ─────────────────────────

#[test]
fn zero_token_file_uses_minimum_weight() {
    let data = export(vec![row("m/empty_weight.rs", "m", "Rust", 0)], &[]);
    let clouds = build_topic_clouds(&data);
    let m_terms = clouds.per_module.get("m").expect("module 'm'");

    // With weight clamped to max(tokens, 1) = 1, terms should still exist
    let empty_term = m_terms.iter().find(|t| t.term == "empty");
    if let Some(t) = empty_term {
        assert_eq!(t.tf, 1, "zero-token file should use weight of 1");
    }
}

// ── 10. Overall terms sorted by score desc ──────────────────────

#[test]
fn overall_terms_sorted_by_score_descending() {
    let data = export(
        vec![
            row("a/feature_one.rs", "a", "Rust", 100),
            row("b/feature_two.rs", "b", "Rust", 200),
            row("c/feature_three.rs", "c", "Rust", 300),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);

    for window in clouds.overall.windows(2) {
        assert!(
            window[0].score >= window[1].score
                || (window[0].score - window[1].score).abs() < f64::EPSILON,
            "overall terms not sorted by score desc: {} < {}",
            window[0].score,
            window[1].score
        );
    }
}

// ── 11. Per-module terms sorted by score desc ───────────────────

#[test]
fn per_module_terms_sorted_by_score_descending() {
    let rows: Vec<FileRow> = (0..15)
        .map(|i| row(&format!("m/item{i}_feature.rs"), "m", "Rust", (i + 1) * 50))
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);

    for (_, terms) in &clouds.per_module {
        for window in terms.windows(2) {
            assert!(
                window[0].score >= window[1].score
                    || (window[0].score - window[1].score).abs() < f64::EPSILON,
                "per-module terms not sorted by score desc"
            );
        }
    }
}

// ── 12. TOP_K=8 limit applied per module ────────────────────────

#[test]
fn top_k_limit_applied_per_module() {
    let rows: Vec<FileRow> = (0..30)
        .map(|i| row(&format!("m/term{i}.rs"), "m", "Rust", (i + 1) * 10))
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);

    let m_terms = clouds.per_module.get("m").expect("module 'm'");
    assert!(
        m_terms.len() <= 8,
        "per-module terms should be capped at TOP_K=8, got {}",
        m_terms.len()
    );
}

// ── 13. TOP_K=8 limit applied to overall ────────────────────────

#[test]
fn top_k_limit_applied_to_overall() {
    let rows: Vec<FileRow> = (0..30)
        .map(|i| row(&format!("m/term{i}.rs"), "m", "Rust", (i + 1) * 10))
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);

    assert!(
        clouds.overall.len() <= 8,
        "overall terms should be capped at TOP_K=8, got {}",
        clouds.overall.len()
    );
}

// ── 14. IDF calculation: ln((N+1)/(df+1)) + 1 ──────────────────

#[test]
fn idf_formula_correctness() {
    // 3 modules, term appears in 1 module
    let data = export(
        vec![
            row("a/unique_term.rs", "a", "Rust", 100),
            row("b/other.rs", "b", "Rust", 100),
            row("c/another.rs", "c", "Rust", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);

    let unique = clouds.overall.iter().find(|t| t.term == "unique");
    if let Some(u) = unique {
        // IDF = ln((3+1)/(1+1)) + 1 = ln(2) + 1 ≈ 1.693
        // TF = weight_for_row = 100 tokens
        // Score = 100 * (ln(2) + 1) ≈ 169.3
        let expected_idf = (4.0_f64 / 2.0).ln() + 1.0;
        let expected_score = 100.0 * expected_idf;
        assert!(
            (u.score - expected_score).abs() < 0.1,
            "expected score ~{expected_score}, got {}",
            u.score
        );
    }
}

// ── 15. Determinism across multiple runs ────────────────────────

#[test]
fn multiple_runs_produce_identical_results() {
    let data = export(
        vec![
            row("api/handler.rs", "api", "Rust", 100),
            row("api/router.rs", "api", "Rust", 200),
            row("db/query.rs", "db", "Rust", 150),
            row("db/migration.rs", "db", "Rust", 50),
        ],
        &[],
    );

    let results: Vec<TopicClouds> = (0..5).map(|_| build_topic_clouds(&data)).collect();

    for i in 1..5 {
        assert_eq!(results[0].overall.len(), results[i].overall.len());
        for (a, b) in results[0].overall.iter().zip(results[i].overall.iter()) {
            assert_eq!(a.term, b.term);
            assert!((a.score - b.score).abs() < f64::EPSILON);
            assert_eq!(a.tf, b.tf);
            assert_eq!(a.df, b.df);
        }
    }
}

// ── 16. Paths with only stopwords produce no terms ──────────────

#[test]
fn path_with_only_stopwords_produces_no_terms() {
    let data = export(
        vec![row("src/lib/mod/index.rs", "src/lib", "Rust", 50)],
        &[],
    );
    let terms = overall_terms(&data);

    // All segments are stopwords: src, lib, mod, index, rs
    assert!(
        terms.is_empty(),
        "path with only stopwords should produce no terms: {terms:?}"
    );
}

// ── 17. Score always positive for valid terms ───────────────────

#[test]
fn score_always_positive() {
    let data = export(
        vec![
            row("a/widget.rs", "a", "Rust", 100),
            row("b/gadget.rs", "b", "Rust", 200),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);

    for term in &clouds.overall {
        assert!(
            term.score > 0.0,
            "score should be positive for '{}'",
            term.term
        );
    }
    for (_, terms) in &clouds.per_module {
        for term in terms {
            assert!(
                term.score > 0.0,
                "per-module score should be positive for '{}'",
                term.term
            );
        }
    }
}

// ── 18. TF is always positive ───────────────────────────────────

#[test]
fn tf_always_positive() {
    let data = export(
        vec![
            row("m/feature.rs", "m", "Rust", 1),
            row("n/another.rs", "n", "Rust", 1),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);

    for term in &clouds.overall {
        assert!(term.tf > 0, "tf should be positive for '{}'", term.term);
    }
}

// ── 19. DF is at least 1 for any present term ───────────────────

#[test]
fn df_at_least_one() {
    let data = export(
        vec![
            row("a/widget.rs", "a", "Rust", 100),
            row("b/gadget.rs", "b", "Rust", 200),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);

    for term in &clouds.overall {
        assert!(term.df >= 1, "df should be >= 1 for '{}'", term.term);
    }
}

// ── 20. Many modules with same term: df tracks file count ───────

#[test]
fn df_tracks_file_count_across_modules() {
    let rows: Vec<FileRow> = (0..10)
        .map(|i| {
            row(
                &format!("mod_{i}/shared_widget.rs"),
                &format!("mod_{i}"),
                "Rust",
                50,
            )
        })
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);

    let shared = clouds.overall.iter().find(|t| t.term == "shared");
    if let Some(s) = shared {
        assert_eq!(s.df, 10, "shared term should have df=10 across 10 files");
    }
}

// ── 21. Unicode path segments are lowercased ────────────────────

#[test]
fn unicode_path_segments_lowercased() {
    let data = export(
        vec![row("UPPER/CamelCase_Feature.rs", "UPPER", "Rust", 50)],
        &[],
    );
    let terms = overall_terms(&data);

    for term in &terms {
        assert_eq!(*term, term.to_lowercase(), "'{term}' should be lowercase");
    }
}

// ── 22. Empty path segments ignored ─────────────────────────────

#[test]
fn empty_path_segments_ignored() {
    let data = export(
        vec![row(
            "///double//slash///file.rs",
            "///double//slash",
            "Rust",
            50,
        )],
        &[],
    );
    let terms = overall_terms(&data);

    for term in &terms {
        assert!(!term.is_empty(), "empty terms should not appear");
    }
}

// ── 23. Single file with many terms truncated by TOP_K ──────────

#[test]
fn single_file_many_segments_truncated() {
    // Path with many unique segments (>8)
    let data = export(
        vec![row(
            "alpha/bravo/charlie/delta/echo/foxtrot/golf/hotel/india/juliet/kilo.rs",
            "alpha/bravo",
            "Rust",
            50,
        )],
        &[],
    );
    let clouds = build_topic_clouds(&data);

    assert!(
        clouds.overall.len() <= 8,
        "should be capped at TOP_K=8 even with many path segments, got {}",
        clouds.overall.len()
    );
}

// ── 24. Per-module and overall agree on single-module data ──────

#[test]
fn single_module_overall_matches_per_module() {
    let data = export(
        vec![
            row("m/alpha_feature.rs", "m", "Rust", 100),
            row("m/beta_feature.rs", "m", "Rust", 200),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let overall = &clouds.overall;
    let per_mod = clouds.per_module.get("m").expect("module 'm'");

    // With single module, overall and per_module should have same terms
    assert_eq!(overall.len(), per_mod.len());
    for (o, p) in overall.iter().zip(per_mod.iter()) {
        assert_eq!(o.term, p.term);
        assert!((o.score - p.score).abs() < f64::EPSILON);
    }
}

// ── 25. Serialization roundtrip preserves per_module keys ───────

#[test]
fn serialization_preserves_per_module_keys() {
    let data = export(
        vec![
            row("alpha/widget.rs", "alpha", "Rust", 50),
            row("beta/gadget.rs", "beta", "Rust", 50),
            row("gamma/tool.rs", "gamma", "Rust", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let json = serde_json::to_string(&clouds).unwrap();
    let back: TopicClouds = serde_json::from_str(&json).unwrap();

    let orig_keys: Vec<&String> = clouds.per_module.keys().collect();
    let back_keys: Vec<&String> = back.per_module.keys().collect();
    assert_eq!(orig_keys, back_keys);
}
