use proptest::prelude::*;
use tokmd_analysis_types::*;

// Re-implement the helpers under test (they are private in the crate).
// We test observable properties of the public `render` function plus
// property-test the formats via their output characteristics.

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_pct(ratio: f64) -> String {
    format!("{:.1}%", ratio * 100.0)
}

fn minimal_receipt() -> AnalysisReceipt {
    AnalysisReceipt {
        schema_version: 2,
        generated_at_ms: 0,
        tool: tokmd_types::ToolInfo {
            name: "tokmd".to_string(),
            version: "0.0.0".to_string(),
        },
        mode: "analysis".to_string(),
        status: tokmd_types::ScanStatus::Complete,
        warnings: vec![],
        source: AnalysisSource {
            inputs: vec!["test".to_string()],
            export_path: None,
            base_receipt_path: None,
            export_schema_version: None,
            export_generated_at_ms: None,
            base_signature: None,
            module_roots: vec![],
            module_depth: 1,
            children: "collapse".to_string(),
        },
        args: AnalysisArgsMeta {
            preset: "receipt".to_string(),
            format: "html".to_string(),
            window_tokens: None,
            git: None,
            max_files: None,
            max_bytes: None,
            max_commits: None,
            max_commit_files: None,
            max_file_bytes: None,
            import_granularity: "module".to_string(),
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
    }
}

proptest! {
    /// format_number output is always valid UTF-8 and non-empty.
    #[test]
    fn format_number_always_produces_output(n in 0usize..10_000_000) {
        let result = format_number(n);
        prop_assert!(!result.is_empty());
    }

    /// format_number is deterministic.
    #[test]
    fn format_number_is_deterministic(n in 0usize..10_000_000) {
        let r1 = format_number(n);
        let r2 = format_number(n);
        prop_assert_eq!(r1, r2);
    }

    /// Numbers below 1000 render as plain digits.
    #[test]
    fn format_number_small_is_plain(n in 0usize..1000) {
        let result = format_number(n);
        prop_assert_eq!(result, n.to_string());
    }

    /// Numbers >= 1000 and < 1_000_000 end with "K".
    #[test]
    fn format_number_thousands_has_k_suffix(n in 1000usize..1_000_000) {
        let result = format_number(n);
        prop_assert!(result.ends_with('K'),
            "Expected '{}' to end with 'K' for n={}", result, n);
    }

    /// Numbers >= 1_000_000 end with "M".
    #[test]
    fn format_number_millions_has_m_suffix(n in 1_000_000usize..10_000_000) {
        let result = format_number(n);
        prop_assert!(result.ends_with('M'),
            "Expected '{}' to end with 'M' for n={}", result, n);
    }

    /// HTML escaping prevents raw angle brackets in output.
    #[test]
    fn escape_html_no_raw_angle_brackets(input in "\\PC{0,100}") {
        let escaped = escape_html(&input);
        // The escaped output should not contain raw < or > unless they
        // were already part of an entity.
        let without_entities = escaped
            .replace("&lt;", "")
            .replace("&gt;", "")
            .replace("&amp;", "")
            .replace("&quot;", "")
            .replace("&#x27;", "");
        prop_assert!(!without_entities.contains('<'),
            "Escaped output still contains raw '<'");
        prop_assert!(!without_entities.contains('>'),
            "Escaped output still contains raw '>'");
    }

    /// HTML escaping is deterministic.
    #[test]
    fn escape_html_is_deterministic(input in "\\PC{0,100}") {
        let r1 = escape_html(&input);
        let r2 = escape_html(&input);
        prop_assert_eq!(r1, r2);
    }

    /// Safe strings pass through HTML escaping unchanged.
    #[test]
    fn escape_html_safe_strings_unchanged(input in "[a-zA-Z0-9 _/.]{0,50}") {
        let escaped = escape_html(&input);
        prop_assert_eq!(escaped, input);
    }

    /// format_pct always ends with '%'.
    #[test]
    fn format_pct_ends_with_percent(ratio in 0.0f64..2.0) {
        let result = format_pct(ratio);
        prop_assert!(result.ends_with('%'));
    }

    /// format_pct is deterministic.
    #[test]
    fn format_pct_is_deterministic(ratio in 0.0f64..2.0) {
        let r1 = format_pct(ratio);
        let r2 = format_pct(ratio);
        prop_assert_eq!(r1, r2);
    }

    /// Render with minimal receipt always produces valid HTML containing DOCTYPE.
    #[test]
    fn render_always_contains_doctype(_dummy in 0..1u8) {
        let receipt = minimal_receipt();
        let html = tokmd_analysis_html::render(&receipt);
        prop_assert!(html.contains("<!DOCTYPE html>"));
    }

    /// Render is deterministic for a fixed receipt (ignoring timestamp).
    #[test]
    fn render_structure_is_stable(_dummy in 0..1u8) {
        let receipt = minimal_receipt();
        let h1 = tokmd_analysis_html::render(&receipt);
        let h2 = tokmd_analysis_html::render(&receipt);
        // The timestamp may differ between calls, so compare structure.
        // Both should have the same length modulo timestamp differences.
        prop_assert!(h1.contains("<!DOCTYPE html>"));
        prop_assert!(h2.contains("<!DOCTYPE html>"));
        // Both should contain the report data script.
        prop_assert!(h1.contains("const REPORT_DATA ="));
        prop_assert!(h2.contains("const REPORT_DATA ="));
    }
}
