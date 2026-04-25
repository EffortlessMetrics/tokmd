//! Deterministic tree renderers from `ExportData`.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use tokmd_types::{ExportData, FileKind, FileRow};

#[derive(Default)]
struct AnalysisNode {
    children: BTreeMap<String, AnalysisNode>,
    lines: usize,
    tokens: usize,
    is_file: bool,
}

fn insert_analysis(node: &mut AnalysisNode, parts: &[&str], lines: usize, tokens: usize) {
    node.lines += lines;
    node.tokens += tokens;
    if let Some((head, tail)) = parts.split_first() {
        let child = node.children.entry((*head).to_string()).or_default();
        insert_analysis(child, tail, lines, tokens);
    } else {
        node.is_file = true;
    }
}

fn render_analysis(node: &AnalysisNode, name: &str, indent: &str, out: &mut String) {
    if !name.is_empty() {
        out.push_str(&format!(
            "{}{} (lines: {}, tokens: {})\n",
            indent, name, node.lines, node.tokens
        ));
    }
    let next_indent = if name.is_empty() {
        indent.to_string()
    } else {
        format!("{indent}  ")
    };
    for (child_name, child) in &node.children {
        render_analysis(child, child_name, &next_indent, out);
    }
}

/// Render the analysis tree used by `analysis.tree`.
///
/// Behavior:
/// - Includes only `FileKind::Parent` rows.
/// - Includes file leaves.
/// - Emits `(lines, tokens)` for each node.
/// - Orders siblings lexicographically for deterministic output.
#[must_use]
pub fn render_analysis_tree(export: &ExportData) -> String {
    let mut root = AnalysisNode::default();
    for row in export.rows.iter().filter(|r| r.kind == FileKind::Parent) {
        let parts: Vec<&str> = row.path.split('/').filter(|seg| !seg.is_empty()).collect();
        insert_analysis(&mut root, &parts, row.lines, row.tokens);
    }

    let mut out = String::new();
    render_analysis(&root, "", "", &mut out);
    out
}

#[derive(Default)]
struct HandoffNode {
    children: BTreeMap<String, HandoffNode>,
    files: usize,
    lines: usize,
    tokens: usize,
}

fn insert_handoff(node: &mut HandoffNode, parts: &[&str], lines: usize, tokens: usize) {
    node.files += 1;
    node.lines += lines;
    node.tokens += tokens;
    if let Some((head, tail)) = parts.split_first()
        && !tail.is_empty()
    {
        let child = node.children.entry((*head).to_string()).or_default();
        insert_handoff(child, tail, lines, tokens);
    }
}

fn render_handoff(
    node: &HandoffNode,
    name: &str,
    indent: &str,
    depth: usize,
    max_depth: usize,
    out: &mut String,
) {
    let display = if name.is_empty() {
        "".to_string()
    } else if name == "(root)" {
        name.to_string()
    } else {
        format!("{name}/")
    };

    if !display.is_empty() {
        out.push_str(&format!(
            "{}{} (files: {}, lines: {}, tokens: {})\n",
            indent, display, node.files, node.lines, node.tokens
        ));
    }

    if depth >= max_depth {
        return;
    }

    let next_indent = format!("{indent}  ");
    for (child_name, child) in &node.children {
        render_handoff(child, child_name, &next_indent, depth + 1, max_depth, out);
    }
}

/// Render the handoff intelligence tree.
///
/// Behavior:
/// - Includes only `FileKind::Parent` rows.
/// - Includes root line and directory nodes only (no file leaves).
/// - Emits `(files, lines, tokens)` for each node.
/// - Stops descending at `max_depth`.
/// - Orders siblings lexicographically for deterministic output.
#[must_use]
pub fn render_handoff_tree(export: &ExportData, max_depth: usize) -> String {
    let parents: Vec<&FileRow> = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .collect();
    if parents.is_empty() {
        return String::new();
    }

    let mut root = HandoffNode::default();
    for row in parents {
        let parts: Vec<&str> = row.path.split('/').filter(|seg| !seg.is_empty()).collect();
        insert_handoff(&mut root, &parts, row.lines, row.tokens);
    }

    let mut out = String::new();
    render_handoff(&root, "(root)", "", 0, max_depth, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use tokmd_types::ChildIncludeMode;

    use super::*;

    fn row(path: &str, kind: FileKind, lines: usize, tokens: usize) -> FileRow {
        FileRow {
            path: path.to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind,
            code: lines,
            comments: 0,
            blanks: 0,
            lines,
            bytes: lines * 10,
            tokens,
        }
    }

    fn export(rows: Vec<FileRow>) -> ExportData {
        ExportData {
            rows,
            module_roots: vec!["crates".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::ParentsOnly,
        }
    }

    #[test]
    fn analysis_tree_empty_export_returns_empty() {
        let out = render_analysis_tree(&export(vec![]));
        assert!(out.is_empty());
    }

    #[test]
    fn analysis_tree_includes_file_leaves() {
        let out = render_analysis_tree(&export(vec![row("src/main.rs", FileKind::Parent, 12, 24)]));
        assert!(out.contains("src (lines: 12, tokens: 24)"));
        assert!(out.contains("main.rs (lines: 12, tokens: 24)"));
    }

    #[test]
    fn analysis_tree_ignores_child_rows() {
        let out = render_analysis_tree(&export(vec![
            row("src/main.rs", FileKind::Parent, 12, 24),
            row("src/main.rs::embedded", FileKind::Child, 30, 90),
        ]));
        assert!(out.contains("main.rs (lines: 12, tokens: 24)"));
        assert!(!out.contains("embedded"));
    }

    #[test]
    fn handoff_tree_empty_export_returns_empty() {
        let out = render_handoff_tree(&export(vec![]), 3);
        assert!(out.is_empty());
    }

    #[test]
    fn handoff_tree_depth_limit_and_no_file_leaves() {
        let out = render_handoff_tree(
            &export(vec![row("a/b/c/file.rs", FileKind::Parent, 10, 20)]),
            1,
        );
        assert!(out.contains("(root) (files: 1, lines: 10, tokens: 20)"));
        assert!(out.contains("a/ (files: 1, lines: 10, tokens: 20)"));
        assert!(!out.contains("b/"));
        assert!(!out.contains("file.rs"));
    }
}
