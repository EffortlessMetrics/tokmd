use anyhow::{Context, Result, bail};
use serde_json::Value;

pub(super) fn expect_schema(value: &Value, expected: &str, label: &str) -> Result<()> {
    expect_string_value(field(value, "schema", label)?, expected, "schema", label)
}

pub(super) fn expect_equal(summary: &Value, manifest: &Value, path: &str) -> Result<()> {
    let summary_value = field(summary, path, "executor summary")?;
    let manifest_value = field(manifest, path, "executor manifest")?;
    if summary_value != manifest_value {
        bail!(
            "executor artifact mismatch at `{path}`: summary {} != manifest {}",
            render_json(summary_value),
            render_json(manifest_value)
        );
    }
    Ok(())
}

pub(super) fn field<'a>(value: &'a Value, path: &str, label: &str) -> Result<&'a Value> {
    let mut current = value;
    for segment in path.split('.') {
        current = current
            .get(segment)
            .with_context(|| format!("{label} artifact is missing `{path}`"))?;
    }
    Ok(current)
}

pub(super) fn expect_array<'a>(
    value: &'a Value,
    path: &str,
    label: &str,
) -> Result<&'a Vec<Value>> {
    value
        .as_array()
        .with_context(|| format!("{label} `{path}` must be an array"))
}

pub(super) fn expect_bool(value: &Value, path: &str, label: &str) -> Result<bool> {
    value
        .as_bool()
        .with_context(|| format!("{label} `{path}` must be a boolean"))
}

pub(super) fn expect_bool_value(
    value: &Value,
    expected: bool,
    path: &str,
    label: &str,
) -> Result<()> {
    let actual = expect_bool(value, path, label)?;
    if actual != expected {
        bail!("{label} `{path}` must be {expected}, got {actual}");
    }
    Ok(())
}

pub(super) fn expect_string(value: &Value, path: &str, label: &str) -> Result<String> {
    value
        .as_str()
        .map(ToOwned::to_owned)
        .with_context(|| format!("{label} `{path}` must be a string"))
}

pub(super) fn expect_optional_string(
    value: &Value,
    path: &str,
    label: &str,
) -> Result<Option<String>> {
    if value.is_null() {
        Ok(None)
    } else {
        expect_string(value, path, label).map(Some)
    }
}

pub(super) fn expect_string_value(
    value: &Value,
    expected: &str,
    path: &str,
    label: &str,
) -> Result<()> {
    let actual = expect_string(value, path, label)?;
    if actual != expected {
        bail!("{label} `{path}` must be `{expected}`, got `{actual}`");
    }
    Ok(())
}

pub(super) fn expect_string_array(value: &Value, path: &str, label: &str) -> Result<Vec<String>> {
    let values = expect_array(value, path, label)?;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            expect_string(value, &format!("{path}[{index}]"), label)
                .with_context(|| format!("{label} `{path}` entry {index} must be a string"))
        })
        .collect()
}

pub(super) fn expect_optional_i64(value: &Value, path: &str, label: &str) -> Result<Option<i64>> {
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_i64()
        .map(Some)
        .with_context(|| format!("{label} `{path}` must be an integer or null"))
}

pub(super) fn expect_usize(value: &Value, path: &str, label: &str) -> Result<usize> {
    let number = value
        .as_u64()
        .with_context(|| format!("{label} `{path}` must be a non-negative integer"))?;
    usize::try_from(number).with_context(|| format!("{label} `{path}` is too large"))
}

pub(super) fn render_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unrenderable>".to_string())
}
