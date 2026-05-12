#![cfg_attr(not(feature = "git"), allow(unused_imports))]
use std::path::Path;

use tokmd_types::ExportData;

use crate::grid::PresetPlan;

#[cfg(feature = "git")]
use super::super::files::{ROOTLESS_GIT_ANALYSIS_WARNING, push_warning_once};
use super::super::outputs::AnalysisOutputs;

#[cfg_attr(not(feature = "git"), allow(dead_code))]
pub(in crate::analysis) struct GitInput<'a> {
    pub(in crate::analysis) root: &'a Path,
    pub(in crate::analysis) export: &'a ExportData,
    pub(in crate::analysis) plan: &'a PresetPlan,
    pub(in crate::analysis) include_git: bool,
    pub(in crate::analysis) max_commits: Option<usize>,
    pub(in crate::analysis) max_commit_files: Option<usize>,
    pub(in crate::analysis) has_host_root: bool,
}

pub(in crate::analysis) fn run(
    input: GitInput<'_>,
    outputs: &mut AnalysisOutputs,
    warnings: &mut Vec<String>,
) {
    if input.include_git {
        #[cfg(feature = "git")]
        {
            if input.has_host_root {
                let repo_root = match tokmd_git::repo_root(input.root) {
                    Some(root) => root,
                    None => {
                        warnings.push("git scan failed: not a git repo".to_string());
                        std::path::PathBuf::new()
                    }
                };
                if !repo_root.as_os_str().is_empty() {
                    match tokmd_git::collect_history(
                        &repo_root,
                        input.max_commits,
                        input.max_commit_files,
                    ) {
                        Ok(commits) => {
                            if input.plan.git {
                                match crate::git::build_git_report(
                                    &repo_root,
                                    input.export,
                                    &commits,
                                ) {
                                    Ok(report) => outputs.git = Some(report),
                                    Err(err) => warnings.push(format!("git scan failed: {}", err)),
                                }
                            }
                            if input.plan.churn {
                                outputs.churn = Some(crate::git::build_predictive_churn_report(
                                    input.export,
                                    &commits,
                                    &repo_root,
                                ));
                            }
                            if input.plan.fingerprint {
                                outputs.fingerprint =
                                    Some(crate::fingerprint::build_corporate_fingerprint(&commits));
                            }
                        }
                        Err(err) => warnings.push(format!("git scan failed: {}", err)),
                    }
                }
            } else {
                push_warning_once(warnings, ROOTLESS_GIT_ANALYSIS_WARNING);
            }
        }
        #[cfg(not(feature = "git"))]
        {
            let _ = (input, outputs);
            warnings.push(
                crate::grid::DisabledFeature::GitMetrics
                    .warning()
                    .to_string(),
            );
        }
    }
}
