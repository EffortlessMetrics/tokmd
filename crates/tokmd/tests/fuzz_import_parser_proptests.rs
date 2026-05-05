//! Deterministic property tests extracted from `fuzz_import_parser`.
//!
//! Validates invariants for:
//! - Import parsing limits and stability

use proptest::prelude::*;

// We use relative path from tests directory
#[path = "../../tokmd-analysis/src/imports/parser.rs"]
mod imports;

use imports::{normalize_import_target, parse_imports, supports_language};

proptest! {
    #[test]
    fn import_parser_invariants(
        lang in "\\PC+",
        body in "\\PC*"
    ) {
        let lines: Vec<&str> = body.lines().take(512).collect();
        let imports = parse_imports(&lang, &lines);

        let _ = supports_language(&lang);
        for import in imports {
            let normalized = normalize_import_target(&import);
            let double_normalized = normalize_import_target(&normalized);
            prop_assert_eq!(&normalized, &double_normalized);
        }
    }
}
