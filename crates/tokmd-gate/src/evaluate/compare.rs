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
