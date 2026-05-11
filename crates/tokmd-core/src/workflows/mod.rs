//! Public workflow facade owner modules.

#[cfg(feature = "analysis")]
mod analyze;
#[cfg(feature = "cockpit")]
mod cockpit;
mod diff;
mod export;
mod lang;
mod module;
mod support;

#[cfg(feature = "analysis")]
pub use analyze::{
    analyze_workflow, analyze_workflow_from_inputs, supports_rootless_in_memory_analyze_preset,
};
#[cfg(all(test, feature = "analysis"))]
pub(crate) use analyze::{parse_analysis_preset, parse_effort_request};
#[cfg(feature = "cockpit")]
pub use cockpit::cockpit_workflow;
#[cfg(all(test, feature = "cockpit"))]
pub(crate) use cockpit::parse_cockpit_range_mode;
pub use diff::diff_workflow;
pub use export::{export_workflow, export_workflow_from_inputs};
pub use lang::{lang_workflow, lang_workflow_from_inputs};
pub use module::{module_workflow, module_workflow_from_inputs};

pub(crate) use support::{
    collect_pure_in_memory_rows, deterministic_in_memory_scan_options, scan_paths_or_current_dir,
    settings_to_scan_options, strip_virtual_export_prefix,
};
