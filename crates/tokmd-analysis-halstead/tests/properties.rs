//! Property-based tests for Halstead metrics using proptest.

use proptest::prelude::*;
use tokmd_analysis_halstead::{
    is_halstead_lang, operators_for_lang, round_f64, tokenize_for_halstead,
};

// ── strategies ───────────────────────────────────────────────────────

fn arb_supported_lang() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("rust"),
        Just("javascript"),
        Just("typescript"),
        Just("python"),
        Just("go"),
        Just("c"),
        Just("c++"),
        Just("java"),
        Just("c#"),
        Just("php"),
        Just("ruby"),
    ]
}

fn arb_rust_snippet() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("fn main() {}".to_string()),
        Just("let x = 1 + 2;".to_string()),
        Just("if a > b { a } else { b }".to_string()),
        Just("match x { 0 => true, _ => false }".to_string()),
        Just("for i in 0..10 { let y = i * 2; }".to_string()),
        Just("struct Foo { x: i32, y: i32 }".to_string()),
        Just("impl Foo { fn new() -> Self { Foo { x: 0, y: 0 } } }".to_string()),
        Just("pub fn add(a: i32, b: i32) -> i32 { a + b }".to_string()),
        Just("while x > 0 { x -= 1; }".to_string()),
        Just("let mut v = Vec::new();".to_string()),
    ]
}

// ── property: tokenize totals are consistent ─────────────────────────

proptest! {
    #[test]
    fn prop_total_operators_equals_sum_of_individual_counts(
        code in arb_rust_snippet()
    ) {
        let counts = tokenize_for_halstead(&code, "rust");
        let sum: usize = counts.operators.values().sum();
        prop_assert_eq!(
            counts.total_operators, sum,
            "total_operators must equal sum of individual operator counts"
        );
    }

    #[test]
    fn prop_total_operands_gte_distinct_operands(
        code in arb_rust_snippet()
    ) {
        let counts = tokenize_for_halstead(&code, "rust");
        prop_assert!(
            counts.total_operands >= counts.operands.len(),
            "total operands ({}) must be >= distinct operands ({})",
            counts.total_operands, counts.operands.len()
        );
    }

    #[test]
    fn prop_distinct_operators_lte_total_operators(
        code in arb_rust_snippet()
    ) {
        let counts = tokenize_for_halstead(&code, "rust");
        prop_assert!(
            counts.operators.len() <= counts.total_operators,
            "distinct operators ({}) must be <= total operators ({})",
            counts.operators.len(), counts.total_operators
        );
    }
}

// ── property: empty or comment-only input ────────────────────────────

proptest! {
    #[test]
    fn prop_empty_string_produces_zero_for_any_lang(
        lang in arb_supported_lang()
    ) {
        let counts = tokenize_for_halstead("", lang);
        prop_assert_eq!(counts.total_operators, 0);
        prop_assert_eq!(counts.total_operands, 0);
    }

    #[test]
    fn prop_comment_only_produces_zero_for_slash_langs(
        lines in prop::collection::vec("//[a-zA-Z0-9 ]{0,40}", 1..5)
    ) {
        let code = lines.join("\n");
        let counts = tokenize_for_halstead(&code, "rust");
        prop_assert_eq!(counts.total_operators, 0);
        prop_assert_eq!(counts.total_operands, 0);
    }

    #[test]
    fn prop_hash_comment_only_produces_zero(
        lines in prop::collection::vec("#[a-zA-Z0-9 ]{0,40}", 1..5)
    ) {
        let code = lines.join("\n");
        let counts = tokenize_for_halstead(&code, "python");
        prop_assert_eq!(counts.total_operators, 0);
        prop_assert_eq!(counts.total_operands, 0);
    }
}

// ── property: Halstead metric invariants ─────────────────────────────

proptest! {
    #[test]
    fn prop_vocabulary_is_sum_of_distinct(
        code in arb_rust_snippet()
    ) {
        let counts = tokenize_for_halstead(&code, "rust");
        let n1 = counts.operators.len();
        let n2 = counts.operands.len();
        let vocabulary = n1 + n2;
        let length = counts.total_operators + counts.total_operands;

        // vocabulary = n1 + n2
        prop_assert_eq!(vocabulary, n1 + n2);
        // length = N1 + N2
        prop_assert_eq!(length, counts.total_operators + counts.total_operands);
        // length >= vocabulary (pigeonhole: if every token is distinct, length == vocabulary)
        prop_assert!(length >= vocabulary || (length == 0 && vocabulary == 0));
    }

    #[test]
    fn prop_volume_non_negative(
        n1 in 0usize..100,
        n2 in 0usize..100,
        total_ops in 0usize..1000,
        total_opds in 0usize..1000,
    ) {
        let vocabulary = n1 + n2;
        let length = total_ops + total_opds;
        let volume = if vocabulary > 0 {
            length as f64 * (vocabulary as f64).log2()
        } else {
            0.0
        };
        prop_assert!(volume >= 0.0, "volume must be non-negative, got {volume}");
    }

    #[test]
    fn prop_difficulty_non_negative(
        n1 in 0usize..100,
        n2 in 1usize..100, // n2 > 0 to avoid div-by-zero
        total_opds in 0usize..1000,
    ) {
        let difficulty = (n1 as f64 / 2.0) * (total_opds as f64 / n2 as f64);
        prop_assert!(difficulty >= 0.0, "difficulty must be non-negative, got {difficulty}");
    }

    #[test]
    fn prop_effort_equals_difficulty_times_volume(
        n1 in 1usize..50,
        n2 in 1usize..50,
        total_ops in 1usize..500,
        total_opds in 1usize..500,
    ) {
        let vocabulary = n1 + n2;
        let length = total_ops + total_opds;
        let volume = length as f64 * (vocabulary as f64).log2();
        let difficulty = (n1 as f64 / 2.0) * (total_opds as f64 / n2 as f64);
        let effort = difficulty * volume;

        let recomputed = difficulty * volume;
        prop_assert!(
            (effort - recomputed).abs() < 1e-10,
            "effort must equal difficulty * volume"
        );
    }

    #[test]
    fn prop_time_equals_effort_over_18(
        effort in 0.0f64..1_000_000.0,
    ) {
        let time = effort / 18.0;
        prop_assert!(
            (time * 18.0 - effort).abs() < 1e-6,
            "time_seconds * 18 must approximately equal effort"
        );
    }

    #[test]
    fn prop_bugs_equals_volume_over_3000(
        volume in 0.0f64..1_000_000.0,
    ) {
        let bugs = volume / 3000.0;
        prop_assert!(
            (bugs * 3000.0 - volume).abs() < 1e-6,
            "estimated_bugs * 3000 must approximately equal volume"
        );
    }
}

// ── property: round_f64 ──────────────────────────────────────────────

proptest! {
    #[test]
    fn prop_round_preserves_integer_values(
        n in -1000i64..1000,
        decimals in 0u32..10,
    ) {
        let val = n as f64;
        let rounded = round_f64(val, decimals);
        prop_assert!(
            (rounded - val).abs() < 1e-10,
            "rounding an integer {val} with {decimals} decimals should preserve it, got {rounded}"
        );
    }

    #[test]
    fn prop_round_result_within_half_unit(
        val in -10000.0f64..10000.0,
        decimals in 0u32..6,
    ) {
        let rounded = round_f64(val, decimals);
        let factor = 10f64.powi(decimals as i32);
        let diff = (rounded - val).abs();
        // Difference should be at most half a unit in the last place
        prop_assert!(
            diff <= 0.5 / factor + 1e-12,
            "round_f64({val}, {decimals}) = {rounded}, diff {diff} exceeds 0.5/{factor}"
        );
    }

    #[test]
    fn prop_round_zero_decimals_is_integer(
        val in -10000.0f64..10000.0,
    ) {
        let rounded = round_f64(val, 0);
        prop_assert_eq!(rounded, rounded.round(), "0-decimal rounding must be integer");
    }

    #[test]
    fn prop_round_idempotent(
        val in -10000.0f64..10000.0,
        decimals in 0u32..6,
    ) {
        let once = round_f64(val, decimals);
        let twice = round_f64(once, decimals);
        prop_assert!(
            (once - twice).abs() < 1e-12,
            "round_f64 should be idempotent: round({val}, {decimals}) = {once}, re-round = {twice}"
        );
    }
}

// ── property: operator tables ────────────────────────────────────────

proptest! {
    #[test]
    fn prop_supported_lang_has_nonempty_ops(
        lang in arb_supported_lang()
    ) {
        let ops = operators_for_lang(lang);
        prop_assert!(!ops.is_empty(), "{lang} should have operators");
    }

    #[test]
    fn prop_is_halstead_lang_consistent_with_operators(
        lang in arb_supported_lang()
    ) {
        // If a language is supported, it must have operators
        prop_assert!(is_halstead_lang(lang));
        prop_assert!(!operators_for_lang(lang).is_empty());
    }

    #[test]
    fn prop_operator_table_has_no_duplicates(
        lang in arb_supported_lang()
    ) {
        let ops = operators_for_lang(lang);
        let unique: std::collections::BTreeSet<&str> = ops.iter().copied().collect();
        prop_assert_eq!(
            ops.len(), unique.len(),
            "operator table has duplicates: {} entries but {} unique",
            ops.len(), unique.len()
        );
    }
}

// ── property: tokenizer determinism ──────────────────────────────────

proptest! {
    #[test]
    fn prop_tokenize_deterministic(
        code in arb_rust_snippet()
    ) {
        let a = tokenize_for_halstead(&code, "rust");
        let b = tokenize_for_halstead(&code, "rust");
        prop_assert_eq!(a.total_operators, b.total_operators);
        prop_assert_eq!(a.total_operands, b.total_operands);
        prop_assert_eq!(a.operators, b.operators);
        prop_assert_eq!(a.operands, b.operands);
    }

    #[test]
    fn prop_adding_blank_lines_does_not_change_counts(
        code in arb_rust_snippet(),
        blanks in 1usize..5,
    ) {
        let padded = format!("{}\n{}", "\n".repeat(blanks), code);
        let orig = tokenize_for_halstead(&code, "rust");
        let with_blanks = tokenize_for_halstead(&padded, "rust");
        prop_assert_eq!(
            orig.total_operators, with_blanks.total_operators,
            "blank lines should not change operator count"
        );
        prop_assert_eq!(
            orig.total_operands, with_blanks.total_operands,
            "blank lines should not change operand count"
        );
    }
}

// ── property: Halstead metrics from tokenization are non-negative ────

proptest! {
    #[test]
    fn prop_derived_halstead_metrics_non_negative(
        code in arb_rust_snippet()
    ) {
        let counts = tokenize_for_halstead(&code, "rust");
        let n1 = counts.operators.len();
        let n2 = counts.operands.len();
        let vocabulary = n1 + n2;
        let length = counts.total_operators + counts.total_operands;

        let volume = if vocabulary > 0 {
            length as f64 * (vocabulary as f64).log2()
        } else {
            0.0
        };
        let difficulty = if n2 > 0 {
            (n1 as f64 / 2.0) * (counts.total_operands as f64 / n2 as f64)
        } else {
            0.0
        };
        let effort = difficulty * volume;
        let time_seconds = effort / 18.0;
        let estimated_bugs = volume / 3000.0;

        prop_assert!(volume >= 0.0, "volume must be non-negative, got {volume}");
        prop_assert!(difficulty >= 0.0, "difficulty must be non-negative, got {difficulty}");
        prop_assert!(effort >= 0.0, "effort must be non-negative, got {effort}");
        prop_assert!(time_seconds >= 0.0, "time must be non-negative, got {time_seconds}");
        prop_assert!(estimated_bugs >= 0.0, "bugs must be non-negative, got {estimated_bugs}");
    }
}

// ── property: more code produces higher or equal metrics ─────────────

proptest! {
    #[test]
    fn prop_more_code_more_or_equal_length(
        snippet_a in arb_rust_snippet(),
        snippet_b in arb_rust_snippet(),
    ) {
        let combined = format!("{}\n{}", snippet_a, snippet_b);
        let counts_a = tokenize_for_halstead(&snippet_a, "rust");
        let counts_combined = tokenize_for_halstead(&combined, "rust");

        // Combined code should have >= operators and operands as first snippet alone
        prop_assert!(
            counts_combined.total_operators + counts_combined.total_operands
                >= counts_a.total_operators + counts_a.total_operands,
            "combined length should be >= single snippet length"
        );
    }
}

// ── property: tokenize across multiple languages yields consistent structure

proptest! {
    #[test]
    fn prop_multi_lang_tokenize_consistent(
        lang in arb_supported_lang()
    ) {
        // Simple code that has identifiers and operators
        let code = "x = 1 + 2";
        let counts = tokenize_for_halstead(code, lang);

        // total_operators must equal sum of individual operator counts
        let op_sum: usize = counts.operators.values().sum();
        prop_assert_eq!(counts.total_operators, op_sum);

        // total_operands >= distinct operands
        prop_assert!(counts.total_operands >= counts.operands.len());
    }
}
