use tokmd::resolve_lang;
use tokmd_config::{CliLangArgs, Profile};

#[test]
fn test_resolve_lang_no_args_no_profile() {
    let cli = CliLangArgs::default();
    let profile = None;

    let resolved = resolve_lang(&cli, profile);

    // Default fallback values
    assert_eq!(resolved.paths[0].to_string_lossy(), ".");
    assert_eq!(resolved.format, tokmd_types::TableFormat::Md);
    assert_eq!(resolved.top, 0);
    assert!(!resolved.files);
}

#[test]
fn test_resolve_lang_cli_overrides_profile() {
    let cli = CliLangArgs {
        top: Some(50),
        format: Some(tokmd_config::CliTableFormat::Json),
        ..Default::default()
    };

    let profile = Profile {
        top: Some(10),
        format: Some("csv".to_string()),
        ..Default::default()
    };

    let resolved = resolve_lang(&cli, Some(&profile));

    assert_eq!(resolved.top, 50);
    assert_eq!(resolved.format, tokmd_types::TableFormat::Json);
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
    assert_eq!(resolved.format, tokmd_types::TableFormat::Tsv);
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
    assert_eq!(resolved.format, tokmd_types::TableFormat::Tsv); // From profile
    assert!(resolved.files); // From CLI
}

use tokmd::config::{resolve_export, resolve_module};
use tokmd_config::{CliExportArgs, CliModuleArgs};

fn empty_module_args() -> CliModuleArgs {
    CliModuleArgs {
        paths: None,
        format: None,
        top: None,
        module_roots: None,
        module_depth: None,
        children: None,
    }
}

#[test]
fn test_resolve_module_no_args_no_profile() {
    let cli = empty_module_args();
    let resolved = resolve_module(&cli, None);

    assert_eq!(resolved.paths[0].to_string_lossy(), ".");
    assert_eq!(resolved.format, tokmd_types::TableFormat::Md);
    assert_eq!(resolved.top, 0);
}

#[test]
fn test_resolve_module_cli_overrides_profile() {
    let mut cli = empty_module_args();
    cli.top = Some(100);
    cli.format = Some(tokmd_config::CliTableFormat::Tsv);

    let profile = Profile {
        top: Some(20),
        format: Some("json".to_string()),
        ..Default::default()
    };
    let resolved = resolve_module(&cli, Some(&profile));

    assert_eq!(resolved.top, 100);
    assert_eq!(resolved.format, tokmd_types::TableFormat::Tsv);
}

fn empty_export_args() -> CliExportArgs {
    CliExportArgs {
        paths: None,
        format: None,
        output: None,
        module_roots: None,
        module_depth: None,
        children: None,
        min_code: None,
        max_rows: None,
        meta: None,
        redact: None,
        strip_prefix: None,
    }
}

#[test]
fn test_resolve_export_no_args_no_profile() {
    let cli = empty_export_args();
    let resolved = resolve_export(&cli, None);

    assert_eq!(resolved.paths[0].to_string_lossy(), ".");
    assert_eq!(resolved.format, tokmd_types::ExportFormat::Jsonl);
}

#[test]
fn test_resolve_export_cli_overrides_profile() {
    let mut cli = empty_export_args();
    cli.format = Some(tokmd_config::CliExportFormat::Csv);

    let profile = Profile {
        format: Some("json".to_string()),
        ..Default::default()
    };
    let resolved = resolve_export(&cli, Some(&profile));

    assert_eq!(resolved.format, tokmd_types::ExportFormat::Csv);
}

#[test]
fn test_resolve_export_profile_overrides_default() {
    let cli = empty_export_args();

    let profile = Profile {
        format: Some("csv".to_string()),
        ..Default::default()
    };

    let resolved = resolve_export(&cli, Some(&profile));

    assert_eq!(resolved.format, tokmd_types::ExportFormat::Csv);
}

#[test]
fn test_resolve_module_profile_overrides_default() {
    let cli = empty_module_args();

    let profile = Profile {
        top: Some(15),
        format: Some("json".to_string()),
        ..Default::default()
    };

    let resolved = resolve_module(&cli, Some(&profile));

    assert_eq!(resolved.top, 15);
    assert_eq!(resolved.format, tokmd_types::TableFormat::Json);
}
