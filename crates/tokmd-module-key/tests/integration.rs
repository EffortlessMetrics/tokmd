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
