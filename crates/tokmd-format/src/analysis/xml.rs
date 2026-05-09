//! XML rendering for analysis receipts.
//!
//! This module owns the compact XML projection used by `AnalysisFormat::Xml`.

use std::fmt::Write;

use tokmd_analysis_types::AnalysisReceipt;

pub(super) fn render(receipt: &AnalysisReceipt) -> String {
    let totals = receipt.derived.as_ref().map(|d| &d.totals);
    let mut out = String::new();
    out.push_str("<analysis>");
    if let Some(totals) = totals {
        let _ = write!(
            out,
            "<totals files=\"{}\" code=\"{}\" comments=\"{}\" blanks=\"{}\" lines=\"{}\" bytes=\"{}\" tokens=\"{}\"/>",
            totals.files,
            totals.code,
            totals.comments,
            totals.blanks,
            totals.lines,
            totals.bytes,
            totals.tokens
        );
    }
    out.push_str("</analysis>");
    out
}
