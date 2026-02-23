#![no_main]

use libfuzzer_sys::fuzz_target;
use tokmd_export_tree::{render_analysis_tree, render_handoff_tree};
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

const MAX_INPUT_SIZE: usize = 16 * 1024;
const MAX_ROWS: usize = 256;
const MAX_PATH_LEN: usize = 256;

fn module_from_path(path: &str) -> String {
    path.split('/')
        .find(|segment| !segment.is_empty())
        .map_or_else(|| "(root)".to_string(), ToString::to_string)
}

fn build_export_data(payload: &[u8]) -> ExportData {
    let text = String::from_utf8_lossy(payload);
    let mut rows = Vec::new();

    for (idx, line) in text.lines().take(MAX_ROWS).enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let normalized = trimmed.replace('\\', "/");
        let truncated: String = normalized.chars().take(MAX_PATH_LEN).collect();
        let path = if truncated.contains('/') || truncated.contains('.') {
            truncated
        } else {
            format!("src/{truncated}.rs")
        };

        let kind = if idx % 5 == 0 {
            FileKind::Child
        } else {
            FileKind::Parent
        };
        let code = (path.len() % 300).saturating_add(1);
        let comments = idx % 17;
        let blanks = idx % 7;
        let lines = code.saturating_add(comments).saturating_add(blanks);
        let bytes = path.len().saturating_mul(4);
        let tokens = (path.bytes().map(usize::from).sum::<usize>() % 2000).saturating_add(1);

        rows.push(FileRow {
            path: path.clone(),
            module: module_from_path(&path),
            lang: "Rust".to_string(),
            kind,
            code,
            comments,
            blanks,
            lines,
            bytes,
            tokens,
        });
    }

    ExportData {
        rows,
        module_roots: vec!["crates".to_string()],
        module_depth: 2,
        children: ChildIncludeMode::ParentsOnly,
    }
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > MAX_INPUT_SIZE {
        return;
    }

    let depth = usize::from(data[0] % 8);
    let export = build_export_data(&data[1..]);

    let analysis_1 = render_analysis_tree(&export);
    let analysis_2 = render_analysis_tree(&export);
    let handoff_1 = render_handoff_tree(&export, depth);
    let handoff_2 = render_handoff_tree(&export, depth);

    assert_eq!(
        analysis_1, analysis_2,
        "analysis tree must be deterministic"
    );
    assert_eq!(handoff_1, handoff_2, "handoff tree must be deterministic");

    let parent_count = export
        .rows
        .iter()
        .filter(|row| row.kind == FileKind::Parent)
        .count();
    if parent_count == 0 {
        assert!(analysis_1.is_empty());
        assert!(handoff_1.is_empty());
    }

    assert!(
        !analysis_1.contains('\\'),
        "analysis tree should be slash-normalized"
    );
    assert!(
        !handoff_1.contains('\\'),
        "handoff tree should be slash-normalized"
    );
});
