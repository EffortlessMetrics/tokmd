use tokmd_analysis_types::{AnalysisSource, DerivedReport};
use tokmd_types::ExportData;

use crate::derived::{build_tree, derive_report};

use super::AnalysisRequest;

pub(super) fn build_derived(export: &ExportData, req: &AnalysisRequest) -> DerivedReport {
    let mut derived = derive_report(export, req.window_tokens);
    if req.args.format.contains("tree") {
        derived.tree = Some(build_tree(export));
    }
    derived
}

pub(super) fn source_with_signature(
    mut source: AnalysisSource,
    signature: String,
) -> AnalysisSource {
    if source.base_signature.is_none() {
        source.base_signature = Some(signature);
    }
    source
}
