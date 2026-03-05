//! Targeted tests for mutation testing coverage gaps (W50).
//!
//! Each test catches common mutations: replacing operators,
//! negating conditions, removing statements.

use tokmd_types::{
    CONTEXT_BUNDLE_SCHEMA_VERSION, CONTEXT_SCHEMA_VERSION, ChildIncludeMode, ChildrenMode,
    DiffTotals, FileKind, FileRow, HANDOFF_SCHEMA_VERSION, LangRow, SCHEMA_VERSION, TokenAudit,
    TokenEstimationMeta, Totals, cockpit::COCKPIT_SCHEMA_VERSION,
};

// ---------------------------------------------------------------------------
// 1. Schema version constants have expected values
// ---------------------------------------------------------------------------

#[test]
fn schema_version_constants_correct() {
    assert_eq!(SCHEMA_VERSION, 2, "core schema");
    assert_eq!(HANDOFF_SCHEMA_VERSION, 5, "handoff schema");
    assert_eq!(CONTEXT_BUNDLE_SCHEMA_VERSION, 2, "context bundle schema");
    assert_eq!(CONTEXT_SCHEMA_VERSION, 4, "context schema");
    assert_eq!(COCKPIT_SCHEMA_VERSION, 3, "cockpit schema");
}

// ---------------------------------------------------------------------------
// 2. Schema versions are all > 0
// ---------------------------------------------------------------------------

#[test]
fn schema_versions_positive() {
    assert!(SCHEMA_VERSION > 0);
    assert!(HANDOFF_SCHEMA_VERSION > 0);
    assert!(CONTEXT_BUNDLE_SCHEMA_VERSION > 0);
    assert!(CONTEXT_SCHEMA_VERSION > 0);
    assert!(COCKPIT_SCHEMA_VERSION > 0);
}

// ---------------------------------------------------------------------------
// 3. ChildrenMode::Collapse != ChildrenMode::Separate
// ---------------------------------------------------------------------------

#[test]
fn children_mode_variants_differ() {
    assert_ne!(ChildrenMode::Collapse, ChildrenMode::Separate);
}

// ---------------------------------------------------------------------------
// 4. ChildIncludeMode variants differ
// ---------------------------------------------------------------------------

#[test]
fn child_include_mode_variants_differ() {
    assert_ne!(ChildIncludeMode::Separate, ChildIncludeMode::ParentsOnly);
}

// ---------------------------------------------------------------------------
// 5. FileKind ordering: Parent < Child
// ---------------------------------------------------------------------------

#[test]
fn file_kind_ordering() {
    assert!(FileKind::Parent < FileKind::Child);
    assert_ne!(FileKind::Parent, FileKind::Child);
}

// ---------------------------------------------------------------------------
// 6. LangRow: row with more code sorts before row with less (desc code sort)
// ---------------------------------------------------------------------------

#[test]
fn lang_row_sort_by_code_desc() {
    let mut rows = vec![
        LangRow {
            lang: "Python".to_string(),
            code: 100,
            lines: 150,
            files: 2,
            bytes: 4000,
            tokens: 1000,
            avg_lines: 75,
        },
        LangRow {
            lang: "Rust".to_string(),
            code: 500,
            lines: 700,
            files: 5,
            bytes: 20000,
            tokens: 5000,
            avg_lines: 140,
        },
    ];

    // Sort descending by code, then by name
    rows.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));

    assert_eq!(rows[0].lang, "Rust", "Rust (500 code) should sort first");
    assert_eq!(
        rows[1].lang, "Python",
        "Python (100 code) should sort second"
    );
}

// ---------------------------------------------------------------------------
// 7. Totals struct equality
// ---------------------------------------------------------------------------

#[test]
fn totals_equality() {
    let a = Totals {
        code: 100,
        lines: 150,
        files: 5,
        bytes: 4000,
        tokens: 1000,
        avg_lines: 30,
    };
    let b = a.clone();
    assert_eq!(a, b);

    let c = Totals {
        code: 99, // different
        ..a.clone()
    };
    assert_ne!(a, c, "differing code should make totals unequal");
}

// ---------------------------------------------------------------------------
// 8. FileRow equality checks all fields
// ---------------------------------------------------------------------------

#[test]
fn file_row_equality() {
    let r1 = FileRow {
        path: "src/lib.rs".to_string(),
        module: "src".to_string(),
        lang: "Rust".to_string(),
        kind: FileKind::Parent,
        code: 100,
        comments: 20,
        blanks: 10,
        lines: 130,
        bytes: 4000,
        tokens: 1000,
    };
    let r2 = r1.clone();
    assert_eq!(r1, r2);

    let r3 = FileRow {
        path: "src/main.rs".to_string(),
        ..r1.clone()
    };
    assert_ne!(r1, r3, "different path should break equality");
}

// ---------------------------------------------------------------------------
// 9. DiffTotals default is all zeros
// ---------------------------------------------------------------------------

#[test]
fn diff_totals_default_zeros() {
    let dt = DiffTotals::default();
    assert_eq!(dt.old_code, 0);
    assert_eq!(dt.new_code, 0);
    assert_eq!(dt.delta_code, 0);
    assert_eq!(dt.old_files, 0);
    assert_eq!(dt.new_files, 0);
    assert_eq!(dt.delta_files, 0);
}

// ---------------------------------------------------------------------------
// 10. TokenEstimationMeta invariant: min <= est <= max
// ---------------------------------------------------------------------------

#[test]
fn token_estimation_invariant() {
    let est = TokenEstimationMeta::from_bytes(4000, 4.0);
    assert!(
        est.tokens_min <= est.tokens_est,
        "tokens_min ({}) must be <= tokens_est ({})",
        est.tokens_min,
        est.tokens_est
    );
    assert!(
        est.tokens_est <= est.tokens_max,
        "tokens_est ({}) must be <= tokens_max ({})",
        est.tokens_est,
        est.tokens_max
    );
    assert_eq!(est.source_bytes, 4000);
}

// ---------------------------------------------------------------------------
// 11. TokenAudit overhead calculation
// ---------------------------------------------------------------------------

#[test]
fn token_audit_overhead() {
    let audit = TokenAudit::from_output(5000, 4500);
    assert_eq!(audit.output_bytes, 5000);
    assert_eq!(audit.overhead_bytes, 500);
    assert!(audit.overhead_pct > 0.0);
    assert!(audit.overhead_pct < 1.0);
}

// ---------------------------------------------------------------------------
// 12. Serde roundtrip for LangRow
// ---------------------------------------------------------------------------

#[test]
fn lang_row_serde_roundtrip() {
    let row = LangRow {
        lang: "Rust".to_string(),
        code: 500,
        lines: 700,
        files: 5,
        bytes: 20000,
        tokens: 5000,
        avg_lines: 140,
    };
    let json = serde_json::to_string(&row).unwrap();
    let back: LangRow = serde_json::from_str(&json).unwrap();
    assert_eq!(row, back);
}
