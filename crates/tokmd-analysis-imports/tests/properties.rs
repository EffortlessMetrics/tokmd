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
}
