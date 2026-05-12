//! Shared numeric coercion for gate evaluation.

use serde_json::Value;

pub(crate) fn value_to_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn numbers_coerce_to_f64() {
        assert_eq!(value_to_f64(&json!(42)), Some(42.0));
        assert_eq!(value_to_f64(&json!(3.5)), Some(3.5));
    }

    #[test]
    fn numeric_strings_coerce_to_f64() {
        assert_eq!(value_to_f64(&json!("42")), Some(42.0));
        assert_eq!(value_to_f64(&json!("-3.5")), Some(-3.5));
    }

    #[test]
    fn non_numeric_values_do_not_coerce() {
        assert_eq!(value_to_f64(&json!("abc")), None);
        assert_eq!(value_to_f64(&json!(true)), None);
        assert_eq!(value_to_f64(&json!(null)), None);
        assert_eq!(value_to_f64(&json!({ "n": 1 })), None);
    }
}
