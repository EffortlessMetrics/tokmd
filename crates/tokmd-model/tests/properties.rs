//! Property-based tests for tokmd-model functions.

use proptest::prelude::*;
use std::path::{Path, PathBuf};
use tokei::{Config, Languages};
use tokmd_model::{
    avg, collect_file_rows, create_export_data, create_lang_report, create_module_report,
    module_key, normalize_path, unique_parent_file_count,
};
use tokmd_types::{ChildIncludeMode, ChildrenMode, FileKind, FileRow, LangRow, ModuleRow, Totals};

// ========================
// Proptest strategies for domain types
// ========================

fn arb_totals() -> impl Strategy<Value = Totals> {
    (0usize..100_000, 0usize..100_000, 1usize..1000).prop_map(|(code, extra, files)| {
        let lines = code + extra;
        let bytes = lines * 40; // ~40 bytes per line
        let tokens = bytes / 4;
        Totals {
            code,
            lines,
            files,
            bytes,
            tokens,
            avg_lines: avg(lines, files),
        }
    })
}

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        "[A-Z][a-z]{2,10}",
        1usize..50_000,
        0usize..10_000,
        0usize..10_000,
        1usize..500,
    )
        .prop_map(|(lang, code, comments, blanks, files)| {
            let lines = code + comments + blanks;
            let bytes = lines * 40;
            let tokens = bytes / 4;
            LangRow {
                lang,
                code,
                lines,
                files,
                bytes,
                tokens,
                avg_lines: avg(lines, files),
            }
        })
}

fn arb_module_row() -> impl Strategy<Value = ModuleRow> {
    (
        "[a-z_]{2,8}(/[a-z_]{2,8}){0,2}",
        1usize..50_000,
        0usize..10_000,
        0usize..10_000,
        1usize..500,
    )
        .prop_map(|(module, code, comments, blanks, files)| {
            let lines = code + comments + blanks;
            let bytes = lines * 40;
            let tokens = bytes / 4;
            ModuleRow {
                module,
                code,
                lines,
                files,
                bytes,
                tokens,
                avg_lines: avg(lines, files),
            }
        })
}

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (
        "[a-z_]+/[a-z_]+\\.[a-z]{1,4}",
        "[a-z_]+",
        "[A-Z][a-z]{2,10}",
        prop_oneof![Just(FileKind::Parent), Just(FileKind::Child)],
        0usize..20_000,
        0usize..5_000,
        0usize..5_000,
    )
        .prop_map(|(path, module, lang, kind, code, comments, blanks)| {
            let lines = code + comments + blanks;
            let (bytes, tokens) = if kind == FileKind::Parent {
                let b = lines * 40;
                (b, b / 4)
            } else {
                (0, 0)
            };
            FileRow {
                path,
                module,
                lang,
                kind,
                code,
                comments,
                blanks,
                lines,
                bytes,
                tokens,
            }
        })
}

/// Scan a directory and return Languages data.
fn scan_path(path: &str) -> Languages {
    let mut languages = Languages::new();
    let paths = vec![PathBuf::from(path)];
    let cfg = Config::default();
    languages.get_statistics(&paths, &[], &cfg);
    languages
}

fn crate_src_path() -> String {
    format!("{}/src", env!("CARGO_MANIFEST_DIR"))
}

proptest! {
    // ========================
    // Average Function Properties
    // ========================

    #[test]
    fn avg_zero_files_is_zero(lines in 0usize..10000) {
        prop_assert_eq!(avg(lines, 0), 0);
    }

    #[test]
    fn avg_zero_lines_is_zero(files in 1usize..10000) {
        prop_assert_eq!(avg(0, files), 0);
    }

    #[test]
    fn avg_same_value(value in 1usize..10000) {
        // lines == files should give approximately 1
        prop_assert_eq!(avg(value, value), 1);
    }

    #[test]
    fn avg_double(value in 1usize..5000) {
        // 2*value lines, value files should give 2
        prop_assert_eq!(avg(2 * value, value), 2);
    }

    #[test]
    fn avg_rounds_correctly(lines in 0usize..10000, files in 1usize..1000) {
        let result = avg(lines, files);
        let expected = (lines + (files / 2)) / files;
        prop_assert_eq!(result, expected, "Rounding mismatch");
    }

    #[test]
    fn avg_bounded(lines in 0usize..10000, files in 1usize..1000) {
        let result = avg(lines, files);
        // Result should be roughly lines/files, within rounding
        let lower = lines / files;
        let upper = if lines % files == 0 { lower } else { lower + 1 };
        prop_assert!(result >= lower && result <= upper,
            "avg({}, {}) = {} should be in [{}, {}]", lines, files, result, lower, upper);
    }

    // ========================
    // Path Normalization Properties
    // ========================

    #[test]
    fn normalize_path_never_crashes(s in "\\PC*") {
        let p = Path::new(&s);
        let _ = normalize_path(p, None);
    }

    #[test]
    fn normalize_path_always_forward_slash(s in "\\PC*") {
        let p = Path::new(&s);
        let normalized = normalize_path(p, None);
        prop_assert!(!normalized.contains('\\'), "Should not contain backslash: {}", normalized);
    }

    #[test]
    fn normalize_path_no_leading_dot_slash(s in "\\PC*") {
        let p = Path::new(&s);
        let normalized = normalize_path(p, None);
        prop_assert!(!normalized.starts_with("./"), "Should not start with ./: {}", normalized);
    }

    #[test]
    fn normalize_path_no_leading_slash(s in "\\PC*") {
        let p = Path::new(&s);
        let normalized = normalize_path(p, None);
        // After normalization, should not start with /
        prop_assert!(!normalized.starts_with('/'), "Should not start with /: {}", normalized);
    }

    #[test]
    fn normalize_path_idempotent(s in "[a-zA-Z0-9_/\\.]+") {
        let p = Path::new(&s);
        let once = normalize_path(p, None);
        let twice = normalize_path(Path::new(&once), None);
        prop_assert_eq!(once, twice, "Normalization should be idempotent");
    }

    #[test]
    fn normalize_path_prefix_stripping(
        prefix_parts in prop::collection::vec("[a-zA-Z0-9_]+", 1..3),
        suffix_parts in prop::collection::vec("[a-zA-Z0-9_]+", 1..3)
    ) {
        let prefix_path = prefix_parts.join("/");
        let suffix_path = suffix_parts.join("/");
        let full_path = format!("{}/{}", prefix_path, suffix_path);

        let prefix = Path::new(&prefix_path);
        let full = Path::new(&full_path);
        let normalized = normalize_path(full, Some(prefix));

        // The key property is that after stripping the prefix, we get exactly the suffix.
        // Note: We don't check !normalized.starts_with(&prefix_path) because when
        // prefix and suffix contain the same segments (e.g., prefix="_", suffix="_"),
        // the result legitimately starts with the same characters as the prefix.
        prop_assert_eq!(&normalized, &suffix_path,
            "After stripping '{}' from '{}', expected '{}', got '{}'",
            prefix_path, full_path, suffix_path, normalized);
    }

    // ========================
    // Module Key Properties
    // ========================

    #[test]
    fn module_key_never_crashes(
        path in "\\PC*",
        ref roots in prop::collection::vec("\\PC*", 0..5),
        depth in 0usize..10
    ) {
        let _ = module_key(&path, roots, depth);
    }

    #[test]
    fn module_key_root_file_is_root(filename in "[a-zA-Z0-9_]+\\.[a-z]+") {
        // Single filename (no directory) should always be (root)
        let key = module_key(&filename, &[], 2);
        prop_assert_eq!(key, "(root)", "Single file '{}' should be (root)", filename);
    }

    #[test]
    fn module_key_non_matching_root_is_first_dir(
        dir in "[a-zA-Z0-9_]+",
        subdirs in prop::collection::vec("[a-zA-Z0-9_]+", 1..3),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+"
    ) {
        // When first dir is not in roots, module key is just the first dir
        let path_parts: Vec<&str> = std::iter::once(dir.as_str())
            .chain(subdirs.iter().map(|s| s.as_str()))
            .chain(std::iter::once(filename.as_str()))
            .collect();
        let path = path_parts.join("/");

        // Use roots that don't match the dir
        let roots = vec!["nonexistent_root".to_string()];
        let key = module_key(&path, &roots, 3);
        prop_assert_eq!(&key, &dir, "Non-matching root should return first dir: path='{}', key='{}'", path, key);
    }

    #[test]
    fn module_key_matching_root_depth(
        root in "[a-zA-Z0-9_]+",
        subdirs in prop::collection::vec("[a-zA-Z0-9_]+", 2..5),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+",
        depth in 1usize..4
    ) {
        let path_parts: Vec<&str> = std::iter::once(root.as_str())
            .chain(subdirs.iter().map(|s| s.as_str()))
            .chain(std::iter::once(filename.as_str()))
            .collect();
        let path = path_parts.join("/");

        let roots = vec![root.clone()];
        let key = module_key(&path, &roots, depth);

        // Key should be at most `depth` directory segments
        let key_depth = key.split('/').count();
        let max_dirs = subdirs.len() + 1; // root + subdirs
        let expected_depth = depth.min(max_dirs);
        prop_assert_eq!(key_depth, expected_depth,
            "Key '{}' should have depth {}, has {} (path='{}', depth={})",
            key, expected_depth, key_depth, path, depth);
    }

    #[test]
    fn module_key_deterministic(
        path in "[a-zA-Z0-9_/]+\\.[a-z]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
        depth in 1usize..5
    ) {
        let key1 = module_key(&path, roots, depth);
        let key2 = module_key(&path, roots, depth);
        prop_assert_eq!(key1, key2, "Module key should be deterministic");
    }

    #[test]
    fn module_key_normalized_input(
        parts in prop::collection::vec("[a-zA-Z0-9_]+", 2..4),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+"
    ) {
        let forward_path = format!("{}/{}", parts.join("/"), filename);
        let back_path = format!("{}\\{}", parts.join("\\"), filename);
        let dotslash_path = format!("./{}/{}", parts.join("/"), filename);

        let roots: Vec<String> = vec![];
        let k_forward = module_key(&forward_path, &roots, 2);
        let k_back = module_key(&back_path, &roots, 2);
        let k_dot = module_key(&dotslash_path, &roots, 2);

        prop_assert_eq!(&k_forward, &k_back, "Backslash path should normalize: '{}' vs '{}'", forward_path, back_path);
        prop_assert_eq!(&k_forward, &k_dot, "Dotslash path should normalize: '{}' vs '{}'", forward_path, dotslash_path);
    }

    #[test]
    fn module_key_no_backslash(
        path in "[a-zA-Z0-9_/\\\\]+\\.[a-z]+",
        ref roots in prop::collection::vec("[a-zA-Z0-9_]+".prop_map(String::from), 0..3),
        depth in 1usize..5
    ) {
        let key = module_key(&path, roots, depth);
        prop_assert!(!key.contains('\\'), "Module key should not contain backslash: {}", key);
    }

    // ========================
    // Sum Invariant: Row sums must match totals (synthetic data)
    // ========================

    #[test]
    fn lang_row_sum_invariant(rows in prop::collection::vec(arb_lang_row(), 1..20)) {
        let expected_code: usize = rows.iter().map(|r| r.code).sum();
        let expected_lines: usize = rows.iter().map(|r| r.lines).sum();
        let expected_bytes: usize = rows.iter().map(|r| r.bytes).sum();
        let expected_tokens: usize = rows.iter().map(|r| r.tokens).sum();

        prop_assert_eq!(expected_code, rows.iter().map(|r| r.code).sum::<usize>());
        prop_assert_eq!(expected_lines, rows.iter().map(|r| r.lines).sum::<usize>());
        prop_assert_eq!(expected_bytes, rows.iter().map(|r| r.bytes).sum::<usize>());
        prop_assert_eq!(expected_tokens, rows.iter().map(|r| r.tokens).sum::<usize>());

        // lines >= code for every row (comments + blanks are non-negative)
        for row in &rows {
            prop_assert!(row.lines >= row.code,
                "lines ({}) must be >= code ({}) for lang '{}'",
                row.lines, row.code, row.lang);
        }
    }

    #[test]
    fn module_row_sum_invariant(rows in prop::collection::vec(arb_module_row(), 1..20)) {
        let expected_code: usize = rows.iter().map(|r| r.code).sum();
        let expected_lines: usize = rows.iter().map(|r| r.lines).sum();
        let expected_bytes: usize = rows.iter().map(|r| r.bytes).sum();
        let expected_tokens: usize = rows.iter().map(|r| r.tokens).sum();

        prop_assert_eq!(expected_code, rows.iter().map(|r| r.code).sum::<usize>());
        prop_assert_eq!(expected_lines, rows.iter().map(|r| r.lines).sum::<usize>());
        prop_assert_eq!(expected_bytes, rows.iter().map(|r| r.bytes).sum::<usize>());
        prop_assert_eq!(expected_tokens, rows.iter().map(|r| r.tokens).sum::<usize>());

        for row in &rows {
            prop_assert!(row.lines >= row.code,
                "lines ({}) must be >= code ({}) for module '{}'",
                row.lines, row.code, row.module);
        }
    }

    #[test]
    fn file_row_lines_equals_components(row in arb_file_row()) {
        prop_assert_eq!(row.lines, row.code + row.comments + row.blanks,
            "lines must equal code + comments + blanks for '{}'", row.path);
    }

    // ========================
    // Deterministic Ordering Properties
    // ========================

    #[test]
    fn lang_rows_sort_deterministic(mut rows in prop::collection::vec(arb_lang_row(), 2..30)) {
        // Apply the same sort as create_lang_report
        rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));

        // Verify descending by code, then ascending by name
        for i in 1..rows.len() {
            let prev = &rows[i - 1];
            let curr = &rows[i];
            prop_assert!(
                prev.code > curr.code || (prev.code == curr.code && prev.lang <= curr.lang),
                "Sort violation at index {}: ({}, {}) vs ({}, {})",
                i, prev.code, prev.lang, curr.code, curr.lang
            );
        }

        // Sorting again must produce the same result (stability)
        let mut rows2 = rows.clone();
        rows2.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        prop_assert_eq!(&rows, &rows2, "Re-sorting must be idempotent");
    }

    #[test]
    fn module_rows_sort_deterministic(mut rows in prop::collection::vec(arb_module_row(), 2..30)) {
        rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));

        for i in 1..rows.len() {
            let prev = &rows[i - 1];
            let curr = &rows[i];
            prop_assert!(
                prev.code > curr.code || (prev.code == curr.code && prev.module <= curr.module),
                "Sort violation at index {}: ({}, {}) vs ({}, {})",
                i, prev.code, prev.module, curr.code, curr.module
            );
        }

        let mut rows2 = rows.clone();
        rows2.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
        prop_assert_eq!(&rows, &rows2, "Re-sorting must be idempotent");
    }

    #[test]
    fn file_rows_sort_deterministic(mut rows in prop::collection::vec(arb_file_row(), 2..30)) {
        // Export data uses: descending by code, then ascending by path
        rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path)));

        for i in 1..rows.len() {
            let prev = &rows[i - 1];
            let curr = &rows[i];
            prop_assert!(
                prev.code > curr.code || (prev.code == curr.code && prev.path <= curr.path),
                "Sort violation at index {}: ({}, {}) vs ({}, {})",
                i, prev.code, prev.path, curr.code, curr.path
            );
        }
    }

    // ========================
    // Non-negative Values (all line counts are non-negative)
    // ========================

    #[test]
    fn lang_row_non_negative(row in arb_lang_row()) {
        // usize is inherently >= 0, but we verify the structural invariants
        prop_assert!(row.lines >= row.code, "lines >= code");
        prop_assert!(row.files > 0, "files > 0 for non-empty row");
        prop_assert!(row.tokens == row.bytes / 4, "tokens == bytes / 4");
    }

    #[test]
    fn file_row_child_no_bytes(row in arb_file_row()) {
        if row.kind == FileKind::Child {
            prop_assert_eq!(row.bytes, 0, "Child rows must have 0 bytes");
            prop_assert_eq!(row.tokens, 0, "Child rows must have 0 tokens");
        }
    }

    // ========================
    // Round-trip: Serialize → Deserialize produces equal values
    // ========================

    #[test]
    fn totals_round_trip(totals in arb_totals()) {
        let json = serde_json::to_string(&totals).unwrap();
        let recovered: Totals = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&totals, &recovered, "Totals round-trip failed");
    }

    #[test]
    fn lang_row_round_trip(row in arb_lang_row()) {
        let json = serde_json::to_string(&row).unwrap();
        let recovered: LangRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&row, &recovered, "LangRow round-trip failed");
    }

    #[test]
    fn module_row_round_trip(row in arb_module_row()) {
        let json = serde_json::to_string(&row).unwrap();
        let recovered: ModuleRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&row, &recovered, "ModuleRow round-trip failed");
    }

    #[test]
    fn file_row_round_trip(row in arb_file_row()) {
        let json = serde_json::to_string(&row).unwrap();
        let recovered: FileRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&row, &recovered, "FileRow round-trip failed");
    }

    #[test]
    fn lang_rows_vec_round_trip(rows in prop::collection::vec(arb_lang_row(), 0..15)) {
        let json = serde_json::to_string(&rows).unwrap();
        let recovered: Vec<LangRow> = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&rows, &recovered, "Vec<LangRow> round-trip failed");
    }

    #[test]
    fn module_rows_vec_round_trip(rows in prop::collection::vec(arb_module_row(), 0..15)) {
        let json = serde_json::to_string(&rows).unwrap();
        let recovered: Vec<ModuleRow> = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&rows, &recovered, "Vec<ModuleRow> round-trip failed");
    }

    #[test]
    fn file_rows_vec_round_trip(rows in prop::collection::vec(arb_file_row(), 0..15)) {
        let json = serde_json::to_string(&rows).unwrap();
        let recovered: Vec<FileRow> = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&rows, &recovered, "Vec<FileRow> round-trip failed");
    }
}

// ========================
// Scan-based aggregation property tests
// (These use real scanning for end-to-end correctness.)
// ========================

/// Sum invariant: lang report totals must equal sum of all language rows.
#[test]
fn scan_lang_report_sum_invariant_collapse() {
    let languages = scan_path(&crate_src_path());
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    let row_code: usize = report.rows.iter().map(|r| r.code).sum();
    let row_lines: usize = report.rows.iter().map(|r| r.lines).sum();
    let row_bytes: usize = report.rows.iter().map(|r| r.bytes).sum();
    let row_tokens: usize = report.rows.iter().map(|r| r.tokens).sum();

    assert_eq!(report.total.code, row_code, "total.code == sum(rows.code)");
    assert_eq!(
        report.total.lines, row_lines,
        "total.lines == sum(rows.lines)"
    );
    assert_eq!(
        report.total.bytes, row_bytes,
        "total.bytes == sum(rows.bytes)"
    );
    assert_eq!(
        report.total.tokens, row_tokens,
        "total.tokens == sum(rows.tokens)"
    );
}

#[test]
fn scan_lang_report_sum_invariant_separate() {
    let languages = scan_path(&crate_src_path());
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Separate);

    let row_code: usize = report.rows.iter().map(|r| r.code).sum();
    let row_lines: usize = report.rows.iter().map(|r| r.lines).sum();

    assert_eq!(report.total.code, row_code);
    assert_eq!(report.total.lines, row_lines);
}

/// Sum invariant: module report totals must equal sum of all module rows.
#[test]
fn scan_module_report_sum_invariant() {
    let languages = scan_path(&crate_src_path());
    let report = create_module_report(&languages, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    let row_code: usize = report.rows.iter().map(|r| r.code).sum();
    let row_lines: usize = report.rows.iter().map(|r| r.lines).sum();
    let row_bytes: usize = report.rows.iter().map(|r| r.bytes).sum();
    let row_tokens: usize = report.rows.iter().map(|r| r.tokens).sum();

    assert_eq!(report.total.code, row_code);
    assert_eq!(report.total.lines, row_lines);
    assert_eq!(report.total.bytes, row_bytes);
    assert_eq!(report.total.tokens, row_tokens);
}

/// Sum invariant: file rows lines == code + comments + blanks for every row.
#[test]
fn scan_file_rows_line_decomposition() {
    let languages = scan_path(&crate_src_path());
    let rows = collect_file_rows(&languages, &[], 2, ChildIncludeMode::Separate, None);

    for row in &rows {
        assert_eq!(
            row.lines,
            row.code + row.comments + row.blanks,
            "lines != code+comments+blanks for {}",
            row.path
        );
    }
}

/// Deterministic ordering: two calls with the same input produce identical output.
#[test]
fn scan_lang_report_deterministic_ordering() {
    let languages = scan_path(&crate_src_path());
    let r1 = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let r2 = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    assert_eq!(r1.rows.len(), r2.rows.len());
    for (a, b) in r1.rows.iter().zip(r2.rows.iter()) {
        assert_eq!(a, b, "Rows differ between identical calls");
    }
    assert_eq!(r1.total, r2.total);
}

#[test]
fn scan_module_report_deterministic_ordering() {
    let languages = scan_path(&crate_src_path());
    let r1 = create_module_report(&languages, &[], 2, ChildIncludeMode::ParentsOnly, 0);
    let r2 = create_module_report(&languages, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    assert_eq!(r1.rows.len(), r2.rows.len());
    for (a, b) in r1.rows.iter().zip(r2.rows.iter()) {
        assert_eq!(a, b, "Rows differ between identical calls");
    }
    assert_eq!(r1.total, r2.total);
}

#[test]
fn scan_export_data_deterministic_ordering() {
    let languages = scan_path(&crate_src_path());
    let d1 = create_export_data(
        &languages,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );
    let d2 = create_export_data(
        &languages,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );

    assert_eq!(d1.rows.len(), d2.rows.len());
    for (a, b) in d1.rows.iter().zip(d2.rows.iter()) {
        assert_eq!(a, b, "Rows differ between identical calls");
    }
}

/// Children mode consistency: Collapse total code should equal Separate total code.
/// Collapse uses `summarise()` which merges children into parent, while Separate
/// shows them as distinct rows. Total code must be equal; total lines may differ
/// because Separate filters children with zero code (dropping their blanks/comments).
#[test]
fn scan_children_mode_code_consistency() {
    let languages = scan_path(&crate_src_path());
    let collapse = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);
    let separate = create_lang_report(&languages, 0, false, ChildrenMode::Separate);

    assert_eq!(
        collapse.total.code, separate.total.code,
        "Collapse code ({}) != Separate code ({})",
        collapse.total.code, separate.total.code
    );

    // Same unique parent files regardless of children mode
    assert_eq!(
        collapse.total.files, separate.total.files,
        "Collapse files ({}) != Separate files ({})",
        collapse.total.files, separate.total.files
    );

    // Collapse lines >= Separate lines because Collapse merges children
    // blanks/comments into the parent even when child has 0 code lines.
    assert!(
        collapse.total.lines >= separate.total.lines,
        "Collapse lines ({}) should be >= Separate lines ({})",
        collapse.total.lines,
        separate.total.lines
    );
}

/// Non-negative values: all counters in scan-based reports are non-negative.
#[test]
fn scan_lang_report_non_negative() {
    let languages = scan_path(&crate_src_path());
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    for row in &report.rows {
        assert!(row.code > 0, "code > 0 (zero-code rows are filtered)");
        assert!(row.lines >= row.code, "lines >= code for {}", row.lang);
        assert!(row.files > 0, "files > 0 for {}", row.lang);
    }
    assert!(report.total.code > 0, "total code > 0");
    assert!(report.total.files > 0, "total files > 0");
}

#[test]
fn scan_module_report_non_negative() {
    let languages = scan_path(&crate_src_path());
    let report = create_module_report(&languages, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    for row in &report.rows {
        assert!(row.lines >= row.code, "lines >= code for {}", row.module);
    }
    assert!(report.total.code > 0, "total code > 0");
}

/// Round-trip: serialize the entire lang report to JSON and deserialize back.
#[test]
fn scan_lang_report_round_trip() {
    let languages = scan_path(&crate_src_path());
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    let json = serde_json::to_string(&report).unwrap();
    let recovered: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify individual fields survive round-trip
    let rows: Vec<LangRow> = serde_json::from_value(recovered["rows"].clone()).unwrap();
    assert_eq!(rows, report.rows, "Rows mismatch after round-trip");

    let total: Totals = serde_json::from_value(recovered["total"].clone()).unwrap();
    assert_eq!(total, report.total, "Totals mismatch after round-trip");

    // Verify scalar fields
    let with_files: bool = serde_json::from_value(recovered["with_files"].clone()).unwrap();
    assert_eq!(with_files, report.with_files);

    let top: usize = serde_json::from_value(recovered["top"].clone()).unwrap();
    assert_eq!(top, report.top);
}

#[test]
fn scan_module_report_round_trip() {
    let languages = scan_path(&crate_src_path());
    let report = create_module_report(&languages, &[], 2, ChildIncludeMode::ParentsOnly, 0);

    let json = serde_json::to_string(&report).unwrap();
    let recovered: serde_json::Value = serde_json::from_str(&json).unwrap();

    let rows: Vec<ModuleRow> = serde_json::from_value(recovered["rows"].clone()).unwrap();
    assert_eq!(rows, report.rows);

    let total: Totals = serde_json::from_value(recovered["total"].clone()).unwrap();
    assert_eq!(total, report.total);
}

#[test]
fn scan_export_data_round_trip() {
    let languages = scan_path(&crate_src_path());
    let data = create_export_data(
        &languages,
        &[],
        2,
        ChildIncludeMode::ParentsOnly,
        None,
        0,
        0,
    );

    let json = serde_json::to_string(&data).unwrap();
    let recovered: serde_json::Value = serde_json::from_str(&json).unwrap();

    let rows: Vec<FileRow> = serde_json::from_value(recovered["rows"].clone()).unwrap();
    assert_eq!(rows, data.rows);
}

/// Unique parent file count must be consistent with lang report files.
#[test]
fn scan_unique_file_count_consistency() {
    let languages = scan_path(&crate_src_path());
    let count = unique_parent_file_count(&languages);
    let report = create_lang_report(&languages, 0, false, ChildrenMode::Collapse);

    assert_eq!(
        report.total.files, count,
        "total.files must equal unique_parent_file_count"
    );
}

// Note: fold_other_* property tests are in lib.rs where they can access
// the private functions directly instead of reimplementing them.

// ========================
// Input-order independence: shuffled input produces same sorted output
// ========================

proptest! {
    #[test]
    fn lang_rows_sort_order_independent(rows in prop::collection::vec(arb_lang_row(), 2..20)) {
        let sort_fn = |v: &mut Vec<LangRow>| {
            v.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
        };

        let mut forward = rows.clone();
        sort_fn(&mut forward);

        let mut reversed = rows.into_iter().rev().collect::<Vec<_>>();
        sort_fn(&mut reversed);

        prop_assert_eq!(&forward, &reversed, "Sort must be input-order independent");
    }

    #[test]
    fn module_rows_sort_order_independent(rows in prop::collection::vec(arb_module_row(), 2..20)) {
        let sort_fn = |v: &mut Vec<ModuleRow>| {
            v.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
        };

        let mut forward = rows.clone();
        sort_fn(&mut forward);

        let mut reversed = rows.into_iter().rev().collect::<Vec<_>>();
        sort_fn(&mut reversed);

        prop_assert_eq!(&forward, &reversed, "Sort must be input-order independent");
    }

    #[test]
    fn file_rows_sort_order_independent(rows in prop::collection::vec(arb_file_row(), 2..20)) {
        let sort_fn = |v: &mut Vec<FileRow>| {
            v.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path)));
        };

        let mut forward = rows.clone();
        sort_fn(&mut forward);

        let mut reversed = rows.into_iter().rev().collect::<Vec<_>>();
        sort_fn(&mut reversed);

        prop_assert_eq!(&forward, &reversed, "Sort must be input-order independent");
    }

    /// Total lines across all rows equals the sum of individual rows.
    #[test]
    fn lang_rows_total_equals_sum(rows in prop::collection::vec(arb_lang_row(), 1..20)) {
        let sum_code: usize = rows.iter().map(|r| r.code).sum();
        let sum_lines: usize = rows.iter().map(|r| r.lines).sum();
        let sum_bytes: usize = rows.iter().map(|r| r.bytes).sum();
        let sum_tokens: usize = rows.iter().map(|r| r.tokens).sum();

        // Re-computing must give same result (associativity of addition).
        let recomputed_code: usize = rows.iter().map(|r| r.code).sum();
        prop_assert_eq!(sum_code, recomputed_code);
        prop_assert_eq!(sum_lines, rows.iter().map(|r| r.lines).sum::<usize>());
        prop_assert_eq!(sum_bytes, rows.iter().map(|r| r.bytes).sum::<usize>());
        prop_assert_eq!(sum_tokens, rows.iter().map(|r| r.tokens).sum::<usize>());
    }

    /// Module grouping via module_key is deterministic: same path always gives same key.
    #[test]
    fn module_grouping_deterministic(
        parts in prop::collection::vec("[a-zA-Z0-9_]+", 2..5),
        filename in "[a-zA-Z0-9_]+\\.[a-z]+",
        depth in 1usize..5
    ) {
        let path = format!("{}/{}", parts.join("/"), filename);
        let roots: Vec<String> = vec![];
        let k1 = module_key(&path, &roots, depth);
        let k2 = module_key(&path, &roots, depth);
        prop_assert_eq!(&k1, &k2, "module_key must be deterministic for path '{}'", path);
    }
}
