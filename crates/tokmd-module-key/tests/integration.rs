use std::collections::BTreeMap;

use tokmd_module_key::module_key;

#[test]
fn module_key_groups_paths_deterministically_for_workspace_like_layout() {
    let roots = vec!["crates".to_string(), "packages".to_string()];
    let depth = 2;
    let paths = [
        "crates/tokmd-model/src/lib.rs",
        "crates/tokmd-model/tests/model_tests.rs",
        "crates/tokmd-format/src/lib.rs",
        "packages/web/src/main.ts",
        "src/main.rs",
    ];

    let mut grouped: BTreeMap<String, usize> = BTreeMap::new();
    for path in paths {
        *grouped.entry(module_key(path, &roots, depth)).or_default() += 1;
    }

    assert_eq!(grouped.get("crates/tokmd-model"), Some(&2));
    assert_eq!(grouped.get("crates/tokmd-format"), Some(&1));
    assert_eq!(grouped.get("packages/web"), Some(&1));
    assert_eq!(grouped.get("src"), Some(&1));
}

#[test]
fn module_key_is_stable_across_input_order() {
    let roots = vec!["crates".to_string()];
    let depth = 2;
    let ordered = ["crates/a/src/lib.rs", "crates/b/src/lib.rs", "tools/gen.rs"];
    let reversed = ["tools/gen.rs", "crates/b/src/lib.rs", "crates/a/src/lib.rs"];

    let keys1: Vec<String> = ordered
        .iter()
        .map(|p| module_key(p, &roots, depth))
        .collect();
    let keys2: Vec<String> = reversed
        .iter()
        .map(|p| module_key(p, &roots, depth))
        .collect();

    assert_eq!(keys1, vec!["crates/a", "crates/b", "tools"]);
    assert_eq!(keys2, vec!["tools", "crates/b", "crates/a"]);
}

#[test]
fn module_key_groups_monorepo_with_three_roots() {
    let roots = vec![
        "crates".to_string(),
        "packages".to_string(),
        "libs".to_string(),
    ];
    let depth = 2;
    let paths = [
        "crates/a/src/lib.rs",
        "crates/b/src/lib.rs",
        "packages/ui/index.ts",
        "libs/core/mod.rs",
        "scripts/build.sh",
        "Cargo.toml",
    ];

    let mut grouped: BTreeMap<String, usize> = BTreeMap::new();
    for path in paths {
        *grouped.entry(module_key(path, &roots, depth)).or_default() += 1;
    }

    assert_eq!(grouped.len(), 6);
    assert_eq!(grouped.get("crates/a"), Some(&1));
    assert_eq!(grouped.get("crates/b"), Some(&1));
    assert_eq!(grouped.get("packages/ui"), Some(&1));
    assert_eq!(grouped.get("libs/core"), Some(&1));
    assert_eq!(grouped.get("scripts"), Some(&1));
    assert_eq!(grouped.get("(root)"), Some(&1)); // Cargo.toml is root-level
}

#[test]
fn module_key_groups_root_level_files_together() {
    let roots = vec!["crates".to_string()];
    let paths = ["Cargo.toml", "README.md", ".gitignore", "Makefile"];

    let mut grouped: BTreeMap<String, usize> = BTreeMap::new();
    for path in paths {
        *grouped.entry(module_key(path, &roots, 2)).or_default() += 1;
    }

    assert_eq!(grouped.len(), 1);
    assert_eq!(grouped.get("(root)"), Some(&4));
}

#[test]
fn module_key_groups_deeply_nested_files_by_depth() {
    let roots = vec!["src".to_string()];
    let paths = [
        "src/a/b/c/file1.rs",
        "src/a/b/d/file2.rs",
        "src/a/x/y/file3.rs",
        "src/z/w/v/file4.rs",
    ];

    // depth 2: src/a, src/z
    let grouped_d2: BTreeMap<String, usize> = paths.iter().fold(BTreeMap::new(), |mut m, p| {
        *m.entry(module_key(p, &roots, 2)).or_default() += 1;
        m
    });
    assert_eq!(grouped_d2.get("src/a"), Some(&3));
    assert_eq!(grouped_d2.get("src/z"), Some(&1));

    // depth 3: src/a/b, src/a/x, src/z/w
    let grouped_d3: BTreeMap<String, usize> = paths.iter().fold(BTreeMap::new(), |mut m, p| {
        *m.entry(module_key(p, &roots, 3)).or_default() += 1;
        m
    });
    assert_eq!(grouped_d3.get("src/a/b"), Some(&2));
    assert_eq!(grouped_d3.get("src/a/x"), Some(&1));
    assert_eq!(grouped_d3.get("src/z/w"), Some(&1));
}

#[test]
fn module_key_windows_and_unix_paths_group_identically() {
    let roots = vec!["crates".to_string()];
    let unix_paths = ["crates/foo/src/lib.rs", "crates/bar/src/lib.rs"];
    let win_paths = [r"crates\foo\src\lib.rs", r"crates\bar\src\lib.rs"];

    let unix_keys: Vec<String> = unix_paths
        .iter()
        .map(|p| module_key(p, &roots, 2))
        .collect();
    let win_keys: Vec<String> = win_paths.iter().map(|p| module_key(p, &roots, 2)).collect();
    assert_eq!(unix_keys, win_keys);
}

#[test]
fn module_key_special_char_dirs_produce_distinct_groups() {
    let roots: Vec<String> = vec![];
    let paths = ["my-lib/src/lib.rs", "my_lib/src/lib.rs", "mylib/src/lib.rs"];

    let grouped: BTreeMap<String, usize> = paths.iter().fold(BTreeMap::new(), |mut m, p| {
        *m.entry(module_key(p, &roots, 2)).or_default() += 1;
        m
    });

    assert_eq!(grouped.len(), 3);
    assert_eq!(grouped.get("my-lib"), Some(&1));
    assert_eq!(grouped.get("my_lib"), Some(&1));
    assert_eq!(grouped.get("mylib"), Some(&1));
}
