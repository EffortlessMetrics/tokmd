        let opts = settings_to_scan_options(&scan);
        assert_eq!(opts.excluded, vec!["target"]);
        assert!(opts.hidden);
        assert!(opts.no_ignore);
    }

    #[test]
    fn scan_settings_current_dir() {
        let settings = ScanSettings::current_dir();
        assert_eq!(settings.paths, vec!["."]);
    }

    #[test]
    fn scan_settings_for_paths() {
        let settings = ScanSettings::for_paths(vec!["src".to_string(), "lib".to_string()]);
        assert_eq!(settings.paths, vec!["src", "lib"]);
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn effort_request_defaults_to_estimate_preset() {
        let analyze = AnalyzeSettings {
            preset: "estimate".to_string(),
            ..Default::default()
        };
        let req = parse_effort_request(&analyze, "estimate").expect("parse effort request");
        let req = req.expect("estimate should imply effort request");
        assert_eq!(
            req.model.as_str(),
            analysis::EffortModelKind::Cocomo81Basic.as_str()
        );
        assert_eq!(req.layer.as_str(), analysis::EffortLayer::Full.as_str());
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn effort_request_not_implied_for_non_estimate_without_flags() {
        let analyze = AnalyzeSettings {
            preset: "receipt".to_string(),
            ..Default::default()
        };
        let req = parse_effort_request(&analyze, "receipt").expect("parse effort request");
        assert!(req.is_none());
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn effort_request_rejects_unsupported_model() {
        let analyze = AnalyzeSettings {
            preset: "estimate".to_string(),
            effort_model: Some("cocomo2-early".to_string()),
            ..Default::default()
        };
        let err =
            parse_effort_request(&analyze, "estimate").expect_err("unsupported model should fail");
        assert!(err.to_string().contains("only 'cocomo81-basic'"));
    }

    fn mk_temp_dir(prefix: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut root = std::env::temp_dir();
        root.push(format!("{prefix}-{timestamp}-{}", std::process::id()));
        root
    }

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn analyze_workflow_estimate_preset_populates_effort_and_size_basis_breakdown() {
        let root = mk_temp_dir("tokmd-core-estimate-preset");
        let _guard = TempDirGuard(root.clone());
        write_file(&root.join("src/main.rs"), "fn main() {}\n");
        write_file(
            &root.join("target/generated/bundle.min.js"),
            "console.log(1);\n",
        );
        write_file(
            &root.join("vendor/lib/external.rs"),
            "pub fn external() {}\n",
        );

        let scan = settings::ScanSettings::for_paths(vec![root.display().to_string()]);
        let analyze = AnalyzeSettings {
            preset: "estimate".to_string(),
            ..Default::default()
        };

        let receipt = analyze_workflow(&scan, &analyze).expect("estimate analyze failed");
        let effort = receipt
            .effort
            .as_ref()
            .expect("estimate preset should produce effort");

        assert!(effort.results.effort_pm_p50 > 0.0);
        assert_eq!(
            effort.size_basis.total_lines,
            effort.size_basis.authored_lines
                + effort.size_basis.generated_lines
                + effort.size_basis.vendored_lines
        );
        assert!(effort.size_basis.authored_lines > 0);
        assert!(
            effort.size_basis.generated_lines + effort.size_basis.vendored_lines > 0,
            "expected deterministic generated or vendored lines"
        );
    }
}

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub mod readme_doctests {}
