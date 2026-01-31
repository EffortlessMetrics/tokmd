//! RFC 6901 JSON Pointer implementation.

use serde_json::Value;

/// Resolve a JSON Pointer against a JSON value.
///
/// Implements RFC 6901 JSON Pointer syntax:
/// - Empty string "" refers to the whole document
/// - "/" refers to an empty key
/// - "/foo/bar" navigates to obj["foo"]["bar"]
/// - "/0" navigates to array index 0
/// - "~0" escapes to "~"
/// - "~1" escapes to "/"
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use tokmd_gate::resolve_pointer;
///
/// let doc = json!({"foo": {"bar": 42}});
/// assert_eq!(resolve_pointer(&doc, "/foo/bar"), Some(&json!(42)));
///
/// let arr = json!({"items": [1, 2, 3]});
/// assert_eq!(resolve_pointer(&arr, "/items/1"), Some(&json!(2)));
/// ```
pub fn resolve_pointer<'a>(value: &'a Value, pointer: &str) -> Option<&'a Value> {
    // Empty pointer refers to whole document
    if pointer.is_empty() {
        return Some(value);
    }

    // Pointer must start with /
    if !pointer.starts_with('/') {
        return None;
    }

    let mut current = value;

    for token in pointer[1..].split('/') {
        // Unescape tokens per RFC 6901
        let unescaped = unescape_token(token);

        current = match current {
            Value::Object(map) => map.get(&unescaped)?,
            Value::Array(arr) => {
                // Try to parse as array index
                let idx: usize = unescaped.parse().ok()?;
                arr.get(idx)?
            }
            _ => return None,
        };
    }

    Some(current)
}

/// Unescape a JSON Pointer token per RFC 6901.
/// ~1 -> /
/// ~0 -> ~
fn unescape_token(token: &str) -> String {
    token.replace("~1", "/").replace("~0", "~")
}

/// Escape a string for use in a JSON Pointer.
/// / -> ~1
/// ~ -> ~0
#[allow(dead_code)]
pub fn escape_token(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_empty_pointer() {
        let doc = json!({"foo": 1});
        assert_eq!(resolve_pointer(&doc, ""), Some(&doc));
    }

    #[test]
    fn test_simple_path() {
        let doc = json!({"foo": {"bar": 42}});
        assert_eq!(resolve_pointer(&doc, "/foo"), Some(&json!({"bar": 42})));
        assert_eq!(resolve_pointer(&doc, "/foo/bar"), Some(&json!(42)));
    }

    #[test]
    fn test_array_index() {
        let doc = json!({"items": [10, 20, 30]});
        assert_eq!(resolve_pointer(&doc, "/items/0"), Some(&json!(10)));
        assert_eq!(resolve_pointer(&doc, "/items/2"), Some(&json!(30)));
        assert_eq!(resolve_pointer(&doc, "/items/3"), None);
    }

    #[test]
    fn test_escaped_tokens() {
        let doc = json!({"a/b": {"c~d": 1}});
        assert_eq!(resolve_pointer(&doc, "/a~1b/c~0d"), Some(&json!(1)));
    }

    #[test]
    fn test_invalid_pointer() {
        let doc = json!({"foo": 1});
        // Missing leading slash
        assert_eq!(resolve_pointer(&doc, "foo"), None);
        // Non-existent path
        assert_eq!(resolve_pointer(&doc, "/bar"), None);
    }

    #[test]
    fn test_nested_arrays() {
        let doc = json!({"matrix": [[1, 2], [3, 4]]});
        assert_eq!(resolve_pointer(&doc, "/matrix/0/1"), Some(&json!(2)));
        assert_eq!(resolve_pointer(&doc, "/matrix/1/0"), Some(&json!(3)));
    }

    #[test]
    fn test_escape_token() {
        assert_eq!(escape_token("a/b"), "a~1b");
        assert_eq!(escape_token("a~b"), "a~0b");
        assert_eq!(escape_token("a/b~c"), "a~1b~0c");
    }
}
