use tokmd_module_key::module_key;

#[test]
fn given_workspace_root_path_when_module_key_computed_then_depth_limits_segments() {
    // Given: a path under a configured monorepo root
    let roots = vec!["crates".to_string(), "packages".to_string()];
    let path = "crates/tokmd-model/src/lib.rs";

    // When: module key is computed with depth 2
    let key = module_key(path, &roots, 2);

    // Then: key includes root + one child directory
    assert_eq!(key, "crates/tokmd-model");
}

#[test]
fn given_non_root_path_when_module_key_computed_then_first_directory_is_used() {
    // Given: a path whose first segment is not a configured root
    let roots = vec!["crates".to_string()];
    let path = "src/commands/analyze.rs";

    // When: module key is computed
    let key = module_key(path, &roots, 5);

    // Then: key is the top-level directory
    assert_eq!(key, "src");
}

#[test]
fn given_windows_style_path_when_module_key_computed_then_result_is_forward_slash_deterministic() {
    // Given: equivalent path forms
    let roots = vec!["crates".to_string()];
    let win = r".\crates\tokmd\src\main.rs";
    let unix = "./crates/tokmd/src/main.rs";

    // When: keys are computed
    let win_key = module_key(win, &roots, 2);
    let unix_key = module_key(unix, &roots, 2);

    // Then: normalized output is stable across separators
    assert_eq!(win_key, unix_key);
    assert_eq!(win_key, "crates/tokmd");
}

#[test]
fn given_path_with_dot_segments_when_module_key_computed_then_dots_are_filtered() {
    // Given: a path with dot segments
    let roots = vec!["crates".to_string()];
    let path = "crates/./tokmd-model/src/lib.rs";

    // When: module key is computed
    let key = module_key(path, &roots, 2);

    // Then: key ignores the dot segments
    assert_eq!(key, "crates/tokmd-model");
}
