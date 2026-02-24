use std::path::PathBuf;

use tokmd_scan_args::scan_args;
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

#[test]
fn given_paths_mode_when_building_scan_args_then_paths_and_exclusions_are_redacted() {
    // Given: user-provided scan paths and exclusion patterns
    let paths = vec![PathBuf::from("secret/src/lib.rs")];
    let global = ScanOptions {
        excluded: vec!["**/private/**".to_string()],
        ..Default::default()
    };

    // When: scan args are constructed with redact paths mode
    let args = scan_args(&paths, &global, Some(RedactMode::Paths));

    // Then: raw paths and exclusion patterns are not present in metadata
    assert_ne!(args.paths[0], "secret/src/lib.rs");
    assert_ne!(args.excluded[0], "**/private/**");
    assert!(args.excluded_redacted);
}

#[test]
fn given_no_redaction_when_building_scan_args_then_normalized_paths_are_preserved() {
    // Given: relative scan inputs with slash variants
    let paths = vec![PathBuf::from(r".\src\main.rs")];
    let global = ScanOptions {
        excluded: vec!["target".to_string()],
        ..Default::default()
    };

    // When: scan args are constructed without redaction
    let args = scan_args(&paths, &global, None);

    // Then: paths are normalized but not hashed, and exclusions remain visible
    assert_eq!(args.paths, vec!["src/main.rs".to_string()]);
    assert_eq!(args.excluded, vec!["target".to_string()]);
    assert!(!args.excluded_redacted);
}

#[test]
fn given_no_ignore_when_building_scan_args_then_ignore_sub_flags_are_forced_true() {
    // Given: no_ignore is enabled but sub-flags are false in settings
    let paths = vec![PathBuf::from(".")];
    let global = ScanOptions {
        no_ignore: true,
        no_ignore_parent: false,
        no_ignore_dot: false,
        no_ignore_vcs: false,
        ..Default::default()
    };

    // When: scan args are constructed
    let args = scan_args(&paths, &global, Some(RedactMode::None));

    // Then: all ignore-family flags are true for deterministic behavior
    assert!(args.no_ignore);
    assert!(args.no_ignore_parent);
    assert!(args.no_ignore_dot);
    assert!(args.no_ignore_vcs);
}
