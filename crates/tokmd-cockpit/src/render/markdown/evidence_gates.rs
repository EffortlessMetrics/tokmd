use std::fmt::Write;

use crate::CockpitReceipt;

pub(super) fn render(s: &mut String, receipt: &CockpitReceipt) {
    let _ = writeln!(s, "### Evidence Gates");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **Overall status**: {:?}",
        receipt.evidence.overall_status
    );
    let _ = writeln!(
        s,
        "- **Mutation**: {:?} (killed: {}, survivors: {})",
        receipt.evidence.mutation.meta.status,
        receipt.evidence.mutation.killed,
        receipt.evidence.mutation.survivors.len()
    );
    if let Some(ref dc) = receipt.evidence.diff_coverage {
        let _ = writeln!(
            s,
            "- **Diff coverage**: {:?} ({:.1}%)",
            dc.meta.status,
            dc.coverage_pct * 100.0
        );
    }
    if let Some(ref contracts) = receipt.evidence.contracts {
        let _ = writeln!(
            s,
            "- **Contracts**: {:?} (failures: {})",
            contracts.meta.status, contracts.failures
        );
    }
    if let Some(ref sc) = receipt.evidence.supply_chain {
        let _ = writeln!(
            s,
            "- **Supply chain**: {:?} (vulnerabilities: {})",
            sc.meta.status,
            sc.vulnerabilities.len()
        );
    }
    if let Some(ref det) = receipt.evidence.determinism {
        let _ = writeln!(
            s,
            "- **Determinism**: {:?} (differences: {})",
            det.meta.status,
            det.differences.len()
        );
    }
    if let Some(ref cx) = receipt.evidence.complexity {
        let _ = writeln!(
            s,
            "- **Complexity**: {:?} (avg cyclomatic: {:.1}, max: {})",
            cx.meta.status, cx.avg_cyclomatic, cx.max_cyclomatic
        );
    }
    let _ = writeln!(s);
}
