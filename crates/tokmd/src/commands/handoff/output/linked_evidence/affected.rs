//! Affected-proof report summary.

use serde_json::Value;

pub(in crate::commands::handoff) struct AffectedSummary {
    pub(in crate::commands::handoff) changed_files: usize,
    pub(in crate::commands::handoff) scopes: usize,
    pub(in crate::commands::handoff) unknown_files: usize,
    pub(in crate::commands::handoff) changed_file_paths: Vec<String>,
    pub(in crate::commands::handoff) scope_names: Vec<String>,
}

pub(super) fn summarize(value: &Value) -> AffectedSummary {
    let changed_files = array_len(value.get("changed_files"));
    let changed_file_paths = value
        .get("changed_files")
        .and_then(Value::as_array)
        .into_iter()
        .flat_map(|paths| paths.iter())
        .filter_map(Value::as_str)
        .take(10)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let scopes_array = value.get("scopes").and_then(Value::as_array);
    let scope_names = scopes_array
        .into_iter()
        .flat_map(|scopes| scopes.iter())
        .filter_map(|scope| scope.get("name").and_then(Value::as_str))
        .take(8)
        .map(str::to_string)
        .collect::<Vec<_>>();

    AffectedSummary {
        changed_files,
        scopes: array_len(value.get("scopes")),
        unknown_files: array_len(value.get("unknown_files")),
        changed_file_paths,
        scope_names,
    }
}

pub(super) fn render(out: &mut String, affected: &AffectedSummary) {
    out.push_str(&format!(
        "- Affected proof: {} changed file(s), {} scope(s), {} unknown file(s)\n",
        affected.changed_files, affected.scopes, affected.unknown_files
    ));
    if !affected.scope_names.is_empty() {
        out.push_str("  - Scopes: ");
        out.push_str(&affected.scope_names.join(", "));
        out.push('\n');
    }
}

fn array_len(value: Option<&Value>) -> usize {
    value.and_then(Value::as_array).map_or(0, Vec::len)
}
