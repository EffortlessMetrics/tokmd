use super::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn scan_options_default() {
    let opts = ScanOptions::default();
    assert!(opts.excluded.is_empty());
    assert!(!opts.hidden);
    assert!(!opts.no_ignore);
}

#[test]
fn scan_settings_current_dir() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, vec!["."]);
}

#[test]
fn scan_settings_for_paths() {
    let s = ScanSettings::for_paths(vec!["src".into(), "lib".into()]);
    assert_eq!(s.paths.len(), 2);
}

#[test]
fn scan_settings_flatten() {
    // Verify that ScanOptions fields are accessible through ScanSettings.
    let s = ScanSettings {
        paths: vec![".".into()],
        options: ScanOptions {
            hidden: true,
            ..Default::default()
        },
    };
    assert!(s.options.hidden);
}

#[test]
fn serde_roundtrip_scan_options() {
    let opts = ScanOptions {
        excluded: vec!["target".into()],
        config: ConfigMode::None,
        hidden: true,
        no_ignore: false,
        no_ignore_parent: true,
        no_ignore_dot: false,
        no_ignore_vcs: true,
        treat_doc_strings_as_comments: true,
    };
    let json = serde_json::to_string(&opts).unwrap();
    let back: ScanOptions = serde_json::from_str(&json).unwrap();
    assert_eq!(back.excluded, opts.excluded);
    assert!(back.hidden);
    assert!(back.no_ignore_parent);
    assert!(back.no_ignore_vcs);
    assert!(back.treat_doc_strings_as_comments);
}

#[test]
fn serde_roundtrip_scan_settings() {
    let s = ScanSettings {
        paths: vec!["src".into()],
        options: ScanOptions {
            excluded: vec!["*.bak".into()],
            ..Default::default()
        },
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: ScanSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.paths, s.paths);
    assert_eq!(back.options.excluded, s.options.excluded);
}

#[test]
fn serde_roundtrip_lang_settings() {
    let s = LangSettings {
        top: 10,
        files: true,
        children: ChildrenMode::Separate,
        redact: Some(RedactMode::Paths),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: LangSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.top, 10);
    assert!(back.files);
}

#[test]
fn serde_roundtrip_export_settings() {
    let s = ExportSettings::default();
    let json = serde_json::to_string(&s).unwrap();
    let back: ExportSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.min_code, 0);
    assert!(back.meta);
}

#[test]
fn serde_roundtrip_analyze_settings() {
    let s = AnalyzeSettings::default();
    let json = serde_json::to_string(&s).unwrap();
    let back: AnalyzeSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.preset, "receipt");
    assert_eq!(back.granularity, "module");
}

#[test]
fn serde_roundtrip_cockpit_settings() {
    let s = CockpitSettings::default();
    let json = serde_json::to_string(&s).unwrap();
    let back: CockpitSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "main");
    assert_eq!(back.head, "HEAD");
    assert_eq!(back.range_mode, "two-dot");
    assert!(back.baseline.is_none());
}

#[test]
fn serde_roundtrip_cockpit_settings_with_baseline() {
    let s = CockpitSettings {
        base: "v1.0".into(),
        head: "feature".into(),
        range_mode: "three-dot".into(),
        baseline: Some("baseline.json".into()),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: CockpitSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.base, "v1.0");
    assert_eq!(back.baseline, Some("baseline.json".to_string()));
}

#[test]
fn serde_roundtrip_diff_settings() {
    let s = DiffSettings {
        from: "v1.0".into(),
        to: "v2.0".into(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: DiffSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from, "v1.0");
    assert_eq!(back.to, "v2.0");
}

#[test]
fn toml_parse_and_view_profiles() {
    let toml_str = r#"
[scan]
hidden = true

[view.llm]
format = "json"
top = 10
"#;
    let config = TomlConfig::parse(toml_str).expect("parse config");
    assert_eq!(config.scan.hidden, Some(true));
    let llm = config.view.get("llm").expect("llm profile");
    assert_eq!(llm.format.as_deref(), Some("json"));
    assert_eq!(llm.top, Some(10));
}

#[test]
fn toml_from_file_roundtrip() {
    let toml_content = r#"
[module]
depth = 3
roots = ["src", "tests"]
"#;

    let mut temp_file = NamedTempFile::new().expect("temp file");
    temp_file
        .write_all(toml_content.as_bytes())
        .expect("write config");

    let config = TomlConfig::from_file(temp_file.path()).expect("load config");
    assert_eq!(config.module.depth, Some(3));
    assert_eq!(
        config.module.roots,
        Some(vec!["src".to_string(), "tests".to_string()])
    );
}
