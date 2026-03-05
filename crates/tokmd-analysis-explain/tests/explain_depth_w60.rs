//! Wave-60 depth tests for `tokmd-analysis-explain`.
//!
//! Covers:
//! - BDD-style edge cases for explanation generation
//! - Property tests for determinism and invariants
//! - Empty, large, and special-character inputs
//! - Catalog–lookup round-trip consistency
//! - Summary content quality constraints

use proptest::prelude::*;
use tokmd_analysis_explain::{catalog, lookup};

// =============================================================================
// 1. BDD: Explanation generation edge cases
// =============================================================================

// ── Scenario: lookup returns consistent format for every entry ──────

#[test]
fn given_every_canonical_key_when_looked_up_then_format_is_key_colon_space_summary() {
    let cat = catalog();
    for line in cat.lines().skip(1) {
        if let Some(key) = line.strip_prefix("- ") {
            let text = lookup(key).unwrap();
            let parts: Vec<&str> = text.splitn(2, ": ").collect();
            assert_eq!(
                parts.len(),
                2,
                "lookup({key}) must have 'canonical: summary' format"
            );
            assert_eq!(parts[0], key, "first part must be the canonical key");
            assert!(
                !parts[1].is_empty(),
                "summary for '{key}' must not be empty"
            );
            assert!(
                parts[1].ends_with('.'),
                "summary for '{key}' must end with period, got: {}",
                parts[1]
            );
        }
    }
}

// ── Scenario: alias variants all converge to same canonical output ──

#[test]
fn given_halstead_aliases_when_looked_up_then_all_match_canonical() {
    let canonical = lookup("halstead").unwrap();
    assert_eq!(lookup("halstead_volume").unwrap(), canonical);
    assert_eq!(lookup("halstead_effort").unwrap(), canonical);
}

#[test]
fn given_duplication_aliases_when_looked_up_then_all_match_canonical() {
    let canonical = lookup("duplicate_waste").unwrap();
    assert_eq!(lookup("dup").unwrap(), canonical);
    assert_eq!(lookup("duplication").unwrap(), canonical);
}

#[test]
fn given_complexity_histogram_alias_when_looked_up_then_matches_canonical() {
    let canonical = lookup("complexity_histogram").unwrap();
    assert_eq!(lookup("histogram").unwrap(), canonical);
}

#[test]
fn given_duplication_density_alias_when_looked_up_then_matches_canonical() {
    let canonical = lookup("duplication_density").unwrap();
    assert_eq!(lookup("dup_density").unwrap(), canonical);
}

// ── Scenario: normalization collapses all separator types ───────────

#[test]
fn given_three_word_key_with_mixed_seps_when_looked_up_then_all_resolve_identically() {
    let ref_val = lookup("max_nesting_depth").unwrap();
    assert_eq!(lookup("max-nesting-depth").unwrap(), ref_val);
    assert_eq!(lookup("max.nesting.depth").unwrap(), ref_val);
    assert_eq!(lookup("max nesting depth").unwrap(), ref_val);
    assert_eq!(lookup("MAX_NESTING_DEPTH").unwrap(), ref_val);
    assert_eq!(lookup("Max-Nesting-Depth").unwrap(), ref_val);
    assert_eq!(lookup("Max.Nesting.Depth").unwrap(), ref_val);
}

#[test]
fn given_two_word_key_with_mixed_seps_when_looked_up_then_all_resolve() {
    let ref_val = lookup("bus_factor").unwrap();
    assert_eq!(lookup("bus-factor").unwrap(), ref_val);
    assert_eq!(lookup("bus.factor").unwrap(), ref_val);
    assert_eq!(lookup("bus factor").unwrap(), ref_val);
    assert_eq!(lookup("BUS_FACTOR").unwrap(), ref_val);
}

// ── Scenario: subtle key variations that must NOT resolve ───────────

#[test]
fn given_key_with_doubled_underscore_when_looked_up_then_returns_none() {
    assert!(lookup("doc__density").is_none());
    assert!(lookup("bus__factor").is_none());
    assert!(lookup("max__cyclomatic").is_none());
}

#[test]
fn given_key_with_trailing_separator_when_looked_up_then_returns_none() {
    assert!(lookup("doc_density_").is_none());
    assert!(lookup("_doc_density").is_none());
    assert!(lookup("gini_").is_none());
}

#[test]
fn given_partial_canonical_key_when_looked_up_then_returns_none() {
    assert!(lookup("doc").is_none());
    assert!(lookup("context_window").is_none());
    assert!(lookup("code_age").is_some()); // this is an alias
    assert!(lookup("code").is_none());
    assert!(lookup("polyglot").is_some()); // this is an alias for polyglot_entropy
}

#[test]
fn given_key_with_numeric_suffix_when_looked_up_then_returns_none() {
    assert!(lookup("gini2").is_none());
    assert!(lookup("halstead1").is_none());
    assert!(lookup("imports0").is_none());
}

// ── Scenario: control characters and unusual whitespace ─────────────

#[test]
fn given_key_with_newline_when_looked_up_then_returns_none() {
    assert!(lookup("doc\ndensity").is_none());
    // "gini\n" resolves because trim() strips newlines
    assert!(lookup("gini\n").is_some());
}

#[test]
fn given_key_with_embedded_newline_returns_none() {
    assert!(lookup("doc\ndensity").is_none());
    assert!(lookup("bus\nfactor").is_none());
}

#[test]
fn given_key_with_carriage_return_when_looked_up_then_returns_none() {
    assert!(lookup("doc\r\ndensity").is_none());
}

#[test]
fn given_key_with_null_byte_when_looked_up_then_returns_none() {
    assert!(lookup("halstead\0").is_none());
    assert!(lookup("\0gini").is_none());
}

#[test]
fn given_key_with_backslash_when_looked_up_then_returns_none() {
    assert!(lookup("doc\\density").is_none());
    assert!(lookup("gini\\").is_none());
}

// ── Scenario: emoji and non-latin characters ────────────────────────

#[test]
fn given_key_with_emoji_prefix_or_suffix_when_looked_up_then_returns_none() {
    assert!(lookup("📊gini").is_none());
    assert!(lookup("gini📊").is_none());
    assert!(lookup("📊").is_none());
}

#[test]
fn given_key_in_cyrillic_when_looked_up_then_returns_none() {
    assert!(lookup("гини").is_none());
}

#[test]
fn given_key_with_zero_width_space_when_looked_up_then_returns_none() {
    assert!(lookup("gini\u{200B}").is_none());
    assert!(lookup("doc\u{200B}_density").is_none());
}

// =============================================================================
// 2. Catalog invariants
// =============================================================================

#[test]
fn catalog_key_count_is_exactly_28() {
    let text = catalog();
    let count = text.lines().skip(1).filter(|l| l.starts_with("- ")).count();
    assert_eq!(count, 28, "expected exactly 28 catalog entries");
}

#[test]
fn catalog_keys_are_all_lowercase_with_underscores() {
    let text = catalog();
    for line in text.lines().skip(1) {
        if let Some(key) = line.strip_prefix("- ") {
            for ch in key.chars() {
                assert!(
                    ch.is_ascii_lowercase() || ch == '_',
                    "catalog key '{key}' contains non-lowercase/non-underscore char '{ch}'"
                );
            }
        }
    }
}

#[test]
fn catalog_keys_do_not_start_or_end_with_underscore() {
    let text = catalog();
    for line in text.lines().skip(1) {
        if let Some(key) = line.strip_prefix("- ") {
            assert!(
                !key.starts_with('_'),
                "key '{key}' should not start with underscore"
            );
            assert!(
                !key.ends_with('_'),
                "key '{key}' should not end with underscore"
            );
        }
    }
}

#[test]
fn catalog_keys_do_not_contain_consecutive_underscores() {
    let text = catalog();
    for line in text.lines().skip(1) {
        if let Some(key) = line.strip_prefix("- ") {
            assert!(
                !key.contains("__"),
                "key '{key}' should not have consecutive underscores"
            );
        }
    }
}

#[test]
fn catalog_header_line_is_not_a_key_line() {
    let text = catalog();
    let first = text.lines().next().unwrap();
    assert!(
        !first.starts_with("- "),
        "first line should be a header, not a key"
    );
}

// =============================================================================
// 3. Summary content quality
// =============================================================================

#[test]
fn all_summaries_are_non_trivially_long() {
    let cat = catalog();
    for line in cat.lines().skip(1) {
        if let Some(key) = line.strip_prefix("- ") {
            let text = lookup(key).unwrap();
            let summary = text.split_once(": ").unwrap().1;
            assert!(
                summary.len() >= 15,
                "summary for '{key}' is too short ({} chars): {summary}",
                summary.len()
            );
        }
    }
}

#[test]
fn no_summary_exceeds_200_characters() {
    let cat = catalog();
    for line in cat.lines().skip(1) {
        if let Some(key) = line.strip_prefix("- ") {
            let text = lookup(key).unwrap();
            let summary = text.split_once(": ").unwrap().1;
            assert!(
                summary.len() <= 200,
                "summary for '{key}' exceeds 200 chars ({} chars)",
                summary.len()
            );
        }
    }
}

#[test]
fn all_summaries_are_pure_ascii() {
    let cat = catalog();
    for line in cat.lines().skip(1) {
        if let Some(key) = line.strip_prefix("- ") {
            let text = lookup(key).unwrap();
            assert!(
                text.is_ascii(),
                "explanation for '{key}' contains non-ASCII characters"
            );
        }
    }
}

#[test]
fn all_summaries_are_single_line() {
    let cat = catalog();
    for line in cat.lines().skip(1) {
        if let Some(key) = line.strip_prefix("- ") {
            let text = lookup(key).unwrap();
            assert!(
                !text.contains('\n'),
                "explanation for '{key}' contains newline"
            );
            assert!(
                !text.contains('\r'),
                "explanation for '{key}' contains carriage return"
            );
        }
    }
}

// =============================================================================
// 4. lookup vs catalog cross-consistency
// =============================================================================

#[test]
fn every_catalog_key_round_trips_through_lookup_back_to_same_key() {
    let cat = catalog();
    for line in cat.lines().skip(1) {
        if let Some(key) = line.strip_prefix("- ") {
            let text = lookup(key).unwrap();
            let resolved_key = text.split_once(": ").unwrap().0;
            assert_eq!(
                resolved_key, key,
                "catalog key '{key}' resolved to different canonical '{resolved_key}'"
            );
        }
    }
}

#[test]
fn catalog_key_set_matches_lookup_resolvable_canonical_keys() {
    let cat = catalog();
    let catalog_keys: Vec<&str> = cat
        .lines()
        .skip(1)
        .filter_map(|l| l.strip_prefix("- "))
        .collect();

    // Every resolvable canonical key should be in the catalog
    let known_canonical = [
        "doc_density",
        "whitespace_ratio",
        "verbosity",
        "test_density",
        "todo_density",
        "polyglot_entropy",
        "gini",
        "avg_cyclomatic",
        "max_cyclomatic",
        "avg_cognitive",
        "max_nesting_depth",
        "maintainability_index",
        "technical_debt_ratio",
        "halstead",
        "complexity_histogram",
        "hotspots",
        "bus_factor",
        "freshness",
        "code_age_distribution",
        "coupling",
        "predictive_churn",
        "duplicate_waste",
        "duplication_density",
        "imports",
        "entropy_suspects",
        "license_radar",
        "archetype",
        "context_window_fit",
    ];
    for key in known_canonical {
        assert!(
            catalog_keys.contains(&key),
            "canonical key '{key}' missing from catalog"
        );
    }
    assert_eq!(catalog_keys.len(), known_canonical.len());
}

// =============================================================================
// 5. Semantic content validation per metric family
// =============================================================================

#[test]
fn complexity_family_summaries_mention_complexity() {
    for key in ["avg_cyclomatic", "max_cyclomatic", "avg_cognitive"] {
        let text = lookup(key).unwrap().to_lowercase();
        assert!(
            text.contains("complex"),
            "'{key}' summary should mention complexity"
        );
    }
}

#[test]
fn git_family_summaries_mention_relevant_concepts() {
    let bus = lookup("bus_factor").unwrap().to_lowercase();
    assert!(bus.contains("author") || bus.contains("ownership"));

    let hot = lookup("hotspots").unwrap().to_lowercase();
    assert!(hot.contains("change") || hot.contains("frequency"));

    let fresh = lookup("freshness").unwrap().to_lowercase();
    assert!(fresh.contains("recen") || fresh.contains("stale"));
}

#[test]
fn duplication_family_summaries_mention_duplicate_or_redundant() {
    for key in ["duplicate_waste", "duplication_density"] {
        let text = lookup(key).unwrap().to_lowercase();
        assert!(
            text.contains("duplicate") || text.contains("redundant"),
            "'{key}' summary should mention duplication"
        );
    }
}

#[test]
fn size_metric_summaries_mention_relevant_terms() {
    let gini = lookup("gini").unwrap().to_lowercase();
    assert!(
        gini.contains("inequality") || gini.contains("concentration") || gini.contains("file"),
        "gini summary should mention inequality or files"
    );

    let verb = lookup("verbosity").unwrap().to_lowercase();
    assert!(
        verb.contains("byte") || verb.contains("line"),
        "verbosity summary should mention bytes or lines"
    );
}

// =============================================================================
// 6. Large / stress inputs
// =============================================================================

#[test]
fn lookup_with_100k_char_input_returns_none_without_panic() {
    let huge = "x".repeat(100_000);
    assert!(lookup(&huge).is_none());
}

#[test]
fn lookup_with_all_separator_chars_repeated() {
    let seps = "-._ ".repeat(500);
    assert!(lookup(&seps).is_none());
}

#[test]
fn catalog_called_1000_times_returns_identical_result() {
    let baseline = catalog();
    for _ in 0..1000 {
        assert_eq!(catalog(), baseline);
    }
}

// =============================================================================
// 7. Property tests
// =============================================================================

proptest! {
    #[test]
    fn lookup_is_deterministic_for_any_input(key in "[ -~]{0,100}") {
        let r1 = lookup(&key);
        let r2 = lookup(&key);
        prop_assert_eq!(r1, r2);
    }

    #[test]
    fn lookup_never_panics_on_binary_like_input(key in prop::collection::vec(0u8..255, 0..200)) {
        let s = String::from_utf8_lossy(&key);
        let _ = lookup(&s);
    }

    #[test]
    fn lookup_result_when_some_always_contains_colon_space(key in "[a-z_]{1,30}") {
        if let Some(text) = lookup(&key) {
            prop_assert!(text.contains(": "), "result must contain ': '");
        }
    }

    #[test]
    fn lookup_canonical_with_random_padding_resolves(
        idx in 0usize..28,
        left in " {0,10}",
        right in " {0,10}",
    ) {
        let keys = [
            "doc_density", "whitespace_ratio", "verbosity", "test_density",
            "todo_density", "polyglot_entropy", "gini", "avg_cyclomatic",
            "max_cyclomatic", "avg_cognitive", "max_nesting_depth",
            "maintainability_index", "technical_debt_ratio", "halstead",
            "complexity_histogram", "hotspots", "bus_factor", "freshness",
            "code_age_distribution", "coupling", "predictive_churn",
            "duplicate_waste", "duplication_density", "imports",
            "entropy_suspects", "license_radar", "archetype", "context_window_fit",
        ];
        let key = keys[idx];
        let padded = format!("{left}{key}{right}");
        prop_assert!(lookup(&padded).is_some(), "padded '{padded}' should resolve");
    }

    #[test]
    fn lookup_with_random_case_resolves(idx in 0usize..28, upper in proptest::bool::ANY) {
        let keys = [
            "doc_density", "whitespace_ratio", "verbosity", "test_density",
            "todo_density", "polyglot_entropy", "gini", "avg_cyclomatic",
            "max_cyclomatic", "avg_cognitive", "max_nesting_depth",
            "maintainability_index", "technical_debt_ratio", "halstead",
            "complexity_histogram", "hotspots", "bus_factor", "freshness",
            "code_age_distribution", "coupling", "predictive_churn",
            "duplicate_waste", "duplication_density", "imports",
            "entropy_suspects", "license_radar", "archetype", "context_window_fit",
        ];
        let key = keys[idx];
        let variant = if upper { key.to_uppercase() } else { key.to_lowercase() };
        prop_assert!(lookup(&variant).is_some());
    }

    #[test]
    fn catalog_never_changes_across_invocations(_seed in 0u32..500) {
        let a = catalog();
        let b = catalog();
        prop_assert_eq!(a, b);
    }

    #[test]
    fn lookup_unknown_prefix_returns_none(prefix in "[0-9]{1,5}", key in "[a-z_]{1,15}") {
        let combined = format!("{prefix}{key}");
        prop_assert!(lookup(&combined).is_none(), "numeric-prefixed '{combined}' should not resolve");
    }

    #[test]
    fn lookup_result_canonical_key_has_no_spaces(key in "[a-z_ .\\-]{1,30}") {
        if let Some(text) = lookup(&key) {
            let canonical = text.split_once(": ").unwrap().0;
            prop_assert!(!canonical.contains(' '), "canonical key should not contain spaces: '{canonical}'");
            prop_assert!(!canonical.contains('-'), "canonical key should not contain hyphens: '{canonical}'");
            prop_assert!(!canonical.contains('.'), "canonical key should not contain dots: '{canonical}'");
        }
    }
}

// =============================================================================
// 8. Additional edge-case scenarios
// =============================================================================

#[test]
fn given_max_cyclomatic_has_no_aliases_when_looked_up_then_only_canonical_works() {
    assert!(lookup("max_cyclomatic").is_some());
    // No aliases defined for max_cyclomatic, verify none accidentally resolve
    assert!(lookup("maximum_cyclomatic").is_none());
    assert!(lookup("max_cc").is_none());
}

#[test]
fn given_every_alias_when_looked_up_then_output_starts_with_its_canonical() {
    let alias_to_canonical = [
        ("documentation_density", "doc_density"),
        ("docs", "doc_density"),
        ("whitespace", "whitespace_ratio"),
        ("bytes_per_line", "verbosity"),
        ("tests", "test_density"),
        ("todo", "todo_density"),
        ("fixme", "todo_density"),
        ("language_entropy", "polyglot_entropy"),
        ("polyglot", "polyglot_entropy"),
        ("distribution_gini", "gini"),
        ("cyclomatic", "avg_cyclomatic"),
        ("cognitive", "avg_cognitive"),
        ("nesting_depth", "max_nesting_depth"),
        ("mi", "maintainability_index"),
        ("debt_ratio", "technical_debt_ratio"),
        ("technical_debt", "technical_debt_ratio"),
        ("halstead_volume", "halstead"),
        ("halstead_effort", "halstead"),
        ("histogram", "complexity_histogram"),
        ("git_hotspots", "hotspots"),
        ("ownership", "bus_factor"),
        ("staleness", "freshness"),
        ("code_age", "code_age_distribution"),
        ("age_buckets", "code_age_distribution"),
        ("module_coupling", "coupling"),
        ("churn", "predictive_churn"),
        ("dup", "duplicate_waste"),
        ("duplication", "duplicate_waste"),
        ("dup_density", "duplication_density"),
        ("import_graph", "imports"),
        ("entropy", "entropy_suspects"),
        ("license", "license_radar"),
        ("project_archetype", "archetype"),
        ("window_fit", "context_window_fit"),
        ("context_fit", "context_window_fit"),
    ];
    for (alias, canonical) in alias_to_canonical {
        let text = lookup(alias).unwrap_or_else(|| panic!("alias '{alias}' should resolve"));
        assert!(
            text.starts_with(&format!("{canonical}:")),
            "alias '{alias}' should resolve to canonical '{canonical}', got: {text}"
        );
    }
}

#[test]
fn given_lookup_with_sql_injection_like_input_then_returns_none() {
    assert!(lookup("'; DROP TABLE metrics; --").is_none());
    assert!(lookup("doc_density OR 1=1").is_none());
}

#[test]
fn given_lookup_with_html_tags_then_returns_none() {
    assert!(lookup("<script>alert('xss')</script>").is_none());
    assert!(lookup("<b>gini</b>").is_none());
}

#[test]
fn given_catalog_when_parsed_then_no_lines_have_trailing_whitespace() {
    let text = catalog();
    for (i, line) in text.lines().enumerate() {
        assert_eq!(
            line,
            line.trim_end(),
            "catalog line {i} has trailing whitespace"
        );
    }
}
