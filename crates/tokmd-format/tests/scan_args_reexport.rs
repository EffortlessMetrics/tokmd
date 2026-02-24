use std::path::PathBuf;

use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

#[test]
fn given_scan_args_reexport_when_called_then_output_matches_core_microcrate() {
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

    // When: both entrypoints are called
    let from_format = tokmd_format::scan_args(&paths, &options, Some(RedactMode::Paths));
    let from_microcrate = tokmd_scan_args::scan_args(&paths, &options, Some(RedactMode::Paths));

    // Then: the compatibility layer is byte-equivalent
    assert_eq!(from_format.paths, from_microcrate.paths);
    assert_eq!(from_format.excluded, from_microcrate.excluded);
    assert_eq!(
        from_format.excluded_redacted,
        from_microcrate.excluded_redacted
    );
    assert_eq!(
        from_format.no_ignore_parent,
        from_microcrate.no_ignore_parent
    );
    assert_eq!(from_format.no_ignore_dot, from_microcrate.no_ignore_dot);
    assert_eq!(from_format.no_ignore_vcs, from_microcrate.no_ignore_vcs);
}
