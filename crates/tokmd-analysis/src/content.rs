use std::path::{Path, PathBuf};

use anyhow::Result;
use tokmd_analysis_types::{DuplicateReport, ImportReport, TodoReport};
use tokmd_types::ExportData;

use crate::analysis::ImportGranularity;
use tokmd_analysis_util::AnalysisLimits;

pub fn build_todo_report(
    root: &Path,
    files: &[PathBuf],
    limits: &AnalysisLimits,
    total_code: usize,
) -> Result<TodoReport> {
    let limits = tokmd_analysis_content::ContentLimits {
        max_bytes: limits.max_bytes,
        max_file_bytes: limits.max_file_bytes,
    };
    tokmd_analysis_content::build_todo_report(root, files, &limits, total_code)
}

pub fn build_duplicate_report(
    root: &Path,
    files: &[PathBuf],
    export: &ExportData,
    limits: &AnalysisLimits,
) -> Result<DuplicateReport> {
    let limits = tokmd_analysis_content::ContentLimits {
        max_bytes: limits.max_bytes,
        max_file_bytes: limits.max_file_bytes,
    };
    tokmd_analysis_content::build_duplicate_report(root, files, export, &limits)
}

pub fn build_import_report(
    root: &Path,
    files: &[PathBuf],
    export: &ExportData,
    granularity: ImportGranularity,
    limits: &AnalysisLimits,
) -> Result<ImportReport> {
    let limits = tokmd_analysis_content::ContentLimits {
        max_bytes: limits.max_bytes,
        max_file_bytes: limits.max_file_bytes,
    };
    let granularity = match granularity {
        ImportGranularity::Module => tokmd_analysis_content::ImportGranularity::Module,
        ImportGranularity::File => tokmd_analysis_content::ImportGranularity::File,
    };
    tokmd_analysis_content::build_import_report(root, files, export, granularity, &limits)
}
