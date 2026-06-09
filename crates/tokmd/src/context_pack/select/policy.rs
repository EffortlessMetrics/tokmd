//! Inclusion-policy preparation for context file selection.

use std::collections::{BTreeMap, BTreeSet};

use tokmd_scan::normalize_slashes as normalize_path;
use tokmd_types::{
    ContextFileRow, FileClassification, FileKind, FileRow, InclusionPolicy, PolicyExcludedFile,
};

use super::{SelectOptions, assign_policy, classify_file, compute_file_cap};

struct FileContextMeta {
    classifications: Vec<FileClassification>,
    policy: InclusionPolicy,
    policy_reason: Option<String>,
    original_tokens: usize,
}

pub(super) struct PolicySelection {
    pub(super) pack_rows: Vec<FileRow>,
    pub(super) excluded_by_policy: Vec<PolicyExcludedFile>,
    file_meta_map: BTreeMap<String, FileContextMeta>,
}

pub(super) fn prepare_policy_selection(
    candidate_rows: &[FileRow],
    budget: usize,
    options: &SelectOptions,
) -> PolicySelection {
    let file_cap = compute_file_cap(budget, options);
    let mut file_meta_map: BTreeMap<String, FileContextMeta> = BTreeMap::new();
    let mut excluded_by_policy: Vec<PolicyExcludedFile> = Vec::new();

    for row in candidate_rows
        .iter()
        .filter(|row| row.kind == FileKind::Parent)
    {
        let path = normalize_path(&row.path);
        let classifications = classify_file(&path, row.tokens, row.lines, options.dense_threshold);
        let (policy, reason) = assign_policy(row.tokens, file_cap, &classifications);

        file_meta_map.insert(
            path.to_string(),
            FileContextMeta {
                classifications: classifications.clone(),
                policy,
                policy_reason: reason.clone(),
                original_tokens: row.tokens,
            },
        );

        if matches!(policy, InclusionPolicy::Skip | InclusionPolicy::Summary) {
            excluded_by_policy.push(PolicyExcludedFile {
                path: path.to_string(),
                original_tokens: row.tokens,
                policy,
                reason: reason.unwrap_or_default(),
                classifications,
            });
        }
    }

    let excluded_paths: BTreeSet<&str> = excluded_by_policy
        .iter()
        .map(|file| file.path.as_ref())
        .collect();

    let pack_rows = candidate_rows
        .iter()
        .filter(|row| {
            if row.kind != FileKind::Parent {
                return true;
            }
            let path = normalize_path(&row.path);
            !excluded_paths.contains(path.as_ref())
        })
        .map(|row| {
            let path = normalize_path(&row.path);
            if let Some(meta) = file_meta_map.get(path.as_ref())
                && meta.policy == InclusionPolicy::HeadTail
            {
                return FileRow {
                    tokens: row.tokens.min(file_cap),
                    ..row.clone()
                };
            }
            row.clone()
        })
        .collect();

    PolicySelection {
        pack_rows,
        excluded_by_policy,
        file_meta_map,
    }
}

impl PolicySelection {
    pub(super) fn annotate_selected(&self, selected: &mut [ContextFileRow]) {
        for file in selected {
            let path = normalize_path(&file.path);
            if let Some(meta) = self.file_meta_map.get(path.as_ref()) {
                file.classifications = meta.classifications.clone();
                file.policy = meta.policy;
                file.policy_reason = meta.policy_reason.clone();
                if meta.policy == InclusionPolicy::HeadTail {
                    file.effective_tokens = Some(file.tokens);
                    file.tokens = meta.original_tokens;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file_row(path: &str, tokens: usize, lines: usize) -> FileRow {
        FileRow {
            path: path.to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: lines,
            comments: 0,
            blanks: 0,
            lines,
            bytes: tokens,
            tokens,
        }
    }

    fn selected_row(path: &str, tokens: usize) -> ContextFileRow {
        ContextFileRow {
            path: path.to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            tokens,
            code: 1,
            lines: 1,
            bytes: tokens,
            value: tokens,
            rank_reason: "test".to_string(),
            policy: InclusionPolicy::Full,
            effective_tokens: None,
            policy_reason: None,
            classifications: Vec::new(),
        }
    }

    #[test]
    fn head_tail_rows_are_capped_for_packing_and_restored_on_annotation() {
        let options = SelectOptions {
            max_file_pct: 0.10,
            max_file_tokens: Some(100),
            ..Default::default()
        };
        let selection =
            prepare_policy_selection(&[file_row("src/big.rs", 150, 150)], 1_000, &options);

        assert_eq!(selection.excluded_by_policy.len(), 0);
        assert_eq!(selection.pack_rows[0].tokens, 100);

        let mut selected = vec![selected_row("src/big.rs", 100)];
        selection.annotate_selected(&mut selected);
        assert_eq!(selected[0].policy, InclusionPolicy::HeadTail);
        assert_eq!(selected[0].effective_tokens, Some(100));
        assert_eq!(selected[0].tokens, 150);
    }

    #[test]
    fn oversized_generated_rows_are_excluded_from_packing() {
        let options = SelectOptions {
            max_file_pct: 0.10,
            max_file_tokens: Some(100),
            ..Default::default()
        };
        let selection = prepare_policy_selection(
            &[file_row("src/generated.pb.rs", 150, 150)],
            1_000,
            &options,
        );

        assert!(selection.pack_rows.is_empty());
        assert_eq!(selection.excluded_by_policy.len(), 1);
        assert_eq!(
            selection.excluded_by_policy[0].policy,
            InclusionPolicy::Skip
        );
    }
}
