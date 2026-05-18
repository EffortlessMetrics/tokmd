//! Rule value comparison helpers for gate policy evaluation.

use crate::numeric::value_to_f64;
use serde_json::Value;

/// Compare two values numerically.
pub(super) fn compare_numeric<F>(
    actual: &Value,
    expected: Option<&Value>,
    cmp: F,
) -> Result<bool, &'static str>
where
    F: Fn(f64, f64) -> bool,
{
    let actual_num = value_to_f64(actual).ok_or("actual value is not numeric")?;
    let expected_num = expected
        .and_then(value_to_f64)
        .ok_or("expected value is missing or not numeric")?;
    Ok(cmp(actual_num, expected_num))
}

/// Compare two values for equality.
pub(super) fn compare_equal(
    actual: &Value,
    expected: Option<&Value>,
) -> Result<bool, &'static str> {
    let expected = expected.ok_or("expected value is missing")?;

    // For strings, compare case-sensitively before numeric coercion to avoid
    // treating special strings like "inf" and "nan" as numbers.
    if let (Some(a), Some(b)) = (actual.as_str(), expected.as_str()) {
        return Ok(a == b);
    }

    // For numeric types, compare as f64 to handle int/float mismatches.
    if let (Some(a), Some(b)) = (value_to_f64(actual), value_to_f64(expected)) {
        return Ok((a - b).abs() < f64::EPSILON);
    }

    // For other types, use JSON equality.
    Ok(actual == expected)
}

/// Check if actual value is in a list of expected values.
pub(super) fn compare_in(
    actual: &Value,
    expected: Option<&Vec<Value>>,
) -> Result<bool, &'static str> {
    let list = expected.ok_or("expected values list is missing")?;

    for item in list {
        if compare_equal(actual, Some(item)).unwrap_or(false) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Check if actual contains expected.
pub(super) fn compare_contains(
    actual: &Value,
    expected: Option<&Value>,
) -> Result<bool, &'static str> {
    let expected = expected.ok_or("expected value is missing")?;

    match actual {
        Value::String(s) => {
            let needle = expected
                .as_str()
                .ok_or("expected value must be a string for string contains checks")?;
            Ok(s.contains(needle))
        }
        Value::Array(arr) => {
            for item in arr {
                if compare_equal(item, Some(expected)).unwrap_or(false) {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        _ => Err("contains is only valid for string or array actual values"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── compare_numeric ────────────────────────────────────────────────
    #[test]
    fn compare_numeric_gt_true_when_actual_exceeds_expected() {
        let result = compare_numeric(&json!(10), Some(&json!(5)), |a, b| a > b);
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn compare_numeric_gt_false_when_actual_equals_expected() {
        let result = compare_numeric(&json!(5), Some(&json!(5)), |a, b| a > b);
        assert_eq!(result, Ok(false));
    }

    #[test]
    fn compare_numeric_gte_true_at_boundary() {
        let result = compare_numeric(&json!(5), Some(&json!(5)), |a, b| a >= b);
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn compare_numeric_lt_true_when_actual_below_expected() {
        let result = compare_numeric(&json!(3), Some(&json!(5)), |a, b| a < b);
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn compare_numeric_lte_true_at_boundary() {
        let result = compare_numeric(&json!(5), Some(&json!(5)), |a, b| a <= b);
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn compare_numeric_coerces_numeric_strings() {
        // Numeric strings should coerce via value_to_f64.
        let result = compare_numeric(&json!("12.5"), Some(&json!(10)), |a, b| a > b);
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn compare_numeric_errors_when_actual_not_numeric() {
        let result = compare_numeric(&json!("abc"), Some(&json!(1)), |a, b| a > b);
        assert_eq!(result, Err("actual value is not numeric"));
    }

    #[test]
    fn compare_numeric_errors_when_expected_missing() {
        let result = compare_numeric(&json!(1), None, |a, b| a > b);
        assert_eq!(result, Err("expected value is missing or not numeric"));
    }

    #[test]
    fn compare_numeric_errors_when_expected_not_numeric() {
        let result = compare_numeric(&json!(1), Some(&json!(true)), |a, b| a > b);
        assert_eq!(result, Err("expected value is missing or not numeric"));
    }

    #[test]
    fn compare_numeric_nan_string_breaks_ordering() {
        // "NaN" as a string coerces via str::parse::<f64>, which DOES parse to
        // f64::NAN. All ordering comparisons against NaN must return false.
        let nan = json!("NaN");
        assert_eq!(
            compare_numeric(&nan, Some(&json!(1)), |a, b| a > b),
            Ok(false)
        );
        assert_eq!(
            compare_numeric(&nan, Some(&json!(1)), |a, b| a < b),
            Ok(false)
        );
        assert_eq!(
            compare_numeric(&nan, Some(&json!(1)), |a, b| (a - b).abs() < f64::EPSILON),
            Ok(false)
        );
    }

    // ── compare_equal ──────────────────────────────────────────────────
    #[test]
    fn compare_equal_strings_match_case_sensitively() {
        assert_eq!(compare_equal(&json!("Foo"), Some(&json!("Foo"))), Ok(true));
        assert_eq!(compare_equal(&json!("Foo"), Some(&json!("foo"))), Ok(false));
    }

    #[test]
    fn compare_equal_special_numeric_strings_compared_as_strings() {
        // "inf" and "nan" must not be coerced to f64 when both sides are strings.
        assert_eq!(compare_equal(&json!("inf"), Some(&json!("inf"))), Ok(true));
        assert_eq!(compare_equal(&json!("nan"), Some(&json!("nan"))), Ok(true));
        assert_eq!(compare_equal(&json!("inf"), Some(&json!("INF"))), Ok(false));
    }

    #[test]
    fn compare_equal_numeric_int_float_mismatch_is_equal() {
        // 42 (int) == 42.0 (float) via f64 coercion.
        assert_eq!(compare_equal(&json!(42), Some(&json!(42.0))), Ok(true));
    }

    #[test]
    fn compare_equal_numeric_strings_coerce() {
        // Mixed: one side string, one side number — falls through to numeric.
        assert_eq!(compare_equal(&json!("42"), Some(&json!(42))), Ok(true));
    }

    #[test]
    fn compare_equal_epsilon_boundary_is_strict() {
        let a = 1.0_f64;
        let b = a + f64::EPSILON;
        assert_eq!(
            compare_equal(&json!(a), Some(&json!(b))),
            Ok(false),
            "difference of exactly EPSILON must not be treated as equal"
        );
    }

    #[test]
    fn compare_equal_falls_back_to_json_equality_for_arrays() {
        assert_eq!(
            compare_equal(&json!([1, 2, 3]), Some(&json!([1, 2, 3]))),
            Ok(true)
        );
        assert_eq!(
            compare_equal(&json!([1, 2, 3]), Some(&json!([3, 2, 1]))),
            Ok(false)
        );
    }

    #[test]
    fn compare_equal_falls_back_to_json_equality_for_objects() {
        assert_eq!(
            compare_equal(&json!({"a": 1}), Some(&json!({"a": 1}))),
            Ok(true)
        );
        assert_eq!(
            compare_equal(&json!({"a": 1}), Some(&json!({"a": 2}))),
            Ok(false)
        );
    }

    #[test]
    fn compare_equal_booleans_match_via_json_equality() {
        assert_eq!(compare_equal(&json!(true), Some(&json!(true))), Ok(true));
        assert_eq!(compare_equal(&json!(true), Some(&json!(false))), Ok(false));
    }

    #[test]
    fn compare_equal_null_matches_null() {
        assert_eq!(compare_equal(&json!(null), Some(&json!(null))), Ok(true));
    }

    #[test]
    fn compare_equal_type_mismatch_returns_false_not_error() {
        // string vs object — falls through to JSON equality which is false.
        assert_eq!(
            compare_equal(&json!("foo"), Some(&json!({"foo": 1}))),
            Ok(false)
        );
    }

    #[test]
    fn compare_equal_errors_when_expected_missing() {
        assert_eq!(
            compare_equal(&json!(1), None),
            Err("expected value is missing")
        );
    }

    // ── compare_in ─────────────────────────────────────────────────────
    #[test]
    fn compare_in_finds_string_member() {
        let list = vec![json!("MIT"), json!("Apache-2.0")];
        assert_eq!(compare_in(&json!("MIT"), Some(&list)), Ok(true));
    }

    #[test]
    fn compare_in_returns_false_for_non_member() {
        let list = vec![json!("MIT"), json!("Apache-2.0")];
        assert_eq!(compare_in(&json!("GPL"), Some(&list)), Ok(false));
    }

    #[test]
    fn compare_in_empty_list_is_never_member() {
        let list: Vec<Value> = vec![];
        assert_eq!(compare_in(&json!("anything"), Some(&list)), Ok(false));
    }

    #[test]
    fn compare_in_errors_when_list_missing() {
        assert_eq!(
            compare_in(&json!("MIT"), None),
            Err("expected values list is missing")
        );
    }

    #[test]
    fn compare_in_finds_numeric_member_with_coercion() {
        let list = vec![json!(1), json!(2), json!(3)];
        // int vs float should coerce.
        assert_eq!(compare_in(&json!(2.0), Some(&list)), Ok(true));
    }

    #[test]
    fn compare_in_skips_uncomparable_items_without_error() {
        // The unwrap_or(false) inside compare_in must swallow errors mid-iteration.
        // Construct a list where an early item would error against actual but a
        // later one matches via JSON equality.
        let list = vec![json!({"unexpected": "object"}), json!("target")];
        assert_eq!(compare_in(&json!("target"), Some(&list)), Ok(true));
    }

    // ── compare_contains ───────────────────────────────────────────────
    #[test]
    fn compare_contains_substring_in_string() {
        assert_eq!(
            compare_contains(&json!("hello world"), Some(&json!("world"))),
            Ok(true)
        );
    }

    #[test]
    fn compare_contains_string_misses() {
        assert_eq!(
            compare_contains(&json!("hello world"), Some(&json!("nope"))),
            Ok(false)
        );
    }

    #[test]
    fn compare_contains_empty_substring_matches_any_string() {
        // String::contains("") is always true.
        assert_eq!(compare_contains(&json!("abc"), Some(&json!(""))), Ok(true));
    }

    #[test]
    fn compare_contains_array_member_match() {
        let arr = json!(["a", "b", "c"]);
        assert_eq!(compare_contains(&arr, Some(&json!("b"))), Ok(true));
    }

    #[test]
    fn compare_contains_array_member_miss() {
        let arr = json!(["a", "b", "c"]);
        assert_eq!(compare_contains(&arr, Some(&json!("z"))), Ok(false));
    }

    #[test]
    fn compare_contains_string_actual_with_non_string_needle_errors() {
        let result = compare_contains(&json!("abc"), Some(&json!(1)));
        assert_eq!(
            result,
            Err("expected value must be a string for string contains checks")
        );
    }

    #[test]
    fn compare_contains_errors_for_non_string_non_array_actual() {
        let result = compare_contains(&json!(42), Some(&json!(1)));
        assert_eq!(
            result,
            Err("contains is only valid for string or array actual values")
        );
    }

    #[test]
    fn compare_contains_errors_when_expected_missing() {
        assert_eq!(
            compare_contains(&json!("abc"), None),
            Err("expected value is missing")
        );
    }

    #[test]
    fn compare_contains_array_with_mixed_types_uses_compare_equal() {
        // Verify integer/float coercion inside array contains.
        let arr = json!([1, 2, 3]);
        assert_eq!(compare_contains(&arr, Some(&json!(2.0))), Ok(true));
    }
}
