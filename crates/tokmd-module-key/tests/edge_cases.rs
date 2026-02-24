use tokmd_module_key::module_key;

#[test]
fn module_key_ignores_dot_segments_in_middle() {
    let roots = vec!["crates".to_string()];
    // If we have a path like crates/./foo/src/lib.rs
    // Ideally it should be treated as crates/foo/src/lib.rs
    let path = "crates/./foo/src/lib.rs";
    let key = module_key(path, &roots, 2);

    // Current implementation likely produces "crates/."
    assert_eq!(key, "crates/foo", "Expected 'crates/foo' but got '{}'", key);
}

#[test]
fn module_key_ignores_multiple_dot_segments() {
    let roots = vec!["crates".to_string()];
    let path = "crates/././foo/src/lib.rs";
    let key = module_key(path, &roots, 2);
    assert_eq!(key, "crates/foo", "Expected 'crates/foo' but got '{}'", key);
}

#[test]
fn module_key_ignores_empty_segments_in_middle() {
    let roots = vec!["crates".to_string()];
    let path = "crates//foo/src/lib.rs";
    let key = module_key(path, &roots, 2);
    assert_eq!(key, "crates/foo", "Expected 'crates/foo' but got '{}'", key);
}
