use tokmd::resolve_lang;
use tokmd_config::{CliLangArgs, Profile};

#[test]
fn test_resolve_lang_no_args_no_profile() {
    let cli = CliLangArgs::default();
    let profile = None;

    let resolved = resolve_lang(&cli, profile);

    // Default fallback values
    assert_eq!(resolved.paths[0].to_string_lossy(), ".");
    assert_eq!(resolved.format, tokmd_config::TableFormat::Md);
    assert_eq!(resolved.top, 0);
    assert!(!resolved.files);
}

#[test]
fn test_resolve_lang_cli_overrides_profile() {
    let cli = CliLangArgs {
        top: Some(50),
        format: Some(tokmd_config::TableFormat::Json),
        ..Default::default()
    };

    let profile = Profile {
        top: Some(10),
        format: Some("csv".to_string()),
        ..Default::default()
    };

    let resolved = resolve_lang(&cli, Some(&profile));

    assert_eq!(resolved.top, 50);
    assert_eq!(resolved.format, tokmd_config::TableFormat::Json);
}

#[test]
fn test_resolve_lang_profile_overrides_default() {
    let cli = CliLangArgs::default();

    let profile = Profile {
        top: Some(10),
        format: Some("tsv".to_string()),
        files: Some(true),
        ..Default::default()
    };

    let resolved = resolve_lang(&cli, Some(&profile));

    assert_eq!(resolved.top, 10);
    assert_eq!(resolved.format, tokmd_config::TableFormat::Tsv);
    assert!(resolved.files);
}

#[test]
fn test_resolve_lang_partial_overrides() {
    let cli = CliLangArgs {
        files: true, // Override files only
        ..Default::default()
    };

    let profile = Profile {
        top: Some(10),                   // Profile sets top
        format: Some("tsv".to_string()), // Profile sets format
        ..Default::default()
    };

    let resolved = resolve_lang(&cli, Some(&profile));

    assert_eq!(resolved.top, 10); // From profile
    assert_eq!(resolved.format, tokmd_config::TableFormat::Tsv); // From profile
    assert!(resolved.files); // From CLI
}
