//! Consumer-side contract smoke for the ub-review evidence packet status
//! taxonomy.
//!
//! The producer side (`tokmd evidence-packet` / `tokmd packet generate`) is
//! covered by `evidence_packet_integration.rs`, which runs the binary and
//! asserts the manifests it writes. This file instead exercises the
//! **consumer** contract documented in `docs/ub-review-integration.md`: given a
//! `sensors/tokmd/manifest.json`, an ub-review lane must read two independent
//! axes (packet `status` and per-artifact advisory state) and follow a fixed
//! trust order. These tests pin each documented taxonomy row against both the
//! published JSON schema (`schemas/evidence-packet.schema.json`) and the public
//! `tokmd_types::EvidencePacketManifest` type, so a drift in the schema, the
//! type, or the canonical examples is caught.
//!
//! Beyond the status taxonomy, a second block pins trust-order step 3
//! (freshness / "cache by identity, not by presence"): a *valid* packet is only
//! reusable when its recorded `(schema, tokmd_version, base, head, paths,
//! preset)` identity matches the current run, an axis independent from
//! attachability that was previously unpinned by any consumer test.
//!
//! The synthetic rows above are pure parse/validate/assert: no binary, git, or
//! feature gate required. The `real_producer_bridge` module at the bottom of
//! this file additionally drives the real producer (`tokmd evidence-packet`)
//! and feeds its emitted manifest through the *same* `consume` + `is_attachable`
//! gate, so the hand-built skeletons cannot silently drift from what the binary
//! actually writes. That bridge is gated on the `analysis` feature and git
//! availability.

use std::error::Error;

use serde_json::{Value, json};
use tokmd_types::{EVIDENCE_PACKET_SCHEMA, EvidencePacketManifest, EvidencePacketStatus};

// Shared git/test helpers, declared at the crate root so the standard
// `tests/common/mod.rs` resolution works on every platform (a nested-module
// `#[path]` with `..` is not portable). Only the `analysis`-gated bridge below
// uses it, so gate the declaration to avoid an unused module warning.
#[cfg(feature = "analysis")]
mod common;

type TestResult = Result<(), Box<dyn Error>>;

const SCHEMA_JSON: &str = include_str!("../schemas/evidence-packet.schema.json");

/// Validate a manifest value against the published evidence-packet schema,
/// returning the collected violations on failure instead of panicking.
fn validate_against_schema(manifest: &Value) -> TestResult {
    let schema: Value = serde_json::from_str(SCHEMA_JSON)?;
    let validator = jsonschema::validator_for(&schema)?;
    if validator.is_valid(manifest) {
        return Ok(());
    }
    let errors: Vec<String> = validator
        .iter_errors(manifest)
        .map(|err| format!("{err} at {}", err.instance_path()))
        .collect();
    Err(format!("manifest failed schema validation:\n{}", errors.join("\n")).into())
}

/// Round-trip a manifest value through the published schema and the public
/// consumer type, returning the typed manifest.
fn consume(manifest: &Value) -> Result<EvidencePacketManifest, Box<dyn Error>> {
    validate_against_schema(manifest)?;
    Ok(serde_json::from_value(manifest.clone())?)
}

/// Encodes the documented consumer gate: read `status` + `errors` first; a
/// `failed` status or any non-empty `errors` means the evidence is invalid and
/// must not be attached as a valid-looking packet (see the status taxonomy and
/// "what to trust first" sections of `docs/ub-review-integration.md`).
fn is_attachable(manifest: &EvidencePacketManifest) -> bool {
    manifest.status != EvidencePacketStatus::Failed && manifest.errors.is_empty()
}

/// The documented cache-identity key for a *reusable* packet:
/// `(schema, tokmd_version, base, head, paths, preset)` (see the "Cache and
/// receipt guidance" section of `docs/ub-review-integration.md`: "Cache by
/// identity, not by presence"). `paths` is compared as the review *scope* set
/// (order-insensitive), matching the doc's "matches the PR's ... review scope".
#[derive(Debug, Clone, PartialEq, Eq)]
struct PacketIdentity {
    schema: String,
    tokmd_version: String,
    base: String,
    head: String,
    paths: Vec<String>,
    preset: String,
}

impl PacketIdentity {
    fn from_manifest(manifest: &EvidencePacketManifest) -> Self {
        let mut paths = manifest.paths.clone();
        paths.sort();
        Self {
            schema: manifest.schema.clone(),
            tokmd_version: manifest.tokmd_version.clone(),
            base: manifest.base.clone(),
            head: manifest.head.clone(),
            paths,
            preset: manifest.preset.clone(),
        }
    }
}

/// Trust-order step 3 (freshness / identity) from
/// `docs/ub-review-integration.md`: a schema-valid, error-free packet is only
/// *reusable* for the current run when its recorded identity matches the
/// requested `(schema, tokmd_version, base, head, paths, preset)`. This is an
/// axis independent from `is_attachable` (which only gates packet *validity*):
/// a valid packet produced for a different diff window or scope is still
/// **stale** and must be regenerated before it is trusted or cached as a pass.
fn is_fresh_for(manifest: &EvidencePacketManifest, expected: &PacketIdentity) -> bool {
    &PacketIdentity::from_manifest(manifest) == expected
}

fn base_artifacts(with_syntax: bool) -> Value {
    let mut artifacts = serde_json::Map::new();
    artifacts.insert("analyze_md".to_string(), json!("sensors/tokmd/analyze.md"));
    artifacts.insert(
        "analyze_json".to_string(),
        json!("sensors/tokmd/analyze.json"),
    );
    artifacts.insert("context_md".to_string(), json!("sensors/tokmd/context.md"));
    if with_syntax {
        artifacts.insert(
            "syntax_json".to_string(),
            json!("sensors/tokmd/syntax.json"),
        );
    }
    Value::Object(artifacts)
}

fn manifest_skeleton(status: &str, artifacts: Value, warnings: Value, errors: Value) -> Value {
    json!({
        "schema": EVIDENCE_PACKET_SCHEMA,
        "tokmd_version": "1.14.0",
        "preset": "bun-ub",
        "base": "origin/main",
        "head": "HEAD",
        "paths": ["src/runtime/api/MarkdownObject.rs"],
        "status": status,
        "artifacts": artifacts,
        "warnings": warnings,
        "errors": errors,
        "non_claims": [
            "bun-ub packages review evidence; it does not prove UB exists or is absent"
        ],
        "reproduce": [
            "tokmd packet generate --base origin/main --head HEAD src/runtime/api/MarkdownObject.rs"
        ],
    })
}

#[test]
fn complete_packet_is_attachable_with_skipped_optional_signal() -> TestResult {
    // Row 1: required artifacts exist, errors empty -> attach and review.
    // No `syntax_json` is the "skipped" advisory state (optional signal not
    // requested), which must keep the packet `complete`, not a failure.
    let manifest = manifest_skeleton("complete", base_artifacts(false), json!([]), json!([]));
    let packet = consume(&manifest)?;

    assert_eq!(packet.status, EvidencePacketStatus::Complete);
    assert!(
        packet.errors.is_empty(),
        "complete packet must carry no errors"
    );
    assert!(
        !packet.artifacts.analyze_md.is_empty()
            && !packet.artifacts.analyze_json.is_empty()
            && !packet.artifacts.context_md.is_empty(),
        "required artifact references must be present"
    );
    assert!(
        packet.artifacts.syntax_json.is_none(),
        "skipped optional signal: syntax_json absent"
    );
    assert!(
        is_attachable(&packet),
        "complete packet is valid review evidence"
    );
    Ok(())
}

#[test]
fn partial_packet_surfaces_advisory_missing_as_named_limit() -> TestResult {
    // Row 2 + advisory-missing: an optional signal was requested for this run
    // but is absent, so the packet degrades to `partial` with a named warning.
    // It stays attachable (exit 0); the limit is named, not a failure.
    let warnings = json!(["optional artifact syntax_json missing for this run"]);
    let manifest = manifest_skeleton("partial", base_artifacts(true), warnings, json!([]));
    let packet = consume(&manifest)?;

    assert_eq!(packet.status, EvidencePacketStatus::Partial);
    assert!(
        packet.errors.is_empty(),
        "partial is a bounded limit, not invalid evidence"
    );
    assert!(
        !packet.warnings.is_empty(),
        "partial status must be explained by at least one named warning"
    );
    assert!(
        packet
            .warnings
            .iter()
            .any(|w| w.contains("syntax_json missing")),
        "advisory-missing must name the absent optional artifact: {:?}",
        packet.warnings
    );
    assert!(
        is_attachable(&packet),
        "partial packet is still attachable evidence"
    );
    Ok(())
}

#[test]
fn failed_packet_is_rejected_not_attached() -> TestResult {
    // Row 3: a required artifact is missing (or refs/parse failed). The manifest
    // is still written (schema-valid) for inspection, but the consumer must NOT
    // attach a valid-looking packet: status `failed` and non-empty `errors`.
    let errors = json!(["required artifact analyze_md missing: sensors/tokmd/analyze.md"]);
    let manifest = manifest_skeleton("failed", base_artifacts(false), json!([]), errors);
    let packet = consume(&manifest)?;

    assert_eq!(packet.status, EvidencePacketStatus::Failed);
    assert!(
        !packet.errors.is_empty(),
        "failed status must record at least one error for the consumer"
    );
    assert!(
        !is_attachable(&packet),
        "failed packet must be rejected, regenerated, or omitted"
    );
    Ok(())
}

#[test]
fn trust_order_rejects_any_nonempty_errors_regardless_of_status() -> TestResult {
    // "Read the manifest top-down ... stop and reject early if an upstream field
    // invalidates the packet." Any non-empty `errors` invalidates the evidence
    // even if a (malformed) producer left `status` more optimistic.
    let errors = json!(["analyze.json could not be parsed"]);
    let manifest = manifest_skeleton("complete", base_artifacts(false), json!([]), errors);
    let packet = consume(&manifest)?;

    assert!(
        !is_attachable(&packet),
        "non-empty errors must fail the validity gate before status is trusted"
    );
    Ok(())
}

#[test]
fn consumer_confirms_schema_identity_before_interpreting_fields() -> TestResult {
    // Trust-order step 2: confirm `schema` + `tokmd_version` before reading the
    // rest. The published schema pins `schema` to the v1 const; a manifest with
    // a foreign schema id must not validate as a v1 evidence packet.
    let valid = manifest_skeleton("complete", base_artifacts(false), json!([]), json!([]));
    let packet = consume(&valid)?;
    assert_eq!(packet.schema, EVIDENCE_PACKET_SCHEMA);
    assert!(
        !packet.tokmd_version.is_empty(),
        "tokmd_version must be present"
    );

    let mut foreign = valid;
    let foreign_obj = foreign
        .as_object_mut()
        .ok_or("manifest skeleton must be a JSON object")?;
    foreign_obj.insert("schema".to_string(), json!("tokmd.evidence-packet/v2"));
    assert!(
        validate_against_schema(&foreign).is_err(),
        "a non-v1 schema id must be rejected by the v1 schema gate"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Trust-order step 3: freshness / cache-by-identity (independent from validity)
//
// The synthetic rows above pin packet *validity* (`status` + `errors`). These
// rows pin the separate, currently-unpinned consumer axis: a *valid* packet is
// only reusable when its recorded identity matches the current run's window and
// scope. Any mismatch is documented as "stale" evidence (trust-order step 3)
// and "Cache by identity, not by presence" guidance, and must be regenerated
// rather than reused as a pass. These rows never touch the binary or git, so
// they do not overlap the `real_producer_bridge` status-taxonomy coverage.
// ---------------------------------------------------------------------------

/// The review window the skeleton fixtures describe, i.e. the identity a
/// consumer would compute from the current PR before deciding whether a cached
/// `sensors/tokmd/` packet is reusable.
fn current_window() -> PacketIdentity {
    PacketIdentity {
        schema: EVIDENCE_PACKET_SCHEMA.to_string(),
        tokmd_version: "1.14.0".to_string(),
        base: "origin/main".to_string(),
        head: "HEAD".to_string(),
        paths: vec!["src/runtime/api/MarkdownObject.rs".to_string()],
        preset: "bun-ub".to_string(),
    }
}

fn complete_manifest() -> Value {
    manifest_skeleton("complete", base_artifacts(false), json!([]), json!([]))
}

/// Override a single top-level manifest field, keeping the value schema-valid so
/// staleness (not schema failure) is what the freshness gate observes.
fn with_field(mut manifest: Value, key: &str, value: Value) -> Result<Value, Box<dyn Error>> {
    manifest
        .as_object_mut()
        .ok_or("manifest skeleton must be a JSON object")?
        .insert(key.to_string(), value);
    Ok(manifest)
}

#[test]
fn fresh_matching_window_packet_is_attachable_and_reusable() -> TestResult {
    // Baseline: a complete packet whose identity matches the current run is both
    // valid (attachable) and fresh (reusable without regeneration).
    let packet = consume(&complete_manifest())?;
    assert!(is_attachable(&packet), "complete packet is valid evidence");
    assert!(
        is_fresh_for(&packet, &current_window()),
        "a packet whose (schema, version, base, head, paths, preset) matches the current \
         run is reusable"
    );
    Ok(())
}

#[test]
fn stale_head_mismatch_is_attachable_but_not_reusable() -> TestResult {
    // A valid packet from a *previous* head is still schema-valid and error-free
    // (attachable), but trust-order step 3 marks it stale for the current head:
    // the two axes are independent, so a green status must not mask a stale
    // window.
    let manifest = with_field(complete_manifest(), "head", json!("deadbeefcafe"))?;
    let packet = consume(&manifest)?;

    assert!(
        is_attachable(&packet),
        "status/errors validity is unaffected by a head mismatch"
    );
    assert!(
        !is_fresh_for(&packet, &current_window()),
        "a head mismatch is stale evidence and must be regenerated, not reused"
    );
    Ok(())
}

#[test]
fn stale_base_mismatch_is_not_reusable() -> TestResult {
    let manifest = with_field(complete_manifest(), "base", json!("origin/release-1.13"))?;
    let packet = consume(&manifest)?;
    assert!(
        is_attachable(&packet),
        "base drift does not invalidate refs"
    );
    assert!(
        !is_fresh_for(&packet, &current_window()),
        "a base mismatch changes the diff window and is stale"
    );
    Ok(())
}

#[test]
fn stale_scope_drift_in_paths_is_not_reusable() -> TestResult {
    // Same base/head but a different review scope: the packet analysed other
    // files, so it cannot be reused for this scope.
    let manifest = with_field(
        complete_manifest(),
        "paths",
        json!(["src/runtime/api/OtherObject.rs"]),
    )?;
    let packet = consume(&manifest)?;
    assert!(is_attachable(&packet));
    assert!(
        !is_fresh_for(&packet, &current_window()),
        "review-scope drift is stale: the packet covers different paths"
    );
    Ok(())
}

#[test]
fn path_reordering_within_same_scope_stays_fresh() -> TestResult {
    // Cache identity treats `paths` as the review *scope set*: the same files in
    // a different serialization order are the same scope and stay reusable, so a
    // producer path-ordering change does not spuriously invalidate a cache.
    let expected = PacketIdentity {
        paths: vec![
            "src/a.rs".to_string(),
            "src/b.rs".to_string(),
            "src/c.rs".to_string(),
        ],
        ..current_window()
    };
    let manifest = with_field(
        complete_manifest(),
        "paths",
        json!(["src/c.rs", "src/a.rs", "src/b.rs"]),
    )?;
    let packet = consume(&manifest)?;
    assert!(
        is_fresh_for(&packet, &expected),
        "the same scope set in a different order is not a staleness signal"
    );
    Ok(())
}

#[test]
fn stale_preset_mismatch_is_not_reusable() -> TestResult {
    // The cache identity key includes `preset`: a packet built with a different
    // preset produced different signals and must not be reused for this run.
    let manifest = with_field(complete_manifest(), "preset", json!("risk"))?;
    let packet = consume(&manifest)?;
    assert!(is_attachable(&packet));
    assert!(
        !is_fresh_for(&packet, &current_window()),
        "a preset mismatch is stale: the cache key includes preset"
    );
    Ok(())
}

#[test]
fn stale_tokmd_version_mismatch_is_not_reusable() -> TestResult {
    // A packet from a different tokmd version may encode different behaviour; the
    // documented cache key pins `tokmd_version`, so a version drift is stale.
    let manifest = with_field(complete_manifest(), "tokmd_version", json!("1.13.0"))?;
    let packet = consume(&manifest)?;
    assert!(is_attachable(&packet));
    assert!(
        !is_fresh_for(&packet, &current_window()),
        "a tokmd_version mismatch is stale: the cache key includes tokmd_version"
    );
    Ok(())
}

/// Realistic-input bridge: run the real producer (`tokmd evidence-packet`) and
/// feed the manifest it writes through the same consumer gate the synthetic rows
/// exercise (`consume` + `is_attachable`). This guards the documented trust
/// order and ADR-0015 ("consume `partial`, reject `failed`") against real binary
/// output rather than only hand-built fixtures.
///
/// Claim boundary: this asserts the producer's emitted `manifest.json` round
/// trips through the published schema and the public `EvidencePacketManifest`
/// type and lands on the documented attachability decision. It does not assert
/// analyze/syntax content correctness, and it does not prove anything about UB.
#[cfg(feature = "analysis")]
mod real_producer_bridge {
    use std::path::Path;

    use assert_cmd::Command;
    use serde_json::{Value, json};
    use tempfile::tempdir;
    use tokmd_types::{EvidencePacketManifest, EvidencePacketStatus};

    use super::{TestResult, consume, is_attachable};
    use crate::common;

    type SetupResult<T> = Result<T, Box<dyn std::error::Error>>;

    const SCOPE_PATH: &str = "src/runtime/api/MarkdownObject.rs";

    /// Write the canonical complete `analyze.json` the producer validates.
    fn write_analyze_json(sensor_dir: &Path) -> SetupResult<()> {
        std::fs::write(
            sensor_dir.join("analyze.json"),
            json!({
                "status": "complete",
                "warnings": [],
                "args": { "preset": "bun-ub" },
                "source": { "inputs": [SCOPE_PATH] }
            })
            .to_string(),
        )?;
        Ok(())
    }

    /// Initialise a git repo whose review scope changed between `main` and HEAD,
    /// matching the diff window the producer resolves. Fallible so the bridge
    /// stays panic-free.
    fn init_repo_with_scope() -> SetupResult<tempfile::TempDir> {
        let dir = tempdir()?;
        if !common::init_git_repo(dir.path()) {
            return Err("git init failed".into());
        }
        let scope_dir = dir.path().join("src").join("runtime").join("api");
        std::fs::create_dir_all(&scope_dir)?;
        std::fs::write(scope_dir.join("MarkdownObject.rs"), "pub fn old() {}\n")?;
        if !common::git_add_commit(dir.path(), "initial") {
            return Err("initial commit failed".into());
        }
        std::fs::write(
            scope_dir.join("MarkdownObject.rs"),
            "pub fn old() {}\npub fn new_boundary() {}\n",
        )?;
        if !common::git_add_commit(dir.path(), "change api") {
            return Err("change commit failed".into());
        }
        Ok(dir)
    }

    /// Write the three required sensor artifacts the producer indexes.
    fn write_required_artifacts(root: &Path) -> SetupResult<()> {
        let sensor_dir = root.join("sensors").join("tokmd");
        std::fs::create_dir_all(&sensor_dir)?;
        std::fs::write(sensor_dir.join("analyze.md"), "# Bun UB analyze\n")?;
        write_analyze_json(&sensor_dir)?;
        std::fs::write(sensor_dir.join("context.md"), "# Context\n")?;
        Ok(())
    }

    /// Run the producer and return its emitted `manifest.json` value.
    fn generate_manifest(root: &Path, extra_args: &[&str]) -> SetupResult<Value> {
        let mut args = vec!["evidence-packet", "--base", "main", "--head", "HEAD"];
        args.extend_from_slice(extra_args);
        args.push(SCOPE_PATH);
        // The producer exits non-zero for `failed` packets but still writes the
        // manifest for inspection, so do not assert on the exit status here; the
        // consumer gate is what this bridge validates.
        let _ = Command::new(env!("CARGO_BIN_EXE_tokmd"))
            .current_dir(root)
            .args(&args)
            .output()?;
        let manifest_path = root.join("sensors").join("tokmd").join("manifest.json");
        let raw = std::fs::read_to_string(manifest_path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    #[test]
    fn real_complete_packet_passes_consumer_gate() -> TestResult {
        if !common::git_available() {
            return Ok(());
        }
        let dir = init_repo_with_scope()?;
        write_required_artifacts(dir.path())?;

        let manifest = generate_manifest(dir.path(), &[])?;
        // Real producer output must round-trip through the published schema and
        // the public consumer type unchanged.
        let packet: EvidencePacketManifest = consume(&manifest)?;

        assert_eq!(packet.status, EvidencePacketStatus::Complete);
        assert!(
            packet.errors.is_empty(),
            "complete packet carries no errors"
        );
        assert!(
            is_attachable(&packet),
            "a real complete packet is valid review evidence"
        );
        Ok(())
    }

    #[test]
    fn real_partial_packet_with_advisory_missing_syntax_is_attachable() -> TestResult {
        if !common::git_available() {
            return Ok(());
        }
        let dir = init_repo_with_scope()?;
        write_required_artifacts(dir.path())?;

        // Request an optional syntax artifact that was never written: the
        // documented "advisory-missing" state. Per ADR-0015 the packet degrades
        // to `partial` with a named warning and stays attachable.
        let manifest =
            generate_manifest(dir.path(), &["--syntax-json", "sensors/tokmd/syntax.json"])?;
        let packet: EvidencePacketManifest = consume(&manifest)?;

        assert_eq!(packet.status, EvidencePacketStatus::Partial);
        assert!(
            packet.errors.is_empty(),
            "advisory-missing syntax is a bounded limit, not invalid evidence"
        );
        assert!(
            packet
                .warnings
                .iter()
                .any(|w| w.contains("syntax_json") && w.contains("missing")),
            "advisory-missing must name the absent optional artifact: {:?}",
            packet.warnings
        );
        assert!(
            is_attachable(&packet),
            "ADR-0015: ub-review consumes partial packets"
        );
        Ok(())
    }

    #[test]
    fn real_failed_packet_is_rejected_by_consumer_gate() -> TestResult {
        if !common::git_available() {
            return Ok(());
        }
        let dir = init_repo_with_scope()?;
        // Omit the required analyze.md / context.md so the producer marks the
        // packet `failed` and records errors, while still writing the manifest.
        let sensor_dir = dir.path().join("sensors").join("tokmd");
        std::fs::create_dir_all(&sensor_dir)?;
        write_analyze_json(&sensor_dir)?;

        let manifest = generate_manifest(dir.path(), &[])?;
        // A failed manifest is still schema-valid and typed for inspection.
        let packet: EvidencePacketManifest = consume(&manifest)?;

        assert_eq!(packet.status, EvidencePacketStatus::Failed);
        assert!(
            !packet.errors.is_empty(),
            "failed status must record at least one error for the consumer"
        );
        assert!(
            !is_attachable(&packet),
            "a real failed packet must be rejected, not attached"
        );
        Ok(())
    }
}
