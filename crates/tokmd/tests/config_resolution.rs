use std::collections::HashMap;
use tokmd::resolve_lang;
use tokmd_config::{CliLangArgs, Profile, UserConfig};
use tokmd_types::LangArgs;

#[test]
fn test_resolve_lang_no_args_no_profile() {
    let cli = CliLangArgs::default();
    let profile = None;

    let resolved = resolve_lang(&cli, profile);

    // Default fallback values
    assert_eq!(resolved.paths[0].to_string_lossy(), ".");
    assert_eq!(resolved.format, tokmd_config::TableFormat::Md);
    assert_eq!(resolved.top, 0);
    assert_eq!(resolved.files, false);
}

#[test]
fn test_resolve_lang_cli_overrides_profile() {
    let mut cli = CliLangArgs::default();
    cli.top = Some(50);
    cli.format = Some(tokmd_config::TableFormat::Json);

    let mut profile = Profile::default();
    profile.top = Some(10);
    profile.format = Some("csv".to_string());

    let resolved = resolve_lang(&cli, Some(&profile));

    assert_eq!(resolved.top, 50);
    assert_eq!(resolved.format, tokmd_config::TableFormat::Json);
}

#[test]
fn test_resolve_lang_profile_overrides_default() {
    let cli = CliLangArgs::default();

    let mut profile = Profile::default();
    profile.top = Some(10);
    profile.format = Some("tsv".to_string());
    profile.files = Some(true);

    let resolved = resolve_lang(&cli, Some(&profile));

    assert_eq!(resolved.top, 10);
    assert_eq!(resolved.format, tokmd_config::TableFormat::Tsv);
    assert_eq!(resolved.files, true);
}

#[test]
fn test_resolve_lang_partial_overrides() {
    let mut cli = CliLangArgs::default();
    cli.files = true; // Override files only

    let mut profile = Profile::default();
    profile.top = Some(10); // Profile sets top
    profile.format = Some("tsv".to_string()); // Profile sets format

    let resolved = resolve_lang(&cli, Some(&profile));

    assert_eq!(resolved.top, 10); // From profile
    assert_eq!(resolved.format, tokmd_config::TableFormat::Tsv); // From profile
    assert_eq!(resolved.files, true); // From CLI
}
