//! Facade Structural Property Verification
//!
//! Verifies the analysis_facade module properties:
//! 1. Type equivalence: facade re-exports the same types
//! 2. Function equivalence: facade render produces identical output
//! 3. Feature gate correctness: facade available when analysis feature enabled

#[cfg(all(test, feature = "analysis"))]
mod facade_tests {
    use tokmd_analysis_format::{RenderedOutput as OriginalOutput, render as original_render};
    use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisReceipt, AnalysisSource};
    use tokmd_core::analysis_facade::{RenderedOutput as FacadeOutput, render as facade_render};
    use tokmd_types::{AnalysisFormat, ScanStatus, ToolInfo};

    fn minimal_receipt() -> AnalysisReceipt {
        AnalysisReceipt {
            schema_version: 2,
            generated_at_ms: 0,
            tool: ToolInfo {
                name: "test".into(),
                version: "0.0.0".into(),
            },
            mode: "analysis".into(),
            status: ScanStatus::Complete,
            warnings: vec![],
            source: AnalysisSource {
                inputs: vec!["test".into()],
                export_path: None,
                base_receipt_path: None,
                export_schema_version: None,
                export_generated_at_ms: None,
                base_signature: None,
                module_roots: vec![],
                module_depth: 1,
                children: "separate".into(),
            },
            args: AnalysisArgsMeta {
                preset: "receipt".into(),
                format: "json".into(),
                window_tokens: None,
                git: None,
                max_files: None,
                max_bytes: None,
                max_commits: None,
                max_commit_files: None,
                max_file_bytes: None,
                import_granularity: "module".into(),
            },
            archetype: None,
            topics: None,
            entropy: None,
            predictive_churn: None,
            corporate_fingerprint: None,
            license: None,
            derived: None,
            assets: None,
            deps: None,
            git: None,
            imports: None,
            dup: None,
            complexity: None,
            api_surface: None,
            fun: None,
            effort: None,
        }
    }

    /// Verify that the facade RenderedOutput is the same type as the original
    #[test]
    fn type_equivalence_rendered_output() {
        // If this compiles, the types are equivalent (re-exported, not wrapped)
        fn assert_type_eq<T>(_: T) {}

        let original: OriginalOutput = OriginalOutput::Text("test".to_string());
        let facade: FacadeOutput = original; // This works because they're the same type
        assert_type_eq(facade);
    }

    /// Verify binary variant type equivalence
    #[test]
    fn type_equivalence_binary_variant() {
        let original: OriginalOutput = OriginalOutput::Binary(vec![1, 2, 3]);
        let facade: FacadeOutput = original;

        match facade {
            FacadeOutput::Binary(bytes) => assert_eq!(bytes, vec![1, 2, 3]),
            _ => panic!("Expected Binary variant"),
        }
    }

    /// Verify that facade render produces identical JSON output to original
    #[test]
    fn function_equivalence_json_format() {
        let receipt = minimal_receipt();

        let facade_result =
            facade_render(&receipt, AnalysisFormat::Json).expect("facade render failed");
        let original_result =
            original_render(&receipt, AnalysisFormat::Json).expect("original render failed");

        match (facade_result, original_result) {
            (FacadeOutput::Text(f), OriginalOutput::Text(o)) => {
                assert_eq!(f, o, "JSON output mismatch between facade and original");
            }
            _ => panic!("Expected Text variant for JSON format"),
        }
    }

    /// Verify that facade render produces identical XML output to original
    #[test]
    fn function_equivalence_xml_format() {
        let receipt = minimal_receipt();

        let facade_result =
            facade_render(&receipt, AnalysisFormat::Xml).expect("facade render failed");
        let original_result =
            original_render(&receipt, AnalysisFormat::Xml).expect("original render failed");

        match (facade_result, original_result) {
            (FacadeOutput::Text(f), OriginalOutput::Text(o)) => {
                assert_eq!(f, o, "XML output mismatch between facade and original");
            }
            _ => panic!("Expected Text variant for XML format"),
        }
    }

    /// Verify that facade render produces identical SVG output to original
    #[test]
    fn function_equivalence_svg_format() {
        let receipt = minimal_receipt();

        let facade_result =
            facade_render(&receipt, AnalysisFormat::Svg).expect("facade render failed");
        let original_result =
            original_render(&receipt, AnalysisFormat::Svg).expect("original render failed");

        match (facade_result, original_result) {
            (FacadeOutput::Text(f), OriginalOutput::Text(o)) => {
                assert_eq!(f, o, "SVG output mismatch between facade and original");
            }
            _ => panic!("Expected Text variant for SVG format"),
        }
    }

    /// Verify that facade render produces identical Tree output to original
    #[test]
    fn function_equivalence_tree_format() {
        let receipt = minimal_receipt();

        let facade_result =
            facade_render(&receipt, AnalysisFormat::Tree).expect("facade render failed");
        let original_result =
            original_render(&receipt, AnalysisFormat::Tree).expect("original render failed");

        match (facade_result, original_result) {
            (FacadeOutput::Text(f), OriginalOutput::Text(o)) => {
                assert_eq!(f, o, "Tree output mismatch between facade and original");
            }
            _ => panic!("Expected Text variant for Tree format"),
        }
    }

    /// Verify that facade render produces identical Markdown output to original
    #[test]
    fn function_equivalence_md_format() {
        let receipt = minimal_receipt();

        let facade_result =
            facade_render(&receipt, AnalysisFormat::Md).expect("facade render failed");
        let original_result =
            original_render(&receipt, AnalysisFormat::Md).expect("original render failed");

        match (facade_result, original_result) {
            (FacadeOutput::Text(f), OriginalOutput::Text(o)) => {
                assert_eq!(f, o, "Markdown output mismatch between facade and original");
            }
            _ => panic!("Expected Text variant for Markdown format"),
        }
    }

    /// Verify that facade module is accessible (feature gate enabled)
    #[test]
    fn feature_gate_facade_accessible() {
        // This test only compiles/runs when 'analysis' feature is enabled
        // Verifying that the re-exports are available
        use tokmd_core::analysis_facade::{RenderedOutput, render};

        // Just verify we can reference the types
        let _: fn(&AnalysisReceipt, AnalysisFormat) -> Result<RenderedOutput, _> = render;
    }
}
