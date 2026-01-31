//! Fuzz target for TOML configuration parsing.
//!
//! Tests `TomlConfig::parse()` with arbitrary TOML input to find
//! panics, hangs, or excessive memory usage in the TOML deserializer.
//! After successful parse, exercises serialization round-trip (both JSON and TOML)
//! and field access.

#![no_main]
use libfuzzer_sys::fuzz_target;
use tokmd_config::TomlConfig;

/// Max input size to prevent pathological parse times
const MAX_INPUT_SIZE: usize = 64 * 1024; // 64KB

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_SIZE {
        return;
    }
    if let Ok(s) = std::str::from_utf8(data) {
        // Try parsing as TOML config
        if let Ok(config) = TomlConfig::parse(s) {
            // Capture original field values for round-trip verification
            let orig_paths = config.scan.paths.clone();
            let orig_roots = config.module.roots.clone();
            let orig_depth = config.module.depth;
            let orig_view_count = config.view.len();
            let orig_export_format = config.export.format.clone();
            let orig_analyze_preset = config.analyze.preset.clone();

            // Exercise JSON serialization round-trip
            if let Ok(json) = serde_json::to_string(&config) {
                // Round-trip through JSON
                if let Ok(roundtrip) = serde_json::from_str::<TomlConfig>(&json) {
                    // Verify key fields match after round-trip
                    assert_eq!(
                        roundtrip.scan.paths, orig_paths,
                        "scan.paths must survive JSON round-trip"
                    );
                    assert_eq!(
                        roundtrip.module.roots, orig_roots,
                        "module.roots must survive JSON round-trip"
                    );
                    assert_eq!(
                        roundtrip.module.depth, orig_depth,
                        "module.depth must survive JSON round-trip"
                    );
                    assert_eq!(
                        roundtrip.view.len(),
                        orig_view_count,
                        "view count must survive JSON round-trip"
                    );
                    assert_eq!(
                        roundtrip.export.format, orig_export_format,
                        "export.format must survive JSON round-trip"
                    );
                    assert_eq!(
                        roundtrip.analyze.preset, orig_analyze_preset,
                        "analyze.preset must survive JSON round-trip"
                    );
                }
            }

            // Exercise TOML serialization round-trip (higher value for TOML-specific edge cases)
            if let Ok(toml_str) = toml::to_string(&config) {
                // Round-trip through TOML - parse the serialized output
                if let Ok(roundtrip) = TomlConfig::parse(&toml_str) {
                    // Verify key fields match after TOML round-trip
                    assert_eq!(
                        roundtrip.scan.paths, orig_paths,
                        "scan.paths must survive TOML round-trip"
                    );
                    assert_eq!(
                        roundtrip.module.roots, orig_roots,
                        "module.roots must survive TOML round-trip"
                    );
                    assert_eq!(
                        roundtrip.module.depth, orig_depth,
                        "module.depth must survive TOML round-trip"
                    );
                    // View profiles may have ordering differences, so just check count
                    assert_eq!(
                        roundtrip.view.len(),
                        orig_view_count,
                        "view count must survive TOML round-trip"
                    );
                }
            }

            // Verify scan.paths consistency (if present, all entries should be non-empty strings)
            if let Some(ref paths) = config.scan.paths {
                for (i, path) in paths.iter().enumerate() {
                    // Paths can be empty strings in TOML, so we just verify they're valid UTF-8
                    // (which they are if we got here)
                    assert!(
                        path.len() <= MAX_INPUT_SIZE,
                        "scan.paths[{i}] should not exceed max input size"
                    );
                }
            }

            // Verify module.roots consistency
            if let Some(ref roots) = config.module.roots {
                for (i, root) in roots.iter().enumerate() {
                    assert!(
                        root.len() <= MAX_INPUT_SIZE,
                        "module.roots[{i}] should not exceed max input size"
                    );
                }
            }

            // Access nested fields to exercise structure traversal
            let _ = config.scan.paths.as_ref().map(|p| p.len());
            let _ = config.module.roots.as_ref().map(|r| r.len());
            let _ = config.view.len();
        }
    }
});
