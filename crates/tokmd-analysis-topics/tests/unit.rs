//! Focused unit tests for topic-cloud extraction.

use tokmd_analysis_topics::build_topic_clouds;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── helpers ──────────────────────────────────────────────────────────

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
        module_roots: roots.iter().map(|s| s.to_string()).collect(),
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

// ── 1. deeply nested paths yield all non-stopword segments ───────────

#[test]
fn deeply_nested_path_extracts_all_segments() {
    let data = export(
        vec![row(
            "alpha/bravo/charlie/delta/feature.rs",
            "alpha/bravo",
            50,
        )],
        &[],
    );
    let terms = overall_terms(&data);
    // "rs" is an extension stopword, rest should appear
    for expected in ["alpha", "bravo", "charlie", "delta", "feature"] {
        assert!(
            terms.contains(&expected.to_string()),
            "missing '{expected}'"
        );
    }
}

// ── 2. duplicate paths in same module accumulate tf ──────────────────

#[test]
fn duplicate_paths_accumulate_tf() {
    let data = export(
        vec![
            row("mod_a/widget.rs", "mod_a", 100),
            row("mod_a/widget.rs", "mod_a", 100),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let widget = clouds
        .overall
        .iter()
        .find(|t| t.term == "widget")
        .expect("widget should exist");
    // Two rows contribute, so tf should be 2 * weight(100)
    assert_eq!(widget.tf, 200, "tf should accumulate across duplicate rows");
}

// ── 3. per_module keys exactly match input modules ───────────────────

#[test]
fn per_module_keys_match_input_modules() {
    let data = export(
        vec![
            row("alpha/foo.rs", "alpha", 50),
            row("beta/bar.rs", "beta", 50),
            row("beta/baz.rs", "beta", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let keys: Vec<&String> = clouds.per_module.keys().collect();
    assert_eq!(keys, vec!["alpha", "beta"]);
}

// ── 4. mixed parent and child rows only use parents ──────────────────

#[test]
fn mixed_parent_child_rows_only_parents_contribute() {
    let mut parent = row("mod/parent_term.rs", "mod", 50);
    let mut child = row("mod/child_term.rs", "mod", 50);
    child.kind = FileKind::Child;
    parent.kind = FileKind::Parent;
    let data = export(vec![parent, child], &[]);
    let terms = overall_terms(&data);
    assert!(
        terms.contains(&"parent".to_string()),
        "parent term should be present"
    );
    // "child" could match as a term from "child_term", but the child row is filtered
    assert!(
        !terms.iter().any(|t| t == "child"),
        "child row terms should not appear: {terms:?}"
    );
}

// ── 5. many modules each get their own per_module entry ──────────────

#[test]
fn many_modules_each_get_per_module_entry() {
    let rows: Vec<FileRow> = (0..15)
        .map(|i| row(&format!("mod_{i}/file_{i}.rs"), &format!("mod_{i}"), 50))
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);
    assert_eq!(clouds.per_module.len(), 15);
}

// ── 6. overall tf sums across modules ────────────────────────────────

#[test]
fn overall_tf_sums_across_modules() {
    let data = export(
        vec![
            row("mod_a/shared.rs", "mod_a", 30),
            row("mod_b/shared.rs", "mod_b", 70),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let shared = clouds
        .overall
        .iter()
        .find(|t| t.term == "shared")
        .expect("shared should exist");
    // tf should be sum of weights: 30 + 70 = 100
    assert_eq!(shared.tf, 100, "overall tf should sum across modules");
}

// ── 7. module roots are case-insensitive stopwords ───────────────────

#[test]
fn module_roots_case_insensitive_stopword() {
    // Module root "Packages" should filter "packages" (lowercased)
    let data = export(
        vec![row("Packages/core/util.rs", "Packages/core", 50)],
        &["Packages"],
    );
    let terms = overall_terms(&data);
    assert!(
        !terms.contains(&"packages".to_string()),
        "'packages' (from root 'Packages') should be stopped"
    );
}

// ── 8. per_module sorted descending by score then by term ────────────

#[test]
fn per_module_sorted_descending_by_score_then_term() {
    let rows: Vec<FileRow> = (0..10)
        .map(|i| row(&format!("m/term_{i}.rs"), "m", (i + 1) * 10))
        .collect();
    let data = export(rows, &[]);
    let clouds = build_topic_clouds(&data);
    let m_terms = clouds.per_module.get("m").expect("module 'm' should exist");
    for window in m_terms.windows(2) {
        let ordering = window[0]
            .score
            .partial_cmp(&window[1].score)
            .unwrap_or(std::cmp::Ordering::Equal);
        assert!(
            ordering != std::cmp::Ordering::Less,
            "per_module not sorted: {} ({}) < {} ({})",
            window[0].term,
            window[0].score,
            window[1].term,
            window[1].score,
        );
        if (window[0].score - window[1].score).abs() < f64::EPSILON {
            assert!(
                window[0].term <= window[1].term,
                "tie-break should be alphabetical: '{}' > '{}'",
                window[0].term,
                window[1].term,
            );
        }
    }
}

// ── 9. single module means idf component is constant ─────────────────

#[test]
fn single_module_idf_is_constant() {
    let data = export(
        vec![row("m/alpha.rs", "m", 100), row("m/beta.rs", "m", 100)],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    // With one module, idf = ln((1+1)/(df+1)) + 1 for all terms
    // All terms with equal tf should have equal score
    let m_terms = clouds.per_module.get("m").unwrap();
    let alpha = m_terms.iter().find(|t| t.term == "alpha").unwrap();
    let beta = m_terms.iter().find(|t| t.term == "beta").unwrap();
    // Same tf, same df, same module_count → same score
    assert_eq!(alpha.tf, beta.tf);
    assert!((alpha.score - beta.score).abs() < f64::EPSILON);
}

// ── 10. high-token file dominates tf ─────────────────────────────────

#[test]
fn high_token_file_dominates_tf_in_module() {
    let data = export(
        vec![row("m/small.rs", "m", 1), row("m/huge.rs", "m", 10_000)],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let m_terms = clouds.per_module.get("m").unwrap();
    let huge = m_terms.iter().find(|t| t.term == "huge").unwrap();
    let small = m_terms.iter().find(|t| t.term == "small").unwrap();
    assert!(
        huge.tf > small.tf,
        "huge.tf ({}) should exceed small.tf ({})",
        huge.tf,
        small.tf
    );
}

// ── 11. empty path segments are skipped ──────────────────────────────

#[test]
fn empty_path_segments_are_skipped() {
    // Double slashes create empty segments
    let data = export(vec![row("a//b///c.rs", "a", 50)], &[]);
    let terms = overall_terms(&data);
    // Should only have real tokens, no empty strings
    for t in &terms {
        assert!(!t.is_empty(), "empty term found in {terms:?}");
    }
}

// ── 12. all-stopword path yields no terms ────────────────────────────

#[test]
fn path_with_only_stopwords_yields_no_contribution() {
    let data = export(vec![row("src/lib/mod/test/index.rs", "src/lib", 50)], &[]);
    let clouds = build_topic_clouds(&data);
    // Every segment is a stopword → no terms extracted
    assert!(clouds.overall.is_empty());
}

// ── 13. unique term in one module scores higher than shared term ──────

#[test]
fn unique_term_has_higher_idf_than_ubiquitous_term() {
    // "common" appears in both modules; "unique" only in mod_a
    let data = export(
        vec![
            row("mod_a/common.rs", "mod_a", 50),
            row("mod_a/unique.rs", "mod_a", 50),
            row("mod_b/common.rs", "mod_b", 50),
            row("mod_b/other.rs", "mod_b", 50),
        ],
        &[],
    );
    let clouds = build_topic_clouds(&data);
    let mod_a = clouds.per_module.get("mod_a").unwrap();
    let common = mod_a.iter().find(|t| t.term == "common").unwrap();
    let unique = mod_a.iter().find(|t| t.term == "unique").unwrap();
    // same tf weight, but "unique" has df=1 vs "common" df=2 → higher score
    assert!(
        unique.score > common.score,
        "unique ({}) should score higher than common ({})",
        unique.score,
        common.score
    );
}

// ── 14. u64::MAX tokens clamp to u32::MAX weight ─────────────────────

#[test]
fn very_large_token_count_does_not_panic() {
    let data = export(vec![row("m/big.rs", "m", usize::MAX)], &[]);
    let clouds = build_topic_clouds(&data);
    // Should not panic; term should exist with clamped weight
    assert!(
        clouds.overall.iter().any(|t| t.term == "big"),
        "should extract term even with MAX tokens"
    );
}

// ── 15. extension stopwords cover many languages ─────────────────────

#[test]
fn common_extensions_are_stopped() {
    // Each file extension should be filtered as a stopword
    let extensions = ["rs", "js", "ts", "py", "go", "java", "cpp", "swift"];
    for ext in extensions {
        let path = format!("mod/feature.{ext}");
        let data = export(vec![row(&path, "mod", 50)], &[]);
        let terms = overall_terms(&data);
        assert!(
            !terms.contains(&ext.to_string()),
            "extension '{ext}' should be a stopword, got {terms:?}"
        );
    }
}
