//! Render audience-specific Markdown from cross-tool packet bundles.

use std::fs;
use std::io::Write as _;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::Value;
use tokmd_format::render_packet_bundle_markdown;
use tokmd_types::{
    CARDS_FILE, CardsFile, MANUAL_CANDIDATES_FILE, ManualCandidatesFile, PacketRenderBundle,
    PacketSiblingInputs, TokmdPacketsManifest,
};

use crate::cli;

const MANIFEST_NAME: &str = "tokmd-packets.json";
const TOKMD_PACKETS_SCHEMA_JSON: &str = include_str!("../../schemas/tokmd-packets.schema.json");

pub(crate) fn handle(args: cli::RenderArgs) -> Result<()> {
    let bundle = load_bundle(&args.from_packets)?;
    let preset = args.preset.as_str();
    let markdown = render_packet_bundle_markdown(&bundle, preset)
        .with_context(|| format!("failed to render preset {preset}"))?;

    if let Some(output) = args.output {
        if let Some(parent) = output.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&output, &markdown)
            .with_context(|| format!("failed to write {}", output.display()))?;
    } else {
        let mut stdout = std::io::stdout().lock();
        stdout
            .write_all(markdown.as_bytes())
            .context("failed to write rendered packet to stdout")?;
        if !markdown.ends_with('\n') {
            stdout
                .write_all(b"\n")
                .context("failed to write trailing newline")?;
        }
    }

    Ok(())
}

fn load_bundle(bundle_dir: &Path) -> Result<PacketRenderBundle> {
    let manifest = load_manifest(bundle_dir)?;
    let siblings = load_sibling_inputs(bundle_dir, &manifest)?;
    Ok(PacketRenderBundle { manifest, siblings })
}

fn load_manifest(bundle_dir: &Path) -> Result<TokmdPacketsManifest> {
    let manifest_path = bundle_dir.join(MANIFEST_NAME);
    let raw = fs::read_to_string(&manifest_path).with_context(|| {
        format!(
            "failed to read packet manifest {}; pass a bundle directory with {MANIFEST_NAME}",
            manifest_path.display()
        )
    })?;
    let value: Value = serde_json::from_str(&raw).with_context(|| {
        format!(
            "failed to parse packet manifest {}",
            manifest_path.display()
        )
    })?;
    validate_manifest_json(&value, &manifest_path)?;
    let manifest: TokmdPacketsManifest = serde_json::from_value(value).with_context(|| {
        format!(
            "failed to decode packet manifest {}",
            manifest_path.display()
        )
    })?;
    if !manifest.schema_matches() {
        bail!(
            "unsupported packet schema {:?} in {}; expected tokmd.packets/v1",
            manifest.schema,
            manifest_path.display()
        );
    }
    Ok(manifest)
}

fn load_sibling_inputs(
    bundle_dir: &Path,
    manifest: &TokmdPacketsManifest,
) -> Result<PacketSiblingInputs> {
    let mut siblings = PacketSiblingInputs::default();

    if manifest
        .inputs_present
        .iter()
        .any(|name| name == MANUAL_CANDIDATES_FILE)
    {
        load_manual_candidates(bundle_dir, &mut siblings)?;
    }
    if manifest
        .inputs_present
        .iter()
        .any(|name| name == CARDS_FILE)
    {
        load_cards(bundle_dir, &mut siblings)?;
    }

    Ok(siblings)
}

fn load_manual_candidates(bundle_dir: &Path, siblings: &mut PacketSiblingInputs) -> Result<()> {
    let path = bundle_dir.join(MANUAL_CANDIDATES_FILE);
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(err) => {
            siblings.load_notes.push(format!(
                "`{MANUAL_CANDIDATES_FILE}` is listed in `inputs_present` but could not be read from the bundle ({err})"
            ));
            return Ok(());
        }
    };
    let parsed: ManualCandidatesFile = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse sibling bundle file {}", path.display()))?;
    if !parsed.schema_matches() {
        siblings.load_notes.push(format!(
            "unsupported schema {:?} in `{MANUAL_CANDIDATES_FILE}`; expected manual-candidates/v1",
            parsed.schema_version
        ));
    }
    siblings.manual_candidates = Some(parsed);
    Ok(())
}

fn load_cards(bundle_dir: &Path, siblings: &mut PacketSiblingInputs) -> Result<()> {
    let path = bundle_dir.join(CARDS_FILE);
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(err) => {
            siblings.load_notes.push(format!(
                "`{CARDS_FILE}` is listed in `inputs_present` but could not be read from the bundle ({err})"
            ));
            return Ok(());
        }
    };
    let parsed: CardsFile = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse sibling bundle file {}", path.display()))?;
    siblings.cards = Some(parsed);
    Ok(())
}

fn validate_manifest_json(document: &Value, manifest_path: &Path) -> Result<()> {
    let schema: Value = serde_json::from_str(TOKMD_PACKETS_SCHEMA_JSON)
        .context("failed to parse embedded tokmd-packets schema")?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|err| anyhow::anyhow!("failed to compile tokmd-packets schema: {err}"))?;
    let errors: Vec<String> = validator
        .iter_errors(document)
        .map(|err| format!("{err} at {}", err.instance_path()))
        .collect();
    if errors.is_empty() {
        return Ok(());
    }
    bail!(
        "packet manifest {} failed schema validation:\n{}",
        manifest_path.display(),
        errors.join("\n")
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use tokmd_types::{PacketPresetInput, TOKMD_PACKETS_SCHEMA};

    #[test]
    fn load_manifest_rejects_schema_validation_failure() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(MANIFEST_NAME),
            r#"{"schema":"tokmd.packets/v0","non_claims":[]}"#,
        )
        .unwrap();
        let err = load_manifest(dir.path()).unwrap_err();
        assert!(
            err.to_string().contains("failed schema validation"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn load_manifest_reads_bundle_file() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = TokmdPacketsManifest {
            schema: TOKMD_PACKETS_SCHEMA.into(),
            producer: None,
            inputs_present: vec![],
            inputs_absent: vec![],
            non_claims: vec!["Does not prove UB.".into()],
            preset_inputs: BTreeMap::from([(
                "bun-ub-handoff".into(),
                PacketPresetInput {
                    sections: BTreeMap::from([("candidate_identity".into(), "seed-1".into())]),
                    limitations: vec![],
                    missing_sections: vec![],
                },
            )]),
        };
        fs::write(
            dir.path().join(MANIFEST_NAME),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        let loaded = load_manifest(dir.path()).unwrap();
        assert_eq!(loaded.schema, TOKMD_PACKETS_SCHEMA);
    }

    #[test]
    fn load_bundle_ingests_manual_candidates_when_present() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = TokmdPacketsManifest {
            schema: TOKMD_PACKETS_SCHEMA.into(),
            producer: None,
            inputs_present: vec![MANUAL_CANDIDATES_FILE.into()],
            inputs_absent: vec![],
            non_claims: vec![],
            preset_inputs: BTreeMap::new(),
        };
        fs::write(
            dir.path().join(MANIFEST_NAME),
            serde_json::to_string(&manifest).unwrap(),
        )
        .unwrap();
        fs::write(
            dir.path().join(MANUAL_CANDIDATES_FILE),
            r#"{"schema_version":"manual-candidates/v1","candidates":[{"id":"seed-9","title":"fixture"}]}"#,
        )
        .unwrap();

        let bundle = load_bundle(dir.path()).unwrap();
        assert!(bundle.siblings.manual_candidates.is_some());
    }
}
