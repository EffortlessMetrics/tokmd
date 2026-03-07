use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result};

use tokmd_analysis_types::{EffortDeltaClassification, EffortDeltaReport, GitReport};
use tokmd_types::{ExportData, FileKind};

#[cfg(feature = "git")]
use tokmd_git::GitRangeMode;

use crate::cocomo81::cocomo81_baseline;

pub fn build_delta(
    root: &Path,
    export: &ExportData,
    git: Option<&GitReport>,
    base_ref: &str,
    head_ref: &str,
) -> Result<EffortDeltaReport> {
    let base = base_ref.trim().to_string();
    let head = head_ref.trim().to_string();
    if base.is_empty() || head.is_empty() {
        anyhow::bail!("both base_ref and head_ref are required");
    }

    #[cfg(not(feature = "git"))]
    {
        let _ = (root, export, git);
        anyhow::bail!("delta estimation requires the tokmd-git feature");
    }

    #[cfg(feature = "git")]
    {
        let repo_root = tokmd_git::repo_root(root)
            .context("failed to locate git repository")?;

        let changed = tokmd_git::get_added_lines(&repo_root, &base, &head, GitRangeMode::TwoDot)
            .context("failed to compute changed files")?;

        let (files_changed, changed_lines) = changed
            .iter()
            .fold((0usize, 0usize), |(files, lines), (_path, hunks)| {
                (files + 1, lines + hunks.len())
            });

        let mut path_to_module_lang = BTreeMap::<String, (String, String)>::new();
        let mut all_modules = BTreeSet::<String>::new();
        let mut all_langs = BTreeSet::<String>::new();
        let mut changed_modules = BTreeSet::<String>::new();
        let mut changed_langs = BTreeSet::<String>::new();

        for row in &export.rows {
            if row.kind != FileKind::Parent {
                continue;
            }
            path_to_module_lang.insert(row.path.clone(), (row.module.clone(), row.lang.clone()));
            all_modules.insert(row.module.clone());
            all_langs.insert(row.lang.clone());
        }

        for path in changed.keys() {
            let key = path.to_string_lossy().trim_start_matches("./").to_string();
            if let Some((module, lang)) = path_to_module_lang.get(&key) {
                changed_modules.insert(module.clone());
                changed_langs.insert(lang.clone());
            }
        }

        let total_files = path_to_module_lang.len();
        let modules_ratio = if all_modules.is_empty() {
            0.0
        } else {
            (changed_modules.len() as f64) / (all_modules.len() as f64)
        };
        let langs_ratio = if all_langs.is_empty() {
            0.0
        } else {
            (changed_langs.len() as f64) / (all_langs.len() as f64)
        };
        let hotspot_files_touched = if let Some(git_report) = git {
            changed
                .keys()
                .filter_map(|path| {
                    let normalized = path.to_string_lossy().trim_start_matches("./");
                    Some(normalized)
                })
                .filter(|path| {
                    git_report
                        .hotspots
                        .iter()
                        .any(|row| row.path == **path)
                })
                .count()
        } else {
            0
        };

        let coupling_total = git.map(|g| g.coupling.len()).unwrap_or(0) as f64;
        let coupled_neighbors_touched = if let Some(git_report) = git {
            let touched_modules = &changed_modules;
            git_report
                .coupling
                .iter()
                .filter(|c| touched_modules.contains(&c.left) || touched_modules.contains(&c.right))
                .count() as f64
        } else {
            0.0
        };

        let coupling_ratio = if coupling_total <= 0.0 {
            0.0
        } else {
            coupled_neighbors_touched / coupling_total
        };

        // Simple deterministic blast score:
        // 15*core + 3*modules + 1*log1p(files) + 2*hotspot + 2*coupled-neighbors
        let log_files = (files_changed as f64).ln_1p();
        let core_boundary_crossed = if files_changed > 0 { 1.0 } else { 0.0 };
        let blast_radius = (15.0 * core_boundary_crossed)
            + (3.0 * modules_ratio)
            + (1.0 * log_files)
            + (2.0 * (hotspot_files_touched as f64))
            + (2.0 * coupled_neighbors_touched);

        let clamped_blast = blast_radius.clamp(0.0, 100.0);

        let changed_kloc = if changed_lines > 0 {
            (changed_lines as f64 / 1000.0) * (1.0 + clamped_blast / 100.0)
        } else if files_changed > 0 {
            (files_changed as f64 * 0.02) * (1.0 + clamped_blast / 100.0)
        } else {
            0.0
        }
        .max(0.001);

        let report = cocomo81_baseline(changed_kloc);

        let classification = classify_blast(clamped_blast);

        Ok(EffortDeltaReport {
            base,
            head,
            files_changed,
            modules_changed: changed_modules.len(),
            langs_changed: changed_langs.len(),
            hotspot_files_touched,
            coupled_neighbors_touched: coupled_neighbors_touched as usize,
            blast_radius: clamped_blast,
            classification,
            effort_pm_low: report.effort_pm_low,
            effort_pm_est: report.effort_pm_p50,
            effort_pm_high: report.effort_pm_p80,
        })
    }
}

fn classify_blast(blast_radius: f64) -> EffortDeltaClassification {
    if blast_radius < 10.0 {
        EffortDeltaClassification::Low
    } else if blast_radius < 20.0 {
        EffortDeltaClassification::Medium
    } else if blast_radius < 35.0 {
        EffortDeltaClassification::High
    } else {
        EffortDeltaClassification::Critical
    }
}
