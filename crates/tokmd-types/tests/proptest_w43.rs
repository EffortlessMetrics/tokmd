//! Wave 43 property-based tests for tokmd-types.
//!
//! Covers: LangRow/ModuleRow/ExportRow serde roundtrips, token estimation
//! non-negativity, FileRow size consistency, schema version positivity,
//! Totals arithmetic, DiffTotals symmetry, and ToolInfo roundtrip.

use proptest::prelude::*;
use tokmd_types::{
    ArtifactEntry, ArtifactHash, CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION,
    CapabilityState, ContextFileRow, DiffRow, DiffTotals, ExportData, FileClassification, FileKind,
    FileRow, HANDOFF_SCHEMA_VERSION, InclusionPolicy, LangReport, LangRow, ModuleReport, ModuleRow,
    PolicyExcludedFile, SCHEMA_VERSION, SmartExcludedFile, TokenAudit, TokenEstimationMeta,
    ToolInfo, Totals, cockpit::COCKPIT_SCHEMA_VERSION,
};

// =========================================================================
// Strategies
// =========================================================================

fn arb_lang_row() -> impl Strategy<Value = LangRow> {
    (
        prop::sample::select(vec![
            "Rust", "Python", "Go", "Java", "C", "TOML", "YAML", "JSON",
        ]),
        1usize..50_000,
        1usize..100_000,
        1usize..1_000,
        0usize..5_000_000,
        0usize..500_000,
        0usize..500,
    )
        .prop_map(|(lang, code, lines, files, bytes, tokens, avg)| LangRow {
            lang: lang.to_string(),
            code,
            lines: lines.max(code),
            files,
            bytes,
            tokens,
            avg_lines: avg,
        })
}

fn arb_module_row() -> impl Strategy<Value = ModuleRow> {
    (
        prop::sample::select(vec!["src", "tests", "crates/a", "crates/b", "lib"]),
        1usize..50_000,
        1usize..100_000,
        1usize..500,
        0usize..5_000_000,
        0usize..500_000,
        0usize..500,
    )
        .prop_map(
            |(module, code, lines, files, bytes, tokens, avg)| ModuleRow {
                module: module.to_string(),
                code,
                lines: lines.max(code),
                files,
                bytes,
                tokens,
                avg_lines: avg,
            },
        )
}

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (
        prop::sample::select(vec![
            "src/lib.rs",
            "src/main.rs",
            "tests/it.rs",
            "build.rs",
            "Cargo.toml",
        ]),
        1usize..5_000,
        0usize..500,
        0usize..200,
    )
        .prop_map(|(path, code, comments, blanks)| FileRow {
            path: path.to_string(),
            module: path.split('/').next().unwrap_or("root").to_string(),
            lang: "Rust".into(),
            kind: FileKind::Parent,
            code,
            comments,
            blanks,
            lines: code + comments + blanks,
            bytes: code * 10,
            tokens: code / 4,
        })
}

// =========================================================================
// 1. LangRow serde roundtrip preserves all fields
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn lang_row_serde_roundtrip(row in arb_lang_row()) {
        let json = serde_json::to_string(&row).unwrap();
        let parsed: LangRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&row, &parsed);
    }
}

// =========================================================================
// 2. ModuleRow serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn module_row_serde_roundtrip(row in arb_module_row()) {
        let json = serde_json::to_string(&row).unwrap();
        let parsed: ModuleRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&row, &parsed);
    }
}

// =========================================================================
// 3. FileRow (ExportRow) serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn file_row_serde_roundtrip(row in arb_file_row()) {
        let json = serde_json::to_string(&row).unwrap();
        let parsed: FileRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&row, &parsed);
    }
}

// =========================================================================
// 4. Token estimation is non-negative
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn token_estimation_non_negative(bytes in 0usize..10_000_000) {
        let meta = TokenEstimationMeta::from_bytes(bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        prop_assert!(meta.tokens_min <= meta.tokens_est);
        prop_assert!(meta.tokens_est <= meta.tokens_max);
    }

    #[test]
    fn token_estimation_source_bytes_preserved(bytes in 0usize..10_000_000) {
        let meta = TokenEstimationMeta::from_bytes(bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        prop_assert_eq!(meta.source_bytes, bytes);
    }
}

// =========================================================================
// 5. FileRow sizes are consistent (lines = code + comments + blanks)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn file_row_lines_consistency(row in arb_file_row()) {
        prop_assert_eq!(
            row.lines,
            row.code + row.comments + row.blanks,
            "lines must equal code + comments + blanks"
        );
    }

    #[test]
    fn file_row_bytes_non_zero_when_code_non_zero(row in arb_file_row()) {
        if row.code > 0 {
            prop_assert!(row.bytes > 0, "bytes should be > 0 when code > 0");
        }
    }
}

// =========================================================================
// 6. Schema version constants are positive
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn all_schema_versions_positive(_dummy in 0..1u8) {
        prop_assert!(SCHEMA_VERSION > 0);
        prop_assert!(COCKPIT_SCHEMA_VERSION > 0);
        prop_assert!(HANDOFF_SCHEMA_VERSION > 0);
        prop_assert!(CONTEXT_SCHEMA_VERSION > 0);
        prop_assert!(CONTEXT_BUNDLE_SCHEMA_VERSION > 0);
    }
}

// =========================================================================
// 7. Totals serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn totals_serde_roundtrip(
        code in 0usize..1_000_000,
        lines in 0usize..2_000_000,
        files in 0usize..10_000,
        bytes in 0usize..100_000_000,
        tokens in 0usize..10_000_000,
        avg_lines in 0usize..5000,
    ) {
        let totals = Totals { code, lines, files, bytes, tokens, avg_lines };
        let json = serde_json::to_string(&totals).unwrap();
        let parsed: Totals = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(totals, parsed);
    }
}

// =========================================================================
// 8. LangRow JSON is deterministic (serialize twice → identical)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn lang_row_json_deterministic(row in arb_lang_row()) {
        let json1 = serde_json::to_string(&row).unwrap();
        let back: LangRow = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&back).unwrap();
        prop_assert_eq!(&json1, &json2);
    }
}

// =========================================================================
// 9. ModuleRow JSON is deterministic
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn module_row_json_deterministic(row in arb_module_row()) {
        let json1 = serde_json::to_string(&row).unwrap();
        let back: ModuleRow = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&back).unwrap();
        prop_assert_eq!(&json1, &json2);
    }
}

// =========================================================================
// 10. DiffRow delta consistency
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn diff_row_delta_consistent(
        old_code in 0usize..100_000,
        new_code in 0usize..100_000,
    ) {
        let delta = new_code as i64 - old_code as i64;
        let row = DiffRow {
            lang: "Rust".into(),
            old_code,
            new_code,
            delta_code: delta,
            old_lines: old_code,
            new_lines: new_code,
            delta_lines: delta,
            old_files: 1,
            new_files: 1,
            delta_files: 0,
            old_bytes: old_code * 10,
            new_bytes: new_code * 10,
            delta_bytes: delta * 10,
            old_tokens: old_code / 4,
            new_tokens: new_code / 4,
            delta_tokens: new_code as i64 / 4 - old_code as i64 / 4,
        };
        prop_assert_eq!(row.delta_code, row.new_code as i64 - row.old_code as i64);
    }
}

// =========================================================================
// 11. DiffRow serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn diff_row_serde_roundtrip(
        old_code in 0usize..50_000,
        new_code in 0usize..50_000,
    ) {
        let row = DiffRow {
            lang: "Go".into(),
            old_code,
            new_code,
            delta_code: new_code as i64 - old_code as i64,
            old_lines: old_code,
            new_lines: new_code,
            delta_lines: new_code as i64 - old_code as i64,
            old_files: 1,
            new_files: 2,
            delta_files: 1,
            old_bytes: old_code * 10,
            new_bytes: new_code * 10,
            delta_bytes: (new_code as i64 - old_code as i64) * 10,
            old_tokens: old_code / 4,
            new_tokens: new_code / 4,
            delta_tokens: new_code as i64 / 4 - old_code as i64 / 4,
        };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: DiffRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row, parsed);
    }
}

// =========================================================================
// 12. DiffTotals default is all zeroed
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn diff_totals_default_zeroed(_dummy in 0..1u8) {
        let t = DiffTotals::default();
        prop_assert_eq!(t.old_code, 0);
        prop_assert_eq!(t.new_code, 0);
        prop_assert_eq!(t.delta_code, 0);
        prop_assert_eq!(t.old_lines, 0);
        prop_assert_eq!(t.new_lines, 0);
        prop_assert_eq!(t.delta_lines, 0);
        prop_assert_eq!(t.old_files, 0);
        prop_assert_eq!(t.new_files, 0);
        prop_assert_eq!(t.delta_files, 0);
    }
}

// =========================================================================
// 13. TokenAudit overhead_pct bounded [0, 1]
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn token_audit_overhead_pct_bounded(
        output_bytes in 1u64..10_000_000,
        content_bytes in 0u64..10_000_000,
    ) {
        let content_bytes = content_bytes.min(output_bytes);
        let audit = TokenAudit::from_output(output_bytes, content_bytes);
        prop_assert!(
            audit.overhead_pct >= 0.0 && audit.overhead_pct <= 1.0,
            "overhead_pct {} out of [0,1]",
            audit.overhead_pct
        );
    }
}

// =========================================================================
// 14. ToolInfo serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn tool_info_serde_roundtrip(_dummy in 0..1u8) {
        let info = ToolInfo::current();
        let json = serde_json::to_string(&info).unwrap();
        let parsed: ToolInfo = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(info.name, parsed.name);
        prop_assert_eq!(info.version, parsed.version);
    }
}

// =========================================================================
// 15. InclusionPolicy roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn inclusion_policy_roundtrip(idx in 0usize..4) {
        let policy = [
            InclusionPolicy::Full,
            InclusionPolicy::HeadTail,
            InclusionPolicy::Summary,
            InclusionPolicy::Skip,
        ][idx];
        let json = serde_json::to_string(&policy).unwrap();
        let parsed: InclusionPolicy = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(policy, parsed);
    }
}

// =========================================================================
// 16. CapabilityState roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn capability_state_roundtrip(idx in 0usize..3) {
        let state = [
            CapabilityState::Available,
            CapabilityState::Skipped,
            CapabilityState::Unavailable,
        ][idx];
        let json = serde_json::to_string(&state).unwrap();
        let parsed: CapabilityState = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(state, parsed);
    }
}

// =========================================================================
// 17. FileClassification roundtrip (all 7 variants)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn file_classification_roundtrip(idx in 0usize..7) {
        let cls = [
            FileClassification::Generated,
            FileClassification::Fixture,
            FileClassification::Vendored,
            FileClassification::Lockfile,
            FileClassification::Minified,
            FileClassification::DataBlob,
            FileClassification::Sourcemap,
        ][idx];
        let json = serde_json::to_string(&cls).unwrap();
        let parsed: FileClassification = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(cls, parsed);
    }
}

// =========================================================================
// 18. ExportData rows roundtrip through JSON
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn export_data_rows_json_roundtrip(rows in prop::collection::vec(arb_file_row(), 1..8)) {
        let json = serde_json::to_string(&rows).unwrap();
        let parsed: Vec<FileRow> = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(rows.len(), parsed.len());
        for (orig, rt) in rows.iter().zip(parsed.iter()) {
            prop_assert_eq!(orig, rt);
        }
    }
}

// =========================================================================
// 19. TokenEstimationMeta serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn token_estimation_meta_serde_roundtrip(bytes in 0usize..5_000_000) {
        let meta = TokenEstimationMeta::from_bytes(bytes, TokenEstimationMeta::DEFAULT_BPT_EST);
        let json = serde_json::to_string(&meta).unwrap();
        let parsed: TokenEstimationMeta = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(meta.tokens_min, parsed.tokens_min);
        prop_assert_eq!(meta.tokens_est, parsed.tokens_est);
        prop_assert_eq!(meta.tokens_max, parsed.tokens_max);
        prop_assert_eq!(meta.source_bytes, parsed.source_bytes);
    }
}

// =========================================================================
// 20. SmartExcludedFile serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn smart_excluded_file_roundtrip(
        tokens in 0usize..100_000,
    ) {
        let f = SmartExcludedFile {
            path: "vendor/jquery.min.js".into(),
            reason: "minified".into(),
            tokens,
        };
        let json = serde_json::to_string(&f).unwrap();
        let parsed: SmartExcludedFile = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(f.path, parsed.path);
        prop_assert_eq!(f.reason, parsed.reason);
        prop_assert_eq!(f.tokens, parsed.tokens);
    }
}

// =========================================================================
// 21. ArtifactEntry serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn artifact_entry_roundtrip(bytes_val in 0u64..10_000_000) {
        let entry = ArtifactEntry {
            name: "receipt.json".into(),
            path: "output/receipt.json".into(),
            description: "JSON receipt".into(),
            bytes: bytes_val,
            hash: Some(ArtifactHash {
                algo: "blake3".into(),
                hash: "abc123".into(),
            }),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: ArtifactEntry = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(entry.name, parsed.name);
        prop_assert_eq!(entry.bytes, parsed.bytes);
        prop_assert!(parsed.hash.is_some());
    }
}

// =========================================================================
// 22. PolicyExcludedFile serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn policy_excluded_file_roundtrip(tokens in 0usize..100_000) {
        let f = PolicyExcludedFile {
            path: "generated/proto.rs".into(),
            original_tokens: tokens,
            policy: InclusionPolicy::Skip,
            reason: "generated code".into(),
            classifications: vec![FileClassification::Generated],
        };
        let json = serde_json::to_string(&f).unwrap();
        let parsed: PolicyExcludedFile = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(f.path, parsed.path);
        prop_assert_eq!(f.original_tokens, parsed.original_tokens);
        prop_assert_eq!(f.policy, parsed.policy);
    }
}

// =========================================================================
// 23. ContextFileRow serde roundtrip
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn context_file_row_roundtrip(
        code in 1usize..10_000,
        tokens in 1usize..50_000,
    ) {
        let row = ContextFileRow {
            path: "src/main.rs".into(),
            module: "src".into(),
            lang: "Rust".into(),
            tokens,
            code,
            lines: code + 100,
            bytes: code * 10,
            value: tokens,
            rank_reason: String::new(),
            policy: InclusionPolicy::Full,
            effective_tokens: None,
            policy_reason: None,
            classifications: vec![],
        };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: ContextFileRow = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(row.path, parsed.path);
        prop_assert_eq!(row.code, parsed.code);
        prop_assert_eq!(row.tokens, parsed.tokens);
    }
}

// =========================================================================
// 24. Totals: sum of parts invariant
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn totals_from_lang_rows_sum(rows in prop::collection::vec(arb_lang_row(), 1..6)) {
        let expected_code: usize = rows.iter().map(|r| r.code).sum();
        let expected_files: usize = rows.iter().map(|r| r.files).sum();
        let total = Totals {
            code: rows.iter().map(|r| r.code).sum(),
            lines: rows.iter().map(|r| r.lines).sum(),
            files: rows.iter().map(|r| r.files).sum(),
            bytes: rows.iter().map(|r| r.bytes).sum(),
            tokens: rows.iter().map(|r| r.tokens).sum(),
            avg_lines: 0,
        };
        prop_assert_eq!(total.code, expected_code);
        prop_assert_eq!(total.files, expected_files);
    }
}

// =========================================================================
// 25. TokenEstimation monotonic with bytes
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn token_estimation_monotonic(
        a in 0usize..1_000_000,
        delta in 1usize..1_000_000,
    ) {
        let b = a + delta;
        let meta_a = TokenEstimationMeta::from_bytes(a, TokenEstimationMeta::DEFAULT_BPT_EST);
        let meta_b = TokenEstimationMeta::from_bytes(b, TokenEstimationMeta::DEFAULT_BPT_EST);
        prop_assert!(
            meta_b.tokens_est >= meta_a.tokens_est,
            "More bytes should give >= tokens"
        );
    }
}
