//! Property-based tests for tokmd-analysis-imports.

use proptest::prelude::*;
use tokmd_analysis_imports::{normalize_import_target, parse_imports, supports_language};

fn arb_supported_lang() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("rust"),
        Just("javascript"),
        Just("typescript"),
        Just("python"),
        Just("go"),
    ]
}

proptest! {
    #[test]
    fn parse_imports_is_deterministic(
        lang in "[a-zA-Z]{0,16}",
        lines in prop::collection::vec("[ -~]{0,120}", 0..64)
    ) {
        let first = parse_imports(&lang, &lines);
        let second = parse_imports(&lang, &lines);
        prop_assert_eq!(first, second);
    }

    #[test]
    fn unsupported_languages_return_empty(
        lang in "[a-zA-Z0-9_]{0,16}",
        lines in prop::collection::vec("[ -~]{0,120}", 0..64)
    ) {
        let lower = lang.to_ascii_lowercase();
        prop_assume!(!matches!(lower.as_str(), "rust" | "javascript" | "typescript" | "python" | "go"));
        prop_assert!(parse_imports(&lang, &lines).is_empty());
    }

    #[test]
    fn supported_languages_are_case_insensitive(
        lang in arb_supported_lang(),
        uppercase in any::<bool>()
    ) {
        let candidate = if uppercase {
            lang.to_ascii_uppercase()
        } else {
            lang.to_string()
        };
        prop_assert!(supports_language(&candidate));
    }

    #[test]
    fn relative_targets_normalize_to_local(
        suffix in "[a-zA-Z0-9_./-]{0,32}"
    ) {
        let target = format!(".{}", suffix);
        prop_assert_eq!(normalize_import_target(&target), "local");
    }

    #[test]
    fn normalize_import_target_is_deterministic(
        target in "[a-zA-Z0-9_./:'\"-]{0,80}"
    ) {
        let first = normalize_import_target(&target);
        let second = normalize_import_target(&target);
        prop_assert_eq!(first, second);
    }

    #[test]
    fn rust_use_lines_always_produce_one_import(
        crate_name in "[a-z_][a-z0-9_]{0,15}"
    ) {
        let line = format!("use {}::Thing;", crate_name);
        let imports = parse_imports("rust", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &crate_name);
    }

    #[test]
    fn python_import_lines_always_produce_one_import(
        module in "[a-z][a-z0-9_]{0,15}"
    ) {
        let line = format!("import {}", module);
        let imports = parse_imports("python", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &module);
    }

    #[test]
    fn python_from_lines_always_produce_one_import(
        module in "[a-z][a-z0-9_]{0,15}"
    ) {
        let line = format!("from {} import thing", module);
        let imports = parse_imports("python", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &module);
    }

    #[test]
    fn go_single_import_always_produces_one_import(
        pkg in "[a-z]{1,12}"
    ) {
        let line = format!(r#"import "{}""#, pkg);
        let imports = parse_imports("go", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &pkg);
    }

    #[test]
    fn js_require_always_produces_one_import(
        pkg in "[a-z][a-z0-9-]{0,15}"
    ) {
        let line = format!(r#"const x = require("{}");"#, pkg);
        let imports = parse_imports("javascript", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &pkg);
    }

    #[test]
    fn js_import_from_always_produces_one_import(
        pkg in "[a-z][a-z0-9-]{0,15}"
    ) {
        let line = format!(r#"import x from "{}";"#, pkg);
        let imports = parse_imports("javascript", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &pkg);
    }

    #[test]
    fn normalize_never_returns_empty_for_nonempty_alpha(
        target in "[a-zA-Z][a-zA-Z0-9_./-]{0,30}"
    ) {
        let result = normalize_import_target(&target);
        prop_assert!(!result.is_empty());
    }

    #[test]
    fn parse_imports_output_count_le_input_lines(
        lang in arb_supported_lang(),
        lines in prop::collection::vec("[ -~]{0,120}", 0..64)
    ) {
        let imports = parse_imports(lang, &lines);
        prop_assert!(imports.len() <= lines.len());
    }

    #[test]
    fn typescript_and_javascript_parse_identically(
        lines in prop::collection::vec("[ -~]{0,120}", 0..32)
    ) {
        let js = parse_imports("javascript", &lines);
        let ts = parse_imports("typescript", &lines);
        prop_assert_eq!(js, ts);
    }

    #[test]
    fn rust_mod_lines_always_produce_one_import(
        name in "[a-z][a-z0-9_]{0,15}"
    ) {
        let line = format!("mod {};", name);
        let imports = parse_imports("rust", &[line]);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &name);
    }

    #[test]
    fn go_block_each_quoted_line_produces_one_import(
        pkg in "[a-z]{1,12}"
    ) {
        let lines = vec![
            "import (".to_string(),
            format!(r#""{}""#, pkg),
            ")".to_string(),
        ];
        let imports = parse_imports("go", &lines);
        prop_assert_eq!(imports.len(), 1);
        prop_assert_eq!(&imports[0], &pkg);
    }

    #[test]
    fn empty_lines_never_produce_imports(
        lang in arb_supported_lang(),
        count in 0usize..32
    ) {
        let lines: Vec<String> = vec![String::new(); count];
        let imports = parse_imports(lang, &lines);
        prop_assert!(imports.is_empty());
    }

    #[test]
    fn python_nested_package_normalizes_to_first_segment(
        root in "[a-z]{1,8}",
        child in "[a-z]{1,8}"
    ) {
        let target = format!("{}.{}", root, child);
        let normalized = normalize_import_target(&target);
        prop_assert_eq!(normalized, root);
    }
}
