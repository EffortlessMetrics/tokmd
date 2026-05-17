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

    #[test]
    fn compare_equal_numeric_int_float_and_string_forms() {
        assert!(compare_equal(&json!(42), Some(&json!(42.0))).unwrap());
        assert!(compare_equal(&json!("42"), Some(&json!(42))).unwrap());
        assert!(!compare_equal(&json!("42.1"), Some(&json!(42))).unwrap());
    }

    #[test]
    fn compare_equal_keeps_string_special_values_literal() {
        assert!(compare_equal(&json!("inf"), Some(&json!("inf"))).unwrap());
        assert!(!compare_equal(&json!("inf"), Some(&json!("Infinity"))).unwrap());
    }

    #[test]
    fn compare_equal_falls_back_to_json_equality_for_containers() {
        assert!(compare_equal(&json!([1, {"a": true}]), Some(&json!([1, {"a": true}]))).unwrap());
        assert!(!compare_equal(&json!({"a": 1}), Some(&json!({"a": "1"}))).unwrap());
    }

    #[test]
    fn compare_numeric_reports_missing_or_non_numeric_expected() {
        assert_eq!(
            compare_numeric(&json!(10), None, |a, b| a <= b),
            Err("expected value is missing or not numeric")
        );
        assert_eq!(
            compare_numeric(&json!(10), Some(&json!("many")), |a, b| a <= b),
            Err("expected value is missing or not numeric")
        );
    }

    #[test]
    fn compare_numeric_reports_non_numeric_actual() {
        assert_eq!(
            compare_numeric(&json!([10]), Some(&json!(10)), |a, b| a <= b),
            Err("actual value is not numeric")
        );
    }

    #[test]
    fn compare_in_uses_same_equality_semantics_as_eq() {
        let values = vec![json!(1), json!("two"), json!({"nested": true})];

        assert!(compare_in(&json!(1.0), Some(&values)).unwrap());
        assert!(compare_in(&json!({"nested": true}), Some(&values)).unwrap());
        assert!(!compare_in(&json!("missing"), Some(&values)).unwrap());
    }

    #[test]
    fn compare_contains_validates_actual_and_expected_types() {
        assert_eq!(
            compare_contains(&json!(123), Some(&json!("2"))),
            Err("contains is only valid for string or array actual values")
        );
        assert_eq!(
            compare_contains(&json!("abc"), Some(&json!(2))),
            Err("expected value must be a string for string contains checks")
        );
    }

    #[test]
    fn compare_contains_array_uses_equality_semantics() {
        let actual = json!(["1", {"lang": "Rust"}, 3]);

        assert!(compare_contains(&actual, Some(&json!(1))).unwrap());
        assert!(compare_contains(&actual, Some(&json!({"lang": "Rust"}))).unwrap());
        assert!(!compare_contains(&actual, Some(&json!({"lang": "Go"}))).unwrap());
    }
}
