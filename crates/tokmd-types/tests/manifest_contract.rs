#[test]
fn manifest_stays_clap_free() {
    let manifest = include_str!("../Cargo.toml");

    for line in manifest.lines().map(str::trim) {
        assert_ne!(
            line, r#"clap = ["dep:clap"]"#,
            "tokmd-types must not expose a clap feature"
        );
        assert!(
            !line.starts_with("clap ") && !line.starts_with("clap."),
            "tokmd-types must not declare a clap dependency"
        );
        assert!(
            !line.starts_with("clap="),
            "tokmd-types must not declare a clap dependency"
        );
    }
}
