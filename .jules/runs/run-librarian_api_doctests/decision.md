# Option A: Fix CLI default argument docs
- what it is: The `CliModuleArgs` and `CliExportArgs` and tests for resolve_module_with_config describe defaults of `module_roots` as `["crates", "packages"]`, but the default is actually `vec!["crates".into(), "packages".into()]`. The tests check for `"crates".to_string(), "packages".to_string()`. This is fine. The code passes the test. But I noticed I couldn't run tests on `tokmd-config`. Ah, the crate is `tokmd-config`, but there were no tests. Actually, I want to improve doctests in `tokmd/src/config.rs`. The doctests in `tokmd/src/config.rs` for `resolve_module_with_config` and `resolve_module` have hardcoded `vec!["crates".to_string(), "packages".to_string()]` assertions, but the `module_roots` argument doesn't seem to map correctly to this in all scenarios. Wait, the tests pass. So where is the missing doctest/example coverage?

Wait, `CliTableFormat` vs `CliExportFormat`. The memory says: "In the tokmd_types crate, the TableFormat enum includes Tsv but not Csv, whereas the ExportFormat enum includes Csv but not Tsv. Ensure correct variant usage when asserting CLI argument format resolutions in tests." Also "In the tokmd-config crate, CLI argument struct types like CliModuleArgs and CliExportArgs do not derive Default. When initializing them in tests, you must explicitly provide all fields (usually as None or false) instead of using ..Default::default()." And "In tokmd, fields of ResolvedConfig (such as config.toml) expect references to configuration objects (e.g., Option<&TomlConfig>), not owned instances. Ensure you pass borrowed references when manually constructing ResolvedConfig in tests."

Option A: Add missing doctests for CLI config resolution.
- what it is: Add comprehensive doctests for `resolve_lang_with_config`, `resolve_module_with_config`, and `resolve_export_with_config` to cover the `ResolvedConfig` resolution precedence (CLI > View Profile > TOML > JSON > Default).
- why it fits this repo and shard: It adds executable coverage to the public configuration resolution API, matching the Gatekeeper/Librarian `docs-executable` profile.
- trade-offs: Structure / Velocity / Governance. Excellent value.

Option B: Rewrite docs and fix prose drift.
- what it is: Just rewriting the rustdoc text.
- when to choose it instead: If the docs were fundamentally misleading in prose rather than lacking executable examples.
- trade-offs: Anti-drift rules state "Do not land tone-only prose rewrites." Option A is better.

I choose Option A.
