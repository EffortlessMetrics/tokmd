//! Property-based tests for tokmd-sensor types and invariants.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_envelope::{SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict};
use tokmd_substrate::{LangSummary, RepoSubstrate, SubstrateFile};

/// Generate an arbitrary `SubstrateFile`.
fn arb_substrate_file() -> impl Strategy<Value = SubstrateFile> {
    (
        "[a-z]+(/[a-z]+){0,3}\\.[a-z]{1,4}",
        "[A-Z][a-z]+",
        0usize..10_000,
        0usize..20_000,
        0usize..1_000_000,
        0usize..250_000,
        "[a-z]+(/[a-z]+){0,2}",
        any::<bool>(),
    )
        .prop_map(
            |(path, lang, code, lines, bytes, tokens, module, in_diff)| SubstrateFile {
                path,
                lang,
                code,
                lines,
                bytes,
                tokens,
                module,
                in_diff,
            },
        )
}

/// Build a `RepoSubstrate` from a vec of files with correct aggregates.
fn substrate_from_files(files: Vec<SubstrateFile>) -> RepoSubstrate {
    let mut lang_summary: BTreeMap<String, LangSummary> = BTreeMap::new();
    let mut total_tokens = 0usize;
    let mut total_bytes = 0usize;
    let mut total_code_lines = 0usize;

    for f in &files {
        let entry = lang_summary.entry(f.lang.clone()).or_insert(LangSummary {
            files: 0,
            code: 0,
            lines: 0,
            bytes: 0,
            tokens: 0,
        });
        entry.files += 1;
        entry.code += f.code;
        entry.lines += f.lines;
        entry.bytes += f.bytes;
        entry.tokens += f.tokens;

        total_tokens += f.tokens;
        total_bytes += f.bytes;
        total_code_lines += f.code;
    }

    RepoSubstrate {
        repo_root: ".".to_string(),
        files,
        lang_summary,
        diff_range: None,
        total_tokens,
        total_bytes,
        total_code_lines,
    }
}

proptest! {
    // ── SensorReport schema invariants ───────────────────────────────

    #[test]
    fn sensor_report_always_has_valid_schema(
        name in "[a-z][a-z0-9_-]{0,20}",
        version in "[0-9]+\\.[0-9]+\\.[0-9]+",
        mode in "[a-z]+",
        verdict in prop::sample::select(vec![
            Verdict::Pass, Verdict::Fail, Verdict::Warn,
            Verdict::Skip, Verdict::Pending,
        ]),
    ) {
        let report = SensorReport::new(
            ToolMeta::new(&name, &version, &mode),
            "2024-01-01T00:00:00Z".to_string(),
            verdict,
            "test summary".to_string(),
        );
        prop_assert_eq!(&report.schema, SENSOR_REPORT_SCHEMA,
            "Schema must be '{}'", SENSOR_REPORT_SCHEMA);
    }

    #[test]
    fn sensor_report_tool_meta_preserves_inputs(
        name in "[a-z][a-z0-9_-]{1,20}",
        version in "[0-9]+\\.[0-9]+\\.[0-9]+",
        mode in "[a-z]+",
    ) {
        let report = SensorReport::new(
            ToolMeta::new(&name, &version, &mode),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "test".to_string(),
        );
        prop_assert_eq!(&report.tool.name, &name);
        prop_assert_eq!(&report.tool.version, &version);
        prop_assert_eq!(&report.tool.mode, &mode);
        prop_assert!(!report.tool.name.is_empty(), "Sensor name must not be empty");
        prop_assert!(!report.tool.version.is_empty(), "Version must not be empty");
    }

    #[test]
    fn sensor_report_serde_roundtrip_preserves_schema(
        name in "[a-z][a-z0-9_-]{1,20}",
        version in "[0-9]+\\.[0-9]+\\.[0-9]+",
    ) {
        let report = SensorReport::new(
            ToolMeta::new(&name, &version, "check"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "roundtrip test".to_string(),
        );
        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.schema, SENSOR_REPORT_SCHEMA);
        prop_assert_eq!(&back.tool.name, &name);
        prop_assert_eq!(&back.tool.version, &version);
    }

    // ── Substrate consistency invariants ──────────────────────────────

    #[test]
    fn substrate_totals_equal_sum_of_files(
        files in prop::collection::vec(arb_substrate_file(), 0..30)
    ) {
        let substrate = substrate_from_files(files);

        let sum_tokens: usize = substrate.files.iter().map(|f| f.tokens).sum();
        let sum_bytes: usize = substrate.files.iter().map(|f| f.bytes).sum();
        let sum_code: usize = substrate.files.iter().map(|f| f.code).sum();

        prop_assert_eq!(substrate.total_tokens, sum_tokens,
            "total_tokens mismatch");
        prop_assert_eq!(substrate.total_bytes, sum_bytes,
            "total_bytes mismatch");
        prop_assert_eq!(substrate.total_code_lines, sum_code,
            "total_code_lines mismatch");
    }

    #[test]
    fn substrate_lang_summary_matches_file_aggregates(
        files in prop::collection::vec(arb_substrate_file(), 0..30)
    ) {
        let substrate = substrate_from_files(files);

        // Recompute lang summary from files
        let mut expected: BTreeMap<String, (usize, usize, usize, usize, usize)> = BTreeMap::new();
        for f in &substrate.files {
            let e = expected.entry(f.lang.clone()).or_default();
            e.0 += 1;      // files
            e.1 += f.code;  // code
            e.2 += f.lines; // lines
            e.3 += f.bytes; // bytes
            e.4 += f.tokens; // tokens
        }

        prop_assert_eq!(substrate.lang_summary.len(), expected.len(),
            "Language count mismatch");

        for (lang, summary) in &substrate.lang_summary {
            let (files, code, lines, bytes, tokens) = expected.get(lang)
                .expect("Language should exist");
            prop_assert_eq!(summary.files, *files, "files mismatch for {}", lang);
            prop_assert_eq!(summary.code, *code, "code mismatch for {}", lang);
            prop_assert_eq!(summary.lines, *lines, "lines mismatch for {}", lang);
            prop_assert_eq!(summary.bytes, *bytes, "bytes mismatch for {}", lang);
            prop_assert_eq!(summary.tokens, *tokens, "tokens mismatch for {}", lang);
        }
    }

    #[test]
    fn substrate_serde_roundtrip_preserves_totals(
        files in prop::collection::vec(arb_substrate_file(), 0..15)
    ) {
        let substrate = substrate_from_files(files);
        let json = serde_json::to_string(&substrate).unwrap();
        let back: RepoSubstrate = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(back.total_tokens, substrate.total_tokens);
        prop_assert_eq!(back.total_bytes, substrate.total_bytes);
        prop_assert_eq!(back.total_code_lines, substrate.total_code_lines);
        prop_assert_eq!(back.files.len(), substrate.files.len());
        prop_assert_eq!(back.lang_summary.len(), substrate.lang_summary.len());
    }
}
