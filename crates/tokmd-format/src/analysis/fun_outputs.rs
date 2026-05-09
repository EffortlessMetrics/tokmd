//! OBJ and MIDI analysis-format adapters.
//!
//! This module owns the `AnalysisFormat::Obj` and `AnalysisFormat::Midi`
//! projections from analysis receipts into the lower-level `fun` renderers.

use anyhow::Result;
use tokmd_analysis_types::AnalysisReceipt;

#[cfg(feature = "fun")]
fn render_obj_fun(receipt: &AnalysisReceipt) -> Result<String> {
    if let Some(derived) = &receipt.derived {
        let buildings: Vec<crate::fun::ObjBuilding> = derived
            .top
            .largest_lines
            .iter()
            .enumerate()
            .map(|(idx, row)| {
                let x = (idx % 5) as f32 * 2.0;
                let y = (idx / 5) as f32 * 2.0;
                let h = (row.lines as f32 / 10.0).max(0.5);
                crate::fun::ObjBuilding {
                    name: row.path.clone(),
                    x,
                    y,
                    w: 1.5,
                    d: 1.5,
                    h,
                }
            })
            .collect();
        return Ok(crate::fun::render_obj(&buildings));
    }
    Ok("# tokmd code city\n".to_string())
}

#[cfg(feature = "fun")]
fn render_midi_fun(receipt: &AnalysisReceipt) -> Result<Vec<u8>> {
    let mut notes = Vec::new();
    if let Some(derived) = &receipt.derived {
        for (idx, row) in derived.top.largest_lines.iter().enumerate() {
            let key = 60u8 + (row.depth as u8 % 12);
            let velocity = (40 + (row.lines.min(127) as u8 / 2)).min(120);
            let start = (idx as u32) * 240;
            notes.push(crate::fun::MidiNote {
                key,
                velocity,
                start,
                duration: 180,
                channel: 0,
            });
        }
    }
    crate::fun::render_midi(&notes, 120)
}

#[cfg(not(feature = "fun"))]
fn render_obj_disabled(_receipt: &AnalysisReceipt) -> Result<String> {
    anyhow::bail!(
        "OBJ format requires the `fun` feature: tokmd-format = {{ version = \"1.9\", features = [\"fun\"] }}"
    )
}

#[cfg(not(feature = "fun"))]
fn render_midi_disabled(_receipt: &AnalysisReceipt) -> Result<Vec<u8>> {
    anyhow::bail!(
        "MIDI format requires the `fun` feature: tokmd-format = {{ version = \"1.9\", features = [\"fun\"] }}"
    )
}

pub(super) fn render_obj(receipt: &AnalysisReceipt) -> Result<String> {
    #[cfg(feature = "fun")]
    {
        render_obj_fun(receipt)
    }
    #[cfg(not(feature = "fun"))]
    {
        render_obj_disabled(receipt)
    }
}

pub(super) fn render_midi(receipt: &AnalysisReceipt) -> Result<Vec<u8>> {
    #[cfg(feature = "fun")]
    {
        render_midi_fun(receipt)
    }
    #[cfg(not(feature = "fun"))]
    {
        render_midi_disabled(receipt)
    }
}
