//! Render audience-specific Markdown from cross-tool packet bundles.

use std::fmt::Write as _;

use anyhow::{Context, Result, bail};
use tokmd_types::{
    BUN_UB_PACKET_PRESETS, PacketPresetInput, PacketRenderBundle, TOKMD_PACKETS_SCHEMA,
    TokmdPacketsManifest,
};

use crate::packet_siblings::resolve_preset_input;

/// Human-readable title for a packet preset.
pub fn preset_title(preset: &str) -> &str {
    match preset {
        "bun-ub-handoff" => "Bun UB implementer handoff",
        "bun-ub-pr-body" => "Bun UB upstream PR body",
        "bun-ub-ledger-note" => "Bun UB ledger note",
        "bun-ub-review-map" => "Bun UB review map",
        "bun-ub-next-pick" => "Bun UB next pick",
        _ => "Packet preset",
    }
}

/// Validate manifest schema and preset name before rendering.
pub fn validate_manifest(manifest: &TokmdPacketsManifest, preset: &str) -> Result<()> {
    if !manifest.schema_matches() {
        bail!(
            "unsupported packet schema {:?}; expected {TOKMD_PACKETS_SCHEMA}",
            manifest.schema
        );
    }
    if !TokmdPacketsManifest::preset_is_known(preset) {
        bail!(
            "unknown packet preset {preset:?}; expected one of: {}",
            BUN_UB_PACKET_PRESETS.join(", ")
        );
    }
    Ok(())
}

/// Render Markdown for `preset` from a loaded bundle (manifest + sibling inputs).
///
/// Missing `preset_inputs` or sections produce explicit limitation notes — never
/// an empty or all-clear document. When manifest sections are absent, sibling
/// bundle files listed in `inputs_present` may supply partial sections.
pub fn render_packet_bundle_markdown(bundle: &PacketRenderBundle, preset: &str) -> Result<String> {
    validate_manifest(&bundle.manifest, preset)?;

    let mut out = String::new();
    writeln!(out, "# {}", preset_title(preset)).context("format title")?;
    writeln!(out).context("format blank line")?;
    writeln!(out, "Preset: `{preset}`").context("format preset id")?;
    writeln!(out).context("format blank line")?;

    let (resolved_input, resolution_notes) =
        resolve_preset_input(&bundle.manifest, &bundle.siblings, preset);

    match resolved_input {
        Some(input) => render_preset_input(&mut out, &input)?,
        None => {
            writeln!(out, "## Limitations").context("format limitations heading")?;
            writeln!(out).context("format blank line")?;
            writeln!(
                out,
                "- `preset_inputs` for `{preset}` is absent from the bundle and no sibling files supplied derivable sections."
            )
            .context("format absent preset_inputs limitation")?;
        }
    }

    render_resolution_notes(&mut out, &resolution_notes)?;
    render_sibling_load_notes(&mut out, &bundle.siblings)?;
    render_bundle_absent_inputs(&mut out, &bundle.manifest)?;
    render_non_claims(&mut out, &bundle.manifest)?;

    Ok(out)
}

/// Render Markdown for `preset` from a validated manifest only.
///
/// Prefer [`render_packet_bundle_markdown`] when sibling bundle files may be present.
pub fn render_packet_preset_markdown(
    manifest: &TokmdPacketsManifest,
    preset: &str,
) -> Result<String> {
    let bundle = PacketRenderBundle {
        manifest: manifest.clone(),
        siblings: tokmd_types::PacketSiblingInputs::default(),
    };
    render_packet_bundle_markdown(&bundle, preset)
}

fn render_preset_input(out: &mut String, input: &PacketPresetInput) -> Result<()> {
    if input.sections.is_empty() {
        writeln!(out, "## Limitations").context("format limitations heading")?;
        writeln!(out).context("format blank line")?;
        writeln!(
            out,
            "- `preset_inputs` is present but contains no `sections`; nothing can be rendered."
        )
        .context("format empty sections limitation")?;
    } else {
        for (section, body) in &input.sections {
            let heading = section_heading(section);
            writeln!(out, "## {heading}").context("format section heading")?;
            writeln!(out).context("format blank line")?;
            writeln!(out, "{body}").context("format section body")?;
            writeln!(out).context("format blank line")?;
        }
    }

    let mut limitations = Vec::new();
    limitations.extend(input.limitations.iter().cloned());
    limitations.extend(
        input
            .missing_sections
            .iter()
            .map(|section| format!("missing required section: {section}")),
    );
    if !limitations.is_empty() {
        writeln!(out, "## Limitations").context("format limitations heading")?;
        writeln!(out).context("format blank line")?;
        for item in limitations {
            writeln!(out, "- {item}").context("format limitation item")?;
        }
        writeln!(out).context("format blank line")?;
    }

    Ok(())
}

fn render_resolution_notes(out: &mut String, notes: &[String]) -> Result<()> {
    if notes.is_empty() {
        return Ok(());
    }
    writeln!(out, "## Bundle source inputs").context("format source heading")?;
    writeln!(out).context("format blank line")?;
    for note in notes {
        writeln!(out, "- {note}").context("format source note")?;
    }
    writeln!(out).context("format blank line")?;
    Ok(())
}

fn render_sibling_load_notes(
    out: &mut String,
    siblings: &tokmd_types::PacketSiblingInputs,
) -> Result<()> {
    if siblings.load_notes.is_empty() {
        return Ok(());
    }
    writeln!(out, "## Limitations").context("format limitations heading")?;
    writeln!(out).context("format blank line")?;
    for note in &siblings.load_notes {
        writeln!(out, "- {note}").context("format sibling load note")?;
    }
    writeln!(out).context("format blank line")?;
    Ok(())
}

fn render_bundle_absent_inputs(out: &mut String, manifest: &TokmdPacketsManifest) -> Result<()> {
    if manifest.inputs_absent.is_empty() {
        return Ok(());
    }
    writeln!(out, "## Bundle inputs absent").context("format absent inputs heading")?;
    writeln!(out).context("format blank line")?;
    for item in &manifest.inputs_absent {
        writeln!(out, "- `{item}` was not present in the bundle.").context("format absent item")?;
    }
    writeln!(out).context("format blank line")?;
    Ok(())
}

fn render_non_claims(out: &mut String, manifest: &TokmdPacketsManifest) -> Result<()> {
    if manifest.non_claims.is_empty() {
        return Ok(());
    }
    writeln!(out, "## Non-claims").context("format non-claims heading")?;
    writeln!(out).context("format blank line")?;
    for claim in &manifest.non_claims {
        writeln!(out, "- {claim}").context("format non-claim item")?;
    }
    Ok(())
}

fn section_heading(section: &str) -> String {
    section
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn sample_manifest(preset_inputs: BTreeMap<String, PacketPresetInput>) -> TokmdPacketsManifest {
        TokmdPacketsManifest {
            schema: TOKMD_PACKETS_SCHEMA.into(),
            producer: None,
            inputs_present: vec!["manual-candidates.json".into()],
            inputs_absent: vec!["comment-plan.json".into()],
            non_claims: vec![
                "Does not prove undefined behavior.".into(),
                "Does not prove memory safety.".into(),
            ],
            preset_inputs,
        }
    }

    #[test]
    fn absent_preset_inputs_emits_limitation_not_empty() {
        let manifest = sample_manifest(BTreeMap::new());
        let md = render_packet_preset_markdown(&manifest, "bun-ub-handoff").unwrap();
        assert!(md.contains("## Limitations"));
        assert!(md.contains("preset_inputs"));
        assert!(md.contains("## Non-claims"));
        assert!(md.contains("Does not prove undefined behavior."));
        assert!(!md.trim().is_empty());
    }

    #[test]
    fn renders_sections_and_missing_section_limitations() {
        let mut sections = BTreeMap::new();
        sections.insert("candidate_identity".into(), "seed-42".into());
        let manifest = sample_manifest(BTreeMap::from([(
            "bun-ub-handoff".into(),
            PacketPresetInput {
                sections,
                limitations: vec!["witness-plan.md absent".into()],
                missing_sections: vec!["test or witness target".into()],
            },
        )]));
        let md = render_packet_preset_markdown(&manifest, "bun-ub-handoff").unwrap();
        assert!(md.contains("## Candidate Identity"));
        assert!(md.contains("seed-42"));
        assert!(md.contains("missing required section: test or witness target"));
        assert!(md.contains("witness-plan.md absent"));
    }

    #[test]
    fn rejects_unknown_preset() {
        let manifest = sample_manifest(BTreeMap::new());
        let err = render_packet_preset_markdown(&manifest, "unknown-preset").unwrap_err();
        assert!(err.to_string().contains("unknown packet preset"));
    }

    #[test]
    fn rejects_mismatched_schema() {
        let manifest = TokmdPacketsManifest {
            schema: "tokmd-packets/v1".into(),
            producer: None,
            inputs_present: vec![],
            inputs_absent: vec![],
            non_claims: vec![],
            preset_inputs: BTreeMap::new(),
        };
        let err = render_packet_preset_markdown(&manifest, "bun-ub-handoff").unwrap_err();
        assert!(err.to_string().contains("unsupported packet schema"));
    }
}
