//! Public workflow facade owner modules.

#[cfg(feature = "analysis")]
mod analyze;
mod diff;
mod export;
mod lang;
mod module;

#[cfg(feature = "analysis")]
pub use analyze::{
    analyze_workflow, analyze_workflow_from_inputs, supports_rootless_in_memory_analyze_preset,
};
#[cfg(all(test, feature = "analysis"))]
pub(crate) use analyze::{parse_analysis_preset, parse_effort_request};
pub use diff::diff_workflow;
pub use export::{export_workflow, export_workflow_from_inputs};
pub use lang::{lang_workflow, lang_workflow_from_inputs};
pub use module::{module_workflow, module_workflow_from_inputs};
