use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, DerivedReport};
use tokmd_types::{ScanStatus, ToolInfo};

use crate::util::now_ms;

use super::reports::AnalysisReports;

pub(super) fn build_receipt(
    source: AnalysisSource,
    args: AnalysisArgsMeta,
    derived: DerivedReport,
    reports: AnalysisReports,
    warnings: Vec<String>,
) -> AnalysisReceipt {
    let status = if warnings.is_empty() {
        ScanStatus::Complete
    } else {
        ScanStatus::Partial
    };

    AnalysisReceipt {
        schema_version: tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "analysis".to_string(),
        status,
        warnings,
        source,
        args,
        archetype: reports.archetype,
        topics: reports.topics,
        entropy: reports.entropy,
        predictive_churn: reports.predictive_churn,
        corporate_fingerprint: reports.corporate_fingerprint,
        license: reports.license,
        derived: Some(derived),
        assets: reports.assets,
        deps: reports.deps,
        git: reports.git,
        imports: reports.imports,
        dup: reports.dup,
        complexity: reports.complexity,
        api_surface: reports.api_surface,
        effort: reports.effort,
        fun: reports.fun,
    }
}
