//! Embedded report JSON rendering for analysis HTML reports.

use tokmd_analysis_types::AnalysisReceipt;

pub(super) fn build_report_json(receipt: &AnalysisReceipt) -> String {
    let mut files = Vec::new();

    if let Some(derived) = &receipt.derived {
        for row in &derived.top.largest_lines {
            files.push(serde_json::json!({
                "path": row.path,
                "module": row.module,
                "lang": row.lang,
                "code": row.code,
                "lines": row.lines,
                "tokens": row.tokens,
            }));
        }
    }

    // Escape < and > to prevent </script> breakout XSS attacks.
    // JSON remains valid because \u003c and \u003e are valid JSON string escapes.
    serde_json::json!({ "files": files })
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
}
