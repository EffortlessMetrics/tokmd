use tokmd_analysis_types::{AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, DerivedReport};
use tokmd_types::{ExportData, ScanStatus, ToolInfo};

use crate::derived::{build_tree, derive_report};
use crate::util::now_ms;

use super::report_set::AnalysisReports;

pub(super) fn prepare_derived(
    export: &ExportData,
    args: &AnalysisArgsMeta,
    window_tokens: Option<usize>,
) -> DerivedReport {
    let mut derived = derive_report(export, window_tokens);
    if args.format.contains("tree") {
        derived.tree = Some(build_tree(export));
    }
    derived
}

pub(super) fn source_with_base_signature(
    source: &AnalysisSource,
    derived: &DerivedReport,
) -> AnalysisSource {
    let mut source = source.clone();
    if source.base_signature.is_none() {
        source.base_signature = Some(derived.integrity.hash.clone());
    }
    source
}

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
