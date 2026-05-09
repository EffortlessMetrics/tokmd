//! Analysis receipt rendering.
//!
//! This module owns analysis-specific formatting under the durable
//! `tokmd-format` capability crate. It supports Markdown, JSON, JSON-LD, XML,
//! SVG, Mermaid, HTML, and optional fun outputs.
//!
//! ## Effort rendering
//!
//! Effort sections are rendered in two tiers:
//!
//! 1. `receipt.effort` the preferred path for the newer effort-estimation
//!    receipt surface. This can render size basis, confidence, drivers,
//!    assumptions, and optional delta data.
//! 2. `derived.cocomo` a legacy fallback used when the richer `effort`
//!    section is absent but classic derived COCOMO data is present.
//!
//! The formatter intentionally renders whatever the receipt contains without
//! inferring missing estimate data. If the upstream effort builder is still
//! scaffold-only, the formatter preserves that truth rather than making the
//! estimate look more complete than it is.
//!
//! ## What belongs here
//! * Analysis receipt rendering to various formats
//! * Format-specific transformations
//! * Fun output integration (OBJ, MIDI when enabled)
//!
//! ## What does NOT belong here
//! * Analysis computation (use tokmd-analysis)
//! * CLI argument parsing
//! * Analysis computation (use tokmd-analysis)

use anyhow::Result;
use tokmd_analysis_types::AnalysisReceipt;
use tokmd_types::AnalysisFormat;

mod fun_outputs;
pub mod html;
mod jsonld;
mod markdown;
mod mermaid;
mod svg;
mod tree;
mod xml;

pub enum RenderedOutput {
    Text(String),
    Binary(Vec<u8>),
}

pub fn render(receipt: &AnalysisReceipt, format: AnalysisFormat) -> Result<RenderedOutput> {
    match format {
        AnalysisFormat::Md => Ok(RenderedOutput::Text(render_md(receipt))),
        AnalysisFormat::Json => Ok(RenderedOutput::Text(serde_json::to_string_pretty(receipt)?)),
        AnalysisFormat::Jsonld => Ok(RenderedOutput::Text(jsonld::render(receipt))),
        AnalysisFormat::Xml => Ok(RenderedOutput::Text(xml::render(receipt))),
        AnalysisFormat::Svg => Ok(RenderedOutput::Text(svg::render(receipt))),
        AnalysisFormat::Mermaid => Ok(RenderedOutput::Text(mermaid::render(receipt))),
        AnalysisFormat::Obj => Ok(RenderedOutput::Text(fun_outputs::render_obj(receipt)?)),
        AnalysisFormat::Midi => Ok(RenderedOutput::Binary(fun_outputs::render_midi(receipt)?)),
        AnalysisFormat::Tree => Ok(RenderedOutput::Text(tree::render(receipt))),
        AnalysisFormat::Html => Ok(RenderedOutput::Text(render_html(receipt))),
    }
}

fn render_md(receipt: &AnalysisReceipt) -> String {
    markdown::render_md(receipt)
}

fn render_html(receipt: &AnalysisReceipt) -> String {
    html::render(receipt)
}

#[cfg(test)]
mod tests;
