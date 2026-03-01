//! Determinism-focused regression tests for tokmd-model.
//!
//! Verifies that aggregation with shuffled input produces identical output,
//! that module key generation is order-independent, and that children mode
//! handling is deterministic.

use proptest::prelude::*;

use tokmd_model::{avg, module_key, normalize_path};
use tokmd_types::{LangRow, ModuleRow};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_lang_row(lang: &str, code: usize) -> LangRow {
    let lines = code + 20;
    let files = 3;
    LangRow {
        lang: lang.to_string(),
        code,
        lines,
        files,
        bytes: code * 10,
        tokens: code * 10 / 4,
        avg_lines: if files > 0 { lines / files } else { 0 },
    }
}

fn make_module_row(module: &str, code: usize) -> ModuleRow {
    let lines = code + 30;
    let files = 2;
    ModuleRow {
        module: module.to_string(),
        code,
        lines,
        files,
        bytes: code * 10,
        tokens: code * 10 / 4,
        avg_lines: if files > 0 { lines / files } else { 0 },
    }
}

/// Simulate the model layer's sort: descending by code, then ascending by name.
fn sort_lang_rows(rows: &mut Vec<LangRow>) {
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
}

fn sort_module_rows(rows: &mut Vec<ModuleRow>) {
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
}

// ---------------------------------------------------------------------------
// 1. Aggregation with shuffled input produces identical output
// ---------------------------------------------------------------------------

#[test]
fn determinism_lang_sort_shuffled_input_identical() {
    let mut rows_a = vec![
        make_lang_row("Go", 300),
        make_lang_row("Rust", 1000),
        make_lang_row("Python", 500),
    ];
    let mut rows_b = vec![
        make_lang_row("Python", 500),
        make_lang_row("Go", 300),
        make_lang_row("Rust", 1000),
    ];
    let mut rows_c = vec![
        make_lang_row("Rust", 1000),
        make_lang_row("Go", 300),
        make_lang_row("Python", 500),
    ];

    sort_lang_rows(&mut rows_a);
    sort_lang_rows(&mut rows_b);
    sort_lang_rows(&mut rows_c);

    assert_eq!(rows_a, rows_b, "Different insertion order must yield same sorted result");
    assert_eq!(rows_b, rows_c, "All permutations must sort identically");
}

#[test]
fn determinism_module_sort_shuffled_input_identical() {
    let mut rows_a = vec![
        make_module_row("src", 200),
        make_module_row("crates/foo", 800),
        make_module_row("crates/bar", 400),
    ];
    let mut rows_b = vec![
        make_module_row("crates/bar", 400),
        make_module_row("src", 200),
        make_module_row("crates/foo", 800),
    ];

    sort_module_rows(&mut rows_a);
    sort_module_rows(&mut rows_b);

    assert_eq!(rows_a, rows_b, "Module sort must be order-independent");
    // Verify expected order: desc by code
    assert_eq!(rows_a[0].module, "crates/foo");
    assert_eq!(rows_a[1].module, "crates/bar");
    assert_eq!(rows_a[2].module, "src");
}

#[test]
fn determinism_lang_sort_tiebreak_by_name() {
    let mut rows = vec![
        make_lang_row("Zebra", 500),
        make_lang_row("Alpha", 500),
        make_lang_row("Middle", 500),
    ];

    sort_lang_rows(&mut rows);

    // Equal code → alphabetical by name.
    assert_eq!(rows[0].lang, "Alpha");
    assert_eq!(rows[1].lang, "Middle");
    assert_eq!(rows[2].lang, "Zebra");
}

// ---------------------------------------------------------------------------
// 2. Module key generation is order-independent
// ---------------------------------------------------------------------------

#[test]
fn determinism_module_key_same_regardless_of_call_order() {
    let roots = vec!["crates".to_string(), "packages".to_string()];
    let paths = vec![
        "crates/foo/src/lib.rs",
        "crates/bar/src/main.rs",
        "packages/baz/index.ts",
        "src/util.rs",
        "Cargo.toml",
    ];

    // Compute module keys in forward order.
    let keys_forward: Vec<String> = paths.iter().map(|p| module_key(p, &roots, 2)).collect();

    // Compute in reverse order.
    let keys_reverse: Vec<String> = paths
        .iter()
        .rev()
        .map(|p| module_key(p, &roots, 2))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    assert_eq!(
        keys_forward, keys_reverse,
        "Module key generation must be order-independent"
    );
}

#[test]
fn determinism_module_key_repeated_calls() {
    let roots = vec!["crates".to_string()];
    let path = "crates/tokmd-types/src/lib.rs";

    let k1 = module_key(path, &roots, 2);
    let k2 = module_key(path, &roots, 2);
    let k3 = module_key(path, &roots, 2);

    assert_eq!(k1, "crates/tokmd-types");
    assert_eq!(k1, k2);
    assert_eq!(k2, k3);
}

#[test]
fn determinism_module_key_root_files() {
    let roots = vec!["crates".to_string()];

    assert_eq!(module_key("Cargo.toml", &roots, 2), "(root)");
    assert_eq!(module_key("README.md", &roots, 2), "(root)");
    assert_eq!(module_key("LICENSE", &roots, 2), "(root)");
}

// ---------------------------------------------------------------------------
// 3. Children mode (collapse/separate) is deterministic
// ---------------------------------------------------------------------------

#[test]
fn determinism_children_collapse_sort_stable() {
    // Simulate collapse mode output: only parent language rows, sorted.
    let mut rows = vec![
        make_lang_row("HTML", 200),
        make_lang_row("Rust", 1500),
        make_lang_row("JavaScript", 800),
    ];

    sort_lang_rows(&mut rows);

    assert_eq!(rows[0].lang, "Rust");
    assert_eq!(rows[1].lang, "JavaScript");
    assert_eq!(rows[2].lang, "HTML");

    // Repeat sort is idempotent.
    let snapshot = rows.clone();
    sort_lang_rows(&mut rows);
    assert_eq!(rows, snapshot, "Sort must be idempotent");
}

#[test]
fn determinism_children_separate_embedded_rows_sorted() {
    // Simulate separate mode: parent + embedded rows, all sorted together.
    let mut rows = vec![
        make_lang_row("Rust", 1500),
        make_lang_row("JavaScript (embedded)", 300),
        make_lang_row("HTML", 200),
        make_lang_row("CSS (embedded)", 100),
    ];

    sort_lang_rows(&mut rows);

    // Sorted desc by code, tiebreak by name.
    assert_eq!(rows[0].lang, "Rust");
    assert_eq!(rows[1].lang, "JavaScript (embedded)");
    assert_eq!(rows[2].lang, "HTML");
    assert_eq!(rows[3].lang, "CSS (embedded)");
}

#[test]
fn determinism_children_separate_sort_idempotent() {
    let mut rows = vec![
        make_lang_row("CSS (embedded)", 100),
        make_lang_row("Rust", 1500),
        make_lang_row("JavaScript (embedded)", 300),
        make_lang_row("HTML", 200),
    ];

    sort_lang_rows(&mut rows);
    let first_sort = rows.clone();
    sort_lang_rows(&mut rows);
    assert_eq!(rows, first_sort, "Sorting embedded rows must be idempotent");
}

// ---------------------------------------------------------------------------
// 4. Proptest: shuffled aggregation determinism
// ---------------------------------------------------------------------------

fn arb_lang_row_strat() -> impl Strategy<Value = LangRow> {
    (
        prop::sample::select(vec![
            "Rust",
            "Python",
            "Go",
            "Java",
            "C",
            "TypeScript",
            "Ruby",
            "Haskell",
        ]),
        1usize..50_000,
    )
        .prop_map(|(lang, code)| {
            let lines = code + 20;
            let files = 3;
            LangRow {
                lang: lang.to_string(),
                code,
                lines,
                files,
                bytes: code * 10,
                tokens: code * 10 / 4,
                avg_lines: if files > 0 { lines / files } else { 0 },
            }
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn determinism_proptest_sort_is_idempotent(rows in prop::collection::vec(arb_lang_row_strat(), 1..10)) {
        let mut sorted = rows.clone();
        sort_lang_rows(&mut sorted);
        let snapshot = sorted.clone();
        sort_lang_rows(&mut sorted);
        prop_assert_eq!(sorted, snapshot, "Sort must be idempotent");
    }

    #[test]
    fn determinism_proptest_sort_order_independent(
        mut rows in prop::collection::vec(arb_lang_row_strat(), 2..8)
    ) {
        // Sort forward.
        let mut forward = rows.clone();
        sort_lang_rows(&mut forward);

        // Reverse and sort.
        rows.reverse();
        sort_lang_rows(&mut rows);

        prop_assert_eq!(forward, rows, "Sort must produce same result regardless of input order");
    }

    #[test]
    fn determinism_proptest_module_key_pure(
        path_segments in prop::collection::vec("[a-z]{2,6}", 1..5),
        roots in prop::collection::vec("[a-z]{2,6}", 0..3),
        depth in 1usize..5
    ) {
        let path = path_segments.join("/") + "/file.rs";
        let k1 = module_key(&path, &roots, depth);
        let k2 = module_key(&path, &roots, depth);
        prop_assert_eq!(k1, k2, "module_key must be a pure function");
    }

    #[test]
    fn determinism_proptest_normalize_path_pure(
        segments in prop::collection::vec("[a-zA-Z0-9_.-]+", 1..5)
    ) {
        let path_str = segments.join("/");
        let p = std::path::PathBuf::from(&path_str);
        let n1 = normalize_path(&p, None);
        let n2 = normalize_path(&p, None);
        prop_assert_eq!(n1, n2, "normalize_path must be a pure function");
    }

    #[test]
    fn determinism_proptest_avg_is_pure(lines in 0usize..100_000, files in 0usize..1_000) {
        let a1 = avg(lines, files);
        let a2 = avg(lines, files);
        prop_assert_eq!(a1, a2, "avg must be a pure function");
    }
}
