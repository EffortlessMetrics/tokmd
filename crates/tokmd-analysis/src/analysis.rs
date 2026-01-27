use std::path::PathBuf;

use anyhow::Result;
use tokmd_analysis_types::{
    AnalysisArgsMeta, AnalysisReceipt, AnalysisSource, AssetReport, DependencyReport,
    DuplicateReport, FunReport, GitReport, ImportReport,
};
use tokmd_types::{ExportData, ScanStatus, ToolInfo};

use crate::derived::{build_tree, derive_report};
use crate::fun::build_fun_report;
use crate::util::now_ms;
#[cfg(feature = "walk")]
use crate::assets::{build_assets_report, build_dependency_report};
#[cfg(feature = "content")]
use crate::content::{build_duplicate_report, build_import_report, build_todo_report};
#[cfg(feature = "git")]
use crate::git::build_git_report;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisPreset {
    Receipt,
    Health,
    Risk,
    Supply,
    Architecture,
    Deep,
    Fun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportGranularity {
    Module,
    File,
}

#[derive(Debug, Clone)]
pub struct AnalysisLimits {
    pub max_files: Option<usize>,
    pub max_bytes: Option<u64>,
    pub max_file_bytes: Option<u64>,
    pub max_commits: Option<usize>,
    pub max_commit_files: Option<usize>,
}

impl Default for AnalysisLimits {
    fn default() -> Self {
        Self {
            max_files: None,
            max_bytes: None,
            max_file_bytes: None,
            max_commits: None,
            max_commit_files: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalysisContext {
    pub export: ExportData,
    pub root: PathBuf,
    pub source: AnalysisSource,
}

#[derive(Debug, Clone)]
pub struct AnalysisRequest {
    pub preset: AnalysisPreset,
    pub args: AnalysisArgsMeta,
    pub limits: AnalysisLimits,
    pub window_tokens: Option<usize>,
    pub git: Option<bool>,
    pub import_granularity: ImportGranularity,
}

#[derive(Debug, Clone, Copy)]
struct AnalysisPlan {
    assets: bool,
    deps: bool,
    todo: bool,
    dup: bool,
    imports: bool,
    git: bool,
    fun: bool,
}

impl AnalysisPlan {
    fn needs_files(&self) -> bool {
        self.assets || self.deps || self.todo || self.dup || self.imports
    }
}

fn plan_for(preset: AnalysisPreset) -> AnalysisPlan {
    match preset {
        AnalysisPreset::Receipt => AnalysisPlan {
            assets: false,
            deps: false,
            todo: false,
            dup: false,
            imports: false,
            git: false,
            fun: false,
        },
        AnalysisPreset::Health => AnalysisPlan {
            assets: false,
            deps: false,
            todo: true,
            dup: false,
            imports: false,
            git: false,
            fun: false,
        },
        AnalysisPreset::Risk => AnalysisPlan {
            assets: false,
            deps: false,
            todo: false,
            dup: false,
            imports: false,
            git: true,
            fun: false,
        },
        AnalysisPreset::Supply => AnalysisPlan {
            assets: true,
            deps: true,
            todo: false,
            dup: false,
            imports: false,
            git: false,
            fun: false,
        },
        AnalysisPreset::Architecture => AnalysisPlan {
            assets: false,
            deps: false,
            todo: false,
            dup: false,
            imports: true,
            git: false,
            fun: false,
        },
        AnalysisPreset::Deep => AnalysisPlan {
            assets: true,
            deps: true,
            todo: true,
            dup: true,
            imports: true,
            git: true,
            fun: false,
        },
        AnalysisPreset::Fun => AnalysisPlan {
            assets: false,
            deps: false,
            todo: false,
            dup: false,
            imports: false,
            git: false,
            fun: true,
        },
    }
}

pub fn analyze(ctx: AnalysisContext, req: AnalysisRequest) -> Result<AnalysisReceipt> {
    let mut warnings: Vec<String> = Vec::new();
    #[cfg_attr(not(feature = "content"), allow(unused_mut))]
    let mut derived = derive_report(&ctx.export, req.window_tokens);
    if req.args.format.contains("tree") {
        derived.tree = Some(build_tree(&ctx.export));
    }

    let plan = plan_for(req.preset);
    let include_git = match req.git {
        Some(flag) => flag,
        None => plan.git,
    };

    #[cfg(feature = "walk")]
    let mut assets: Option<AssetReport> = None;
    #[cfg(not(feature = "walk"))]
    let assets: Option<AssetReport> = None;

    #[cfg(feature = "walk")]
    let mut deps: Option<DependencyReport> = None;
    #[cfg(not(feature = "walk"))]
    let deps: Option<DependencyReport> = None;

    #[cfg(feature = "content")]
    let mut imports: Option<ImportReport> = None;
    #[cfg(not(feature = "content"))]
    let imports: Option<ImportReport> = None;

    #[cfg(feature = "content")]
    let mut dup: Option<DuplicateReport> = None;
    #[cfg(not(feature = "content"))]
    let dup: Option<DuplicateReport> = None;

    #[cfg(feature = "git")]
    let mut git: Option<GitReport> = None;
    #[cfg(not(feature = "git"))]
    let git: Option<GitReport> = None;

    let mut fun: Option<FunReport> = None;

    #[cfg(any(feature = "walk", feature = "content"))]
    let mut files: Option<Vec<PathBuf>> = None;
    #[cfg(not(any(feature = "walk", feature = "content")))]
    let _files: Option<Vec<PathBuf>> = None;

    if plan.needs_files() {
        #[cfg(feature = "walk")]
        match tokmd_walk::list_files(&ctx.root, req.limits.max_files) {
            Ok(list) => files = Some(list),
            Err(err) => warnings.push(format!("walk failed: {}", err)),
        }
        #[cfg(not(feature = "walk"))]
        {
            warnings.push("walk feature disabled; skipping file inventory".to_string());
        }
    }

    if plan.assets {
        #[cfg(feature = "walk")]
        {
            if let Some(list) = files.as_deref() {
                match build_assets_report(&ctx.root, list) {
                    Ok(report) => assets = Some(report),
                    Err(err) => warnings.push(format!("asset scan failed: {}", err)),
                }
            }
        }
    }

    if plan.deps {
        #[cfg(feature = "walk")]
        {
            if let Some(list) = files.as_deref() {
                match build_dependency_report(&ctx.root, list) {
                    Ok(report) => deps = Some(report),
                    Err(err) => warnings.push(format!("dependency scan failed: {}", err)),
                }
            }
        }
    }

    if plan.todo {
        #[cfg(feature = "content")]
        {
            if let Some(list) = files.as_deref() {
                match build_todo_report(&ctx.root, list, &req.limits, derived.totals.code) {
                    Ok(report) => derived.todo = Some(report),
                    Err(err) => warnings.push(format!("todo scan failed: {}", err)),
                }
            }
        }
        #[cfg(not(feature = "content"))]
        warnings.push("content feature disabled; skipping TODO scan".to_string());
    }

    if plan.dup {
        #[cfg(feature = "content")]
        {
            if let Some(list) = files.as_deref() {
                match build_duplicate_report(&ctx.root, list, &req.limits) {
                    Ok(report) => dup = Some(report),
                    Err(err) => warnings.push(format!("dup scan failed: {}", err)),
                }
            }
        }
        #[cfg(not(feature = "content"))]
        warnings.push("content feature disabled; skipping duplication scan".to_string());
    }

    if plan.imports {
        #[cfg(feature = "content")]
        {
            if let Some(list) = files.as_deref() {
                match build_import_report(
                    &ctx.root,
                    list,
                    &ctx.export,
                    req.import_granularity,
                    &req.limits,
                ) {
                    Ok(report) => imports = Some(report),
                    Err(err) => warnings.push(format!("import scan failed: {}", err)),
                }
            }
        }
        #[cfg(not(feature = "content"))]
        warnings.push("content feature disabled; skipping import scan".to_string());
    }

    if include_git {
        #[cfg(feature = "git")]
        match build_git_report(&ctx.root, &ctx.export, &req.limits) {
            Ok(report) => git = Some(report),
            Err(err) => warnings.push(format!("git scan failed: {}", err)),
        }
        #[cfg(not(feature = "git"))]
        warnings.push("git feature disabled; skipping git metrics".to_string());
    }

    if plan.fun {
        fun = Some(build_fun_report(&derived));
    }

    let status = if warnings.is_empty() {
        ScanStatus::Complete
    } else {
        ScanStatus::Partial
    };

    let receipt = AnalysisReceipt {
        schema_version: tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION,
        generated_at_ms: now_ms(),
        tool: ToolInfo::current(),
        mode: "analysis".to_string(),
        status,
        warnings,
        source: ctx.source,
        args: req.args,
        derived: Some(derived),
        assets,
        deps,
        git,
        imports,
        dup,
        fun,
    };

    Ok(receipt)
}

// Optional enrichers are implemented in later stages.
#[allow(dead_code)]
fn _unused_sections(
    _assets: Option<AssetReport>,
    _deps: Option<DependencyReport>,
    _git: Option<GitReport>,
    _imports: Option<ImportReport>,
    _dup: Option<DuplicateReport>,
    _fun: Option<FunReport>,
) {
}