//! Contract-change detection for cockpit receipts.

use tokmd_types::cockpit::Contracts;

/// Detect contract changes.
pub fn detect_contracts<S: AsRef<str>>(files: &[S]) -> Contracts {
    let mut api_changed = false;
    let mut cli_changed = false;
    let mut schema_changed = false;
    let mut breaking_indicators = 0;

    for file in files.iter() {
        if file.as_ref().ends_with("lib.rs") || file.as_ref().ends_with("mod.rs") {
            api_changed = true;
        }
        if file.as_ref().contains("crates/tokmd/src/commands/")
            || file.as_ref().contains("crates/tokmd/src/cli/")
            || file.as_ref() == "crates/tokmd/src/config.rs"
        {
            cli_changed = true;
        }
        if file.as_ref() == "docs/schema.json" || file.as_ref() == "docs/SCHEMA.md" {
            schema_changed = true;
        }
    }

    if api_changed {
        breaking_indicators += 1;
    }
    if schema_changed {
        breaking_indicators += 1;
    }

    Contracts {
        api_changed,
        cli_changed,
        schema_changed,
        breaking_indicators,
    }
}
