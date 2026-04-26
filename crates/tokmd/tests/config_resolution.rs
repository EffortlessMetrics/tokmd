use tokmd::cli::{CliLangArgs, TableFormat};
use tokmd::resolve_lang;
use tokmd_settings::Profile;

#[test]
fn test_resolve_lang_no_args_no_profile() {
    let cli = CliLangArgs::default();
    let profile = None;

    let resolved = resolve_lang(&cli, profile);

    // Default fallback values
    assert_eq!(resolved.paths[0].to_string_lossy(), ".");
    assert_eq!(resolved.format, TableFormat::Md);
    assert_eq!(resolved.top, 0);
    assert!(!resolved.files);
}

#[test]
fn test_resolve_lang_cli_overrides_profile() {
    let cli = CliLangArgs {
        top: Some(50),
        format: Some(TableFormat::Json),
        ..Default::default()
    };

    let profile = Profile {
        top: Some(10),
        format: Some("csv".to_string()),
        ..Default::default()
    };

    let resolved = resolve_lang(&cli, Some(&profile));

    assert_eq!(resolved.top, 50);
    assert_eq!(resolved.format, TableFormat::Json);
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
    assert_eq!(resolved.format, TableFormat::Tsv);
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
    assert_eq!(resolved.format, TableFormat::Tsv); // From profile
    assert!(resolved.files); // From CLI
}

#[test]
fn test_resolve_export_cli_overrides_profile() {
    use tokmd::cli::{CliExportArgs, ExportFormat};
    use tokmd::resolve_export;

    let cli = CliExportArgs {
        format: Some(ExportFormat::Csv),
        min_code: Some(50),
        paths: None,
        output: None,
        module_roots: None,
        module_depth: None,
        children: None,
        max_rows: None,
        redact: None,
        meta: None,
        strip_prefix: None,
    };

    let profile = Profile {
        format: Some("json".to_string()),
        min_code: Some(10),
        ..Default::default()
    };

    let resolved = resolve_export(&cli, Some(&profile));

    assert_eq!(resolved.format, ExportFormat::Csv);
    assert_eq!(resolved.min_code, 50);
}

#[test]
fn test_resolve_module_profile_overrides_default() {
    use tokmd::cli::CliModuleArgs;
    use tokmd::resolve_module;

    let cli = CliModuleArgs {
        paths: None,
        format: None,
        top: None,
        module_roots: None,
        module_depth: None,
        children: None,
    };

    let profile = Profile {
        module_depth: Some(5),
        module_roots: Some(vec!["src".to_string()]),
        ..Default::default()
    };

    let resolved = resolve_module(&cli, Some(&profile));

    assert_eq!(resolved.module_depth, 5);
    assert_eq!(resolved.module_roots, vec!["src".to_string()]);
}

#[test]
fn test_resolve_export_with_config() {
    use tokmd::cli::{CliExportArgs, ExportFormat};
    use tokmd::{ResolvedConfig, resolve_export_with_config};
    use tokmd_settings::{ExportConfig, TomlConfig};

    let cli = CliExportArgs {
        format: Some(ExportFormat::Csv),
        min_code: None,
        paths: None,
        output: None,
        module_roots: None,
        module_depth: None,
        children: None,
        max_rows: None,
        redact: None,
        meta: None,
        strip_prefix: None,
    };

    let toml = TomlConfig {
        export: ExportConfig {
            min_code: Some(25),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut config = ResolvedConfig::default();
    // In Rust we typically construct via fields if the lifetimes are tight.
    // Or if `toml` is `&'a TomlConfig`, let's just make it a local reference.
    // The previous test failed because `ResolvedConfig::toml` expects an `Option<&'a TomlConfig>`.
    let toml_ref = &toml;
    config.toml = Some(toml_ref);

    let resolved = resolve_export_with_config(&cli, &config);

    assert_eq!(resolved.format, ExportFormat::Csv);
    assert_eq!(resolved.min_code, 25);
}

#[test]
fn test_resolve_module_with_config() {
    use tokmd::cli::CliModuleArgs;
    use tokmd::{ResolvedConfig, resolve_module_with_config};
    use tokmd_settings::{ModuleConfig, TomlConfig};

    let cli = CliModuleArgs {
        paths: None,
        format: None,
        top: None,
        module_roots: None,
        module_depth: None,
        children: None,
    };

    let toml = TomlConfig {
        module: ModuleConfig {
            depth: Some(8),
            roots: Some(vec!["libs".to_string()]),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut config = ResolvedConfig::default();
    let toml_ref = &toml;
    config.toml = Some(toml_ref);

    let resolved = resolve_module_with_config(&cli, &config);

    assert_eq!(resolved.module_depth, 8);
    assert_eq!(resolved.module_roots, vec!["libs".to_string()]);
}

#[test]
fn test_resolve_export_no_args_no_profile() {
    use tokmd::cli::{CliExportArgs, ExportFormat};
    use tokmd::resolve_export;

    let cli = CliExportArgs {
        paths: None,
        format: None,
        output: None,
        module_roots: None,
        module_depth: None,
        children: None,
        min_code: None,
        max_rows: None,
        redact: None,
        meta: None,
        strip_prefix: None,
    };
    let resolved = resolve_export(&cli, None);

    assert_eq!(resolved.paths[0].to_string_lossy(), ".");
    assert_eq!(resolved.format, ExportFormat::Jsonl);
    assert_eq!(
        resolved.module_roots,
        vec!["crates".to_string(), "packages".to_string()]
    );
    assert_eq!(resolved.module_depth, 2);
    assert_eq!(resolved.min_code, 0);
    assert_eq!(resolved.max_rows, 0);
    assert_eq!(resolved.meta, true);
}

#[test]
fn test_resolve_module_no_args_no_profile() {
    use tokmd::cli::CliModuleArgs;
    use tokmd::resolve_module;

    let cli = CliModuleArgs {
        paths: None,
        format: None,
        top: None,
        module_roots: None,
        module_depth: None,
        children: None,
    };
    let resolved = resolve_module(&cli, None);

    assert_eq!(resolved.paths[0].to_string_lossy(), ".");
    assert_eq!(resolved.format, tokmd::cli::TableFormat::Md);
    assert_eq!(resolved.top, 0);
    assert_eq!(
        resolved.module_roots,
        vec!["crates".to_string(), "packages".to_string()]
    );
    assert_eq!(resolved.module_depth, 2);
}
