use serde::Serialize;
use tokmd_config::{ChildIncludeMode, ChildrenMode};

/// A small totals struct shared by summary outputs.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Totals {
    pub code: usize,
    pub lines: usize,
    pub files: usize,
    pub avg_lines: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LangRow {
    pub lang: String,
    pub code: usize,
    pub lines: usize,
    pub files: usize,
    pub avg_lines: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct LangReport {
    pub rows: Vec<LangRow>,
    pub total: Totals,
    pub with_files: bool,
    pub children: ChildrenMode,
    pub top: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ModuleRow {
    pub module: String,
    pub code: usize,
    pub lines: usize,
    pub files: usize,
    pub avg_lines: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleReport {
    pub rows: Vec<ModuleRow>,
    pub total: Totals,
    pub module_roots: Vec<String>,
    pub module_depth: usize,
    pub children: ChildIncludeMode,
    pub top: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    Parent,
    Child,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FileRow {
    pub path: String,
    pub module: String,
    pub lang: String,
    pub kind: FileKind,
    pub code: usize,
    pub comments: usize,
    pub blanks: usize,
    pub lines: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportData {
    pub rows: Vec<FileRow>,
    pub module_roots: Vec<String>,
    pub module_depth: usize,
    pub children: ChildIncludeMode,
}
