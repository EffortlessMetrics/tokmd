//! Oracle anchoring `docs/specs/ast-shadow-backend.md` to emitted wire values.
//!
//! The backend spec defines two documentation-level vocabularies layered over
//! the artifacts in `docs/specs/ast-shadow.md` and `docs/specs/syntax-receipts.md`:
//!
//! 1. a *backend identity* mapping (`heuristic` / `tree-sitter`) derived from the
//!    `tokmd.ast_shadow.v1` `kind` and the `tree-sitter-*` `parser_crate`; and
//! 2. a *backend-aware mismatch taxonomy* that unifies the diff comparison
//!    buckets and the advisory parse statuses under one vocabulary.
//!
//! The spec is intentionally additive: it adds no `backend_id` wire field and
//! changes no default receipt. These tests guard against the spec silently
//! drifting from the code by asserting that every wire value a real producer
//! emits maps onto exactly one documented identity / mismatch kind, and that the
//! documented receipt-status taxonomy is total over `SyntaxParseStatus`.
//!
//! Claim boundary: this anchors the spec's *documentation tables* to the wire
//! strings the shadow and syntax-receipt producers emit. It does not add a
//! backend identity wire field, does not change default behavior, and asserts
//! nothing about parse correctness or AST fact accuracy.
//!
//! The mapping helpers and JSON accessors are fallible so the oracle stays
//! panic-free: an undocumented wire value or a missing field surfaces as a test
//! error rather than a panic-family call.
#![cfg(feature = "ast")]

use serde_json::Value;
use tokmd_analysis::ast::{
    AstLanguage, ShadowFileInput, ShadowLandmark, SyntaxParseOptions, SyntaxParseStatus,
    build_shadow_artifacts, parse_syntax_receipt,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const TREE_SITTER_PREFIX: &str = "tree-sitter-";

fn str_field<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("expected string field `{key}`"))
}

fn array_field<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("expected array field `{key}`"))
}

/// Spec "Outputs" backend-identity table (wire value -> backend identity).
fn backend_identity_for_shadow_kind(kind: &str) -> Result<&'static str, String> {
    match kind {
        "heuristic" => Ok("heuristic"),
        "ast" => Ok("tree-sitter"),
        other => Err(format!("undocumented tokmd.ast_shadow.v1 kind: {other}")),
    }
}

/// Spec "Backend-aware mismatch taxonomy" table for `tokmd.ast_shadow.v1` diff
/// `status` values. `compared` is the healthy diff state whose per-landmark
/// buckets carry the `agree` / `heuristic_only` / `backend_only` kinds, so it is
/// not itself an advisory mismatch kind (modelled as `Ok(None)`).
fn taxonomy_for_diff_status(status: &str) -> Result<Option<&'static str>, String> {
    match status {
        "compared" => Ok(None),
        "parse_degraded" => Ok(Some("parse_degraded")),
        "unsupported" => Ok(Some("unsupported")),
        other => Err(format!("undocumented diff status: {other}")),
    }
}

/// Spec "Backend-aware mismatch taxonomy" table for the diff comparison buckets.
fn taxonomy_for_diff_bucket(bucket: &str) -> Result<&'static str, String> {
    match bucket {
        "matches" => Ok("agree"),
        "heuristic_only" => Ok("heuristic_only"),
        "ast_only" => Ok("backend_only"),
        other => Err(format!("undocumented diff bucket: {other}")),
    }
}

/// Spec "Backend-aware mismatch taxonomy" table for `tokmd.syntax_receipt.v1`
/// `status` values. `complete` is the healthy parser-backed state, not an
/// advisory mismatch kind (modelled as `Ok(None)`).
fn taxonomy_for_receipt_status(status: &str) -> Result<Option<&'static str>, String> {
    match status {
        "complete" => Ok(None),
        "parse_degraded" => Ok(Some("parse_degraded")),
        "parser_failed" => Ok(Some("parser_failed")),
        "unsupported_language" => Ok(Some("unsupported")),
        "skipped_generated_or_vendor" | "skipped_too_large" => Ok(Some("skipped")),
        other => Err(format!("undocumented syntax receipt status: {other}")),
    }
}

#[test]
fn ast_shadow_wire_kinds_map_to_documented_backend_identity() -> TestResult {
    let files = [ShadowFileInput {
        path: "src/lib.rs",
        language: AstLanguage::Rust,
        source: "fn top_level() {}\n",
        heuristic_landmarks: &[],
    }];

    let artifacts = build_shadow_artifacts(&files)?;

    let heuristic_kind = str_field(&artifacts.heuristic, "kind")?;
    assert_eq!(
        backend_identity_for_shadow_kind(heuristic_kind)?,
        "heuristic"
    );

    let ast_kind = str_field(&artifacts.ast, "kind")?;
    assert_eq!(backend_identity_for_shadow_kind(ast_kind)?, "tree-sitter");

    // Every declared shadow capability is a `tree-sitter-*` crate, so the `ast`
    // kind maps unambiguously to the `tree-sitter` backend identity.
    let capabilities = array_field(&artifacts.ast, "capabilities")?;
    assert!(
        !capabilities.is_empty(),
        "ast artifact should declare at least one parser capability"
    );
    for capability in capabilities {
        let parser_crate = str_field(capability, "parser_crate")?;
        assert!(
            parser_crate.starts_with(TREE_SITTER_PREFIX),
            "capability {parser_crate} should map to the tree-sitter backend identity"
        );
    }

    Ok(())
}

#[test]
fn ast_syntax_receipt_parser_crate_maps_to_tree_sitter_identity() -> TestResult {
    let receipt = parse_syntax_receipt(
        "src/lib.rs",
        "fn top_level() {}\n",
        SyntaxParseOptions::default(),
    );
    let value = receipt.to_value();

    assert_eq!(str_field(&value, "schema")?, "tokmd.syntax_receipt.v1");
    let parser_crate = str_field(&value, "parser_crate")?;
    assert!(
        parser_crate.starts_with(TREE_SITTER_PREFIX),
        "{parser_crate} should map to the tree-sitter backend identity"
    );

    Ok(())
}

#[test]
fn ast_shadow_diff_wire_values_map_to_documented_mismatch_taxonomy() -> TestResult {
    // Exercise every diff-level outcome: agree + backend_only (parser finds an
    // extra function), heuristic_only, parse_degraded (recovered syntax error),
    // and unsupported (no parser backend for the language).
    let agree_landmark = [ShadowLandmark::function("top_level", 1, 1)];
    let heuristic_only_landmark = [ShadowLandmark::function("ghost", 1, 1)];
    let unsupported_landmark = [ShadowLandmark::function("run", 1, 1)];
    let files = [
        ShadowFileInput {
            path: "src/agree.rs",
            language: AstLanguage::Rust,
            source: "fn top_level() {}\nfn extra() {}\n",
            heuristic_landmarks: &agree_landmark,
        },
        ShadowFileInput {
            path: "src/heuristic_only.rs",
            language: AstLanguage::Rust,
            source: "fn other() {}\n",
            heuristic_landmarks: &heuristic_only_landmark,
        },
        ShadowFileInput {
            path: "src/degraded.rs",
            language: AstLanguage::Rust,
            source: "fn ok() {}\nfn broken(",
            heuristic_landmarks: &[],
        },
        ShadowFileInput {
            path: "tools/run.py",
            language: AstLanguage::Python,
            source: "def run():\n    return 1\n",
            heuristic_landmarks: &unsupported_landmark,
        },
    ];

    let artifacts = build_shadow_artifacts(&files)?;
    let diff_files = array_field(&artifacts.diff, "files")?;

    let mut observed_statuses = Vec::new();
    let mut observed_kinds = Vec::new();
    for file in diff_files {
        let status = str_field(file, "status")?;
        observed_statuses.push(status.to_owned());
        // Errors on any undocumented status; healthy `compared` files carry
        // their mismatch kinds in the per-landmark buckets.
        taxonomy_for_diff_status(status)?;

        for bucket in ["matches", "heuristic_only", "ast_only"] {
            let entries = array_field(file, bucket)?;
            if !entries.is_empty() {
                observed_kinds.push(taxonomy_for_diff_bucket(bucket)?);
            }
        }
    }

    // The corpus reaches the advisory diff statuses and each comparison bucket.
    assert!(observed_statuses.iter().any(|status| status == "compared"));
    assert!(
        observed_statuses
            .iter()
            .any(|status| status == "parse_degraded")
    );
    assert!(
        observed_statuses
            .iter()
            .any(|status| status == "unsupported")
    );

    assert!(observed_kinds.contains(&"agree"));
    assert!(observed_kinds.contains(&"heuristic_only"));
    assert!(observed_kinds.contains(&"backend_only"));

    Ok(())
}

#[test]
fn ast_syntax_receipt_status_taxonomy_is_total_over_wire_statuses() -> TestResult {
    // Constructing every `SyntaxParseStatus` variant makes the documented
    // receipt-status taxonomy total over the wire vocabulary, including the
    // `parser_failed` status that cannot be triggered deterministically with a
    // healthy locked grammar. The mapping errors on any undocumented status.
    let all_statuses = [
        SyntaxParseStatus::Complete,
        SyntaxParseStatus::ParseDegraded,
        SyntaxParseStatus::ParserFailed,
        SyntaxParseStatus::SkippedGeneratedOrVendor,
        SyntaxParseStatus::SkippedTooLarge,
        SyntaxParseStatus::UnsupportedLanguage,
    ];
    for status in all_statuses {
        taxonomy_for_receipt_status(status.as_str())?;
    }

    assert_eq!(
        taxonomy_for_receipt_status(SyntaxParseStatus::ParserFailed.as_str())?,
        Some("parser_failed")
    );
    assert_eq!(
        taxonomy_for_receipt_status(SyntaxParseStatus::UnsupportedLanguage.as_str())?,
        Some("unsupported")
    );
    assert_eq!(
        taxonomy_for_receipt_status(SyntaxParseStatus::SkippedTooLarge.as_str())?,
        Some("skipped")
    );
    assert_eq!(
        taxonomy_for_receipt_status(SyntaxParseStatus::Complete.as_str())?,
        None
    );

    Ok(())
}

#[test]
fn ast_syntax_receipt_producers_emit_only_documented_statuses() -> TestResult {
    // Drive the real producer through each deterministically reachable status
    // and confirm the emitted wire string is part of the documented taxonomy.
    let huge_limit = SyntaxParseOptions {
        max_bytes: 4,
        skip_generated_vendor: false,
    };
    let cases = [
        ("src/lib.rs", "fn ok() {}\n", SyntaxParseOptions::default()),
        (
            "src/lib.rs",
            "fn ok() {}\nfn broken(",
            SyntaxParseOptions::default(),
        ),
        ("README.md", "# docs\n", SyntaxParseOptions::default()),
        (
            "vendor/crate/src/lib.rs",
            "fn ignored() {}\n",
            SyntaxParseOptions::default(),
        ),
        ("src/lib.rs", "fn main() {}\n", huge_limit),
    ];

    let mut observed = Vec::new();
    for (path, source, options) in cases {
        let receipt = parse_syntax_receipt(path, source, options);
        let value = receipt.to_value();
        let status = str_field(&value, "status")?.to_owned();
        // Errors on any undocumented status string.
        taxonomy_for_receipt_status(&status)?;
        observed.push(status);
    }

    for expected in [
        "complete",
        "parse_degraded",
        "unsupported_language",
        "skipped_generated_or_vendor",
        "skipped_too_large",
    ] {
        assert!(
            observed.iter().any(|status| status == expected),
            "expected to observe receipt status {expected}: {observed:?}"
        );
    }

    Ok(())
}
