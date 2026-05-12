use std::path::Path;

use tokmd_analysis_types::AnalysisLimits;
use tokmd_types::ExportData;

use crate::grid::PresetPlan;

use super::reports::AnalysisReports;
#[cfg(feature = "git")]
use super::{ROOTLESS_GIT_ANALYSIS_WARNING, has_host_root, push_warning_once};

pub(super) fn enrich_git_reports(
    root: &Path,
    export: &ExportData,
    limits: &AnalysisLimits,
    include_git: bool,
    plan: &PresetPlan,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    if !include_git {
        return;
    }

    #[cfg(feature = "git")]
    {
        if has_host_root(root) {
            collect_git_reports(root, export, limits, plan, reports, warnings);
        } else {
            push_warning_once(warnings, ROOTLESS_GIT_ANALYSIS_WARNING);
        }
    }

    #[cfg(not(feature = "git"))]
    warnings.push(
        crate::grid::DisabledFeature::GitMetrics
            .warning()
            .to_string(),
    );

    let _ = (root, export, limits, plan, reports, warnings);
}

#[cfg(feature = "git")]
fn collect_git_reports(
    root: &Path,
    export: &ExportData,
    limits: &AnalysisLimits,
    plan: &PresetPlan,
    reports: &mut AnalysisReports,
    warnings: &mut Vec<String>,
) {
    let Some(repo_root) = tokmd_git::repo_root(root) else {
        warnings.push("git scan failed: not a git repo".to_string());
        return;
    };

    match tokmd_git::collect_history(&repo_root, limits.max_commits, limits.max_commit_files) {
        Ok(commits) => {
            if plan.git {
                match crate::git::build_git_report(&repo_root, export, &commits) {
                    Ok(report) => reports.git = Some(report),
                    Err(err) => warnings.push(format!("git scan failed: {}", err)),
                }
            }
            if plan.churn {
                reports.predictive_churn = Some(crate::git::build_predictive_churn_report(
                    export, &commits, &repo_root,
                ));
            }
            if plan.fingerprint {
                reports.corporate_fingerprint =
                    Some(crate::fingerprint::build_corporate_fingerprint(&commits));
            }
        }
        Err(err) => warnings.push(format!("git scan failed: {}", err)),
    }
}
