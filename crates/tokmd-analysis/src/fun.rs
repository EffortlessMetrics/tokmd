use tokmd_analysis_types::{DerivedReport, EcoLabel, FunReport};

use crate::util::round_f64;

pub(crate) fn build_fun_report(derived: &DerivedReport) -> FunReport {
    let bytes = derived.totals.bytes as u64;
    let mb = bytes as f64 / (1024.0 * 1024.0);
    let (label, score) = if mb <= 1.0 {
        ("A", 95.0)
    } else if mb <= 10.0 {
        ("B", 80.0)
    } else if mb <= 50.0 {
        ("C", 65.0)
    } else if mb <= 200.0 {
        ("D", 45.0)
    } else {
        ("E", 30.0)
    };
    FunReport {
        eco_label: Some(EcoLabel {
            score,
            label: label.to_string(),
            bytes,
            notes: format!("Size-based eco label ({} MB)", round_f64(mb, 2)),
        }),
    }
}
