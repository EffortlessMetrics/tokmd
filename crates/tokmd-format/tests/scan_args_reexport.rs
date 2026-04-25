use std::path::PathBuf;

use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

#[test]
fn given_scan_args_returns_stable_results() {
    // Given: canonical scan settings with redaction enabled
    let paths = vec![
        PathBuf::from("./src/lib.rs"),
        PathBuf::from(r".\src\main.rs"),
    ];
    let options = ScanOptions {
        excluded: vec!["target".into(), "node_modules".into()],
        no_ignore: true,
        ..Default::default()
    };

    // When: normalization is called twice
    let from_format = tokmd_format::scan_args(&paths, &options, Some(RedactMode::Paths));
    let from_format_repeat = tokmd_format::scan_args(&paths, &options, Some(RedactMode::Paths));

    // Then: output is deterministic across invocations
    assert_eq!(from_format.paths, from_format_repeat.paths);
    assert_eq!(from_format.excluded, from_format_repeat.excluded);
    assert_eq!(
        from_format.excluded_redacted,
        from_format_repeat.excluded_redacted
    );
    assert_eq!(
        from_format.no_ignore_parent,
        from_format_repeat.no_ignore_parent
    );
    assert_eq!(from_format.no_ignore_dot, from_format_repeat.no_ignore_dot);
    assert_eq!(from_format.no_ignore_vcs, from_format_repeat.no_ignore_vcs);
}
