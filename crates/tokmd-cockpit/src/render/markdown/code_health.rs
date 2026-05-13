use std::fmt::Write;

use crate::CockpitReceipt;

pub(super) fn append(s: &mut String, receipt: &CockpitReceipt) {
    let _ = writeln!(s, "### Code Health");
    let _ = writeln!(s);
    let _ = writeln!(s, "- **Score**: {}/100", receipt.code_health.score);
    let _ = writeln!(s, "- **Grade**: {}", receipt.code_health.grade);
    let _ = writeln!(
        s,
        "- **Large files touched**: {}",
        receipt.code_health.large_files_touched
    );
    let _ = writeln!(
        s,
        "- **Average file size**: {}",
        receipt.code_health.avg_file_size
    );
    let _ = writeln!(
        s,
        "- **Complexity indicator**: {:?}",
        receipt.code_health.complexity_indicator
    );
    if !receipt.code_health.warnings.is_empty() {
        let _ = writeln!(s, "- **Warnings**:");
        for warning in &receipt.code_health.warnings {
            let _ = writeln!(s, "  - {}: {}", warning.path, warning.message);
        }
    }
    let _ = writeln!(s);
}
