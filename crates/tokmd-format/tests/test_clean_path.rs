use tokmd_format::redact::redact_path;

#[test]
fn test_redact_path_normalizes_parent_segments() {
    let p1 = "crates/tokmd/../foo/lib.rs";
    let p2 = "crates/foo/lib.rs";
    assert_eq!(redact_path(p1), redact_path(p2));
}

#[test]
fn test_redact_path_normalizes_parent_segments_at_root() {
    let p1 = "../foo/lib.rs";
    let p2 = "../foo/lib.rs";
    assert_eq!(redact_path(p1), redact_path(p2));
}

#[test]
fn test_redact_path_preserves_absolute_paths() {
    let p1 = "/crates/tokmd/../foo/lib.rs";
    let p2 = "/crates/foo/lib.rs";
    assert_eq!(redact_path(p1), redact_path(p2));
}

#[test]
fn test_redact_path_complex_normalization() {
    let p1 = "./crates/foo/./bar/../../baz/lib.rs";
    let p2 = "crates/baz/lib.rs";
    assert_eq!(redact_path(p1), redact_path(p2));
}

#[test]
fn test_redact_path_complex_absolute() {
    let p1 = "/crates/foo/./bar/../../baz/lib.rs";
    let p2 = "/crates/baz/lib.rs";
    assert_eq!(redact_path(p1), redact_path(p2));
}

#[test]
fn test_redact_path_double_slash() {
    let p1 = "crates//foo/lib.rs";
    let p2 = "crates/foo/lib.rs";
    assert_eq!(redact_path(p1), redact_path(p2));
}

#[test]
fn test_redact_path_dot_double_slash() {
    let p1 = "./crates//foo/lib.rs";
    let p2 = "crates/foo/lib.rs";
    assert_eq!(redact_path(p1), redact_path(p2));
}
