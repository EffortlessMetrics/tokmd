//! # tokmd-import
//!
//! **Tier 2 (Adapter Layer)**
//!
//! Import extraction and dependency analysis for tokmd.
//! Handles extraction of import statements and dependency resolution.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

/// Errors that can occur during import analysis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    /// File could not be read.
    IoError { path: String, message: String },
    /// Content is not valid UTF-8.
    InvalidUtf8 { path: String },
    /// Language not supported for import extraction.
    UnsupportedLanguage { lang: String },
    /// Import pattern matching failed.
    PatternError { message: String },
    /// Dependency resolution failed.
    ResolutionError { dependency: String, message: String },
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError { path, message } => {
                write!(f, "IO error reading '{}': {}", path, message)
            }
            Self::InvalidUtf8 { path } => write!(f, "Invalid UTF-8 in file: {}", path),
            Self::UnsupportedLanguage { lang } => {
                write!(f, "Language not supported for imports: {}", lang)
            }
            Self::PatternError { message } => write!(f, "Pattern error: {}", message),
            Self::ResolutionError { dependency, message } => {
                write!(f, "Failed to resolve '{}': {}", dependency, message)
            }
        }
    }
}

impl std::error::Error for ImportError {}

impl From<std::io::Error> for ImportError {
    fn from(err: std::io::Error) -> Self {
        ImportError::IoError {
            path: "unknown".to_string(),
            message: err.to_string(),
        }
    }
}

/// Represents an extracted import.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Import {
    /// The imported module/path.
    pub target: String,
    /// Whether this is a local (relative) import.
    pub is_local: bool,
    /// The import statement as written in source.
    pub source_line: String,
}

/// Import analysis results for a file.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportAnalysis {
    /// Source file path.
    pub file_path: String,
    /// All imports found in the file.
    pub imports: Vec<Import>,
    /// Unique external dependencies (non-local).
    pub external_deps: Vec<String>,
    /// Unique local dependencies.
    pub local_deps: Vec<String>,
}

/// Analyze imports in a source file.
///
/// # Arguments
/// * `path` - Path to the source file
/// * `lang` - Programming language identifier
///
/// # Returns
/// Import analysis or an ImportError
pub fn analyze_file_imports(path: &Path, lang: &str) -> Result<ImportAnalysis, ImportError> {
    // Convert unwrap to Result propagation - Site 1
    let content = std::fs::read_to_string(path).map_err(|e| ImportError::IoError {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;

    let imports = extract_imports(&content, lang)?;

    let mut external = Vec::new();
    let mut local = Vec::new();

    for imp in &imports {
        if imp.is_local {
            local.push(imp.target.clone());
        } else {
            external.push(imp.target.clone());
        }
    }

    external.sort();
    external.dedup();
    local.sort();
    local.dedup();

    Ok(ImportAnalysis {
        file_path: path.display().to_string(),
        imports,
        external_deps: external,
        local_deps: local,
    })
}

/// Extract imports from content based on language.
fn extract_imports(content: &str, lang: &str) -> Result<Vec<Import>, ImportError> {
    match lang.to_ascii_lowercase().as_str() {
        "rust" | "rs" => extract_rust_imports(content),
        "python" | "py" => extract_python_imports(content),
        "javascript" | "js" | "typescript" | "ts" => extract_js_imports(content),
        "go" => extract_go_imports(content),
        _ => Err(ImportError::UnsupportedLanguage {
            lang: lang.to_string(),
        }),
    }
}

/// Extract Rust imports (use statements and mod declarations).
fn extract_rust_imports(content: &str) -> Result<Vec<Import>, ImportError> {
    let mut imports = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Handle use statements
        if trimmed.starts_with("use ") {
            let rest = trimmed.strip_prefix("use ").unwrap_or(trimmed);
            let target = rest
                .trim_end_matches(';')
                .split("::")
                .next()
                .unwrap_or(rest)
                .to_string();

            let is_local = target.starts_with("crate::")
                || target.starts_with("super::")
                || target.starts_with("self::")
                || target == "crate"
                || target == "super"
                || target == "self";

            imports.push(Import {
                target,
                is_local,
                source_line: trimmed.to_string(),
            });
        }

        // Handle mod statements (local modules)
        if trimmed.starts_with("mod ") && !trimmed.contains('{') {
            let rest = trimmed.strip_prefix("mod ").unwrap_or(trimmed);
            let target = rest
                .trim_end_matches(';')
                .split_whitespace()
                .next()
                .unwrap_or(rest)
                .to_string();

            imports.push(Import {
                target,
                is_local: true,
                source_line: trimmed.to_string(),
            });
        }
    }

    Ok(imports)
}

/// Extract Python imports.
fn extract_python_imports(content: &str) -> Result<Vec<Import>, ImportError> {
    let mut imports = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Handle: import x
        if trimmed.starts_with("import ") {
            let target = trimmed
                .strip_prefix("import ")
                .unwrap_or("")
                .split_whitespace()
                .next()
                .unwrap_or("")
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();

            if !target.is_empty() {
                imports.push(Import {
                    target: target.clone(),
                    is_local: target.starts_with('.'),
                    source_line: trimmed.to_string(),
                });
            }
        }

        // Handle: from x import y
        if trimmed.starts_with("from ") {
            let target = trimmed
                .strip_prefix("from ")
                .unwrap_or("")
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();

            if !target.is_empty() && !target.starts_with("import") {
                imports.push(Import {
                    target: target.clone(),
                    is_local: target.starts_with('.'),
                    source_line: trimmed.to_string(),
                });
            }
        }
    }

    Ok(imports)
}

/// Extract JavaScript/TypeScript imports.
fn extract_js_imports(content: &str) -> Result<Vec<Import>, ImportError> {
    let mut imports = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Handle: import x from 'y' or import { x } from 'y'
        if trimmed.starts_with("import ") {
            // Find the 'from' keyword
            if let Some(from_pos) = trimmed.find(" from ") {
                let after_from = &trimmed[from_pos + 6..];
                // Extract quoted string
                let target = extract_quoted(after_from).unwrap_or_default();

                if !target.is_empty() {
                    imports.push(Import {
                        target: target.clone(),
                        is_local: target.starts_with('.'),
                        source_line: trimmed.to_string(),
                    });
                }
            }
            continue;
        }

        // Handle: require('x')
        if trimmed.contains("require(") {
            if let Some(start) = trimmed.find("require(") {
                let after_require = &trimmed[start + 8..];
                if let Some(target) = extract_quoted(after_require) {
                    if !target.is_empty() {
                        imports.push(Import {
                            target: target.clone(),
                            is_local: target.starts_with('.'),
                            source_line: trimmed.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(imports)
}

/// Extract quoted string from text (single or double quotes)
fn extract_quoted(text: &str) -> Option<String> {
    let trimmed = text.trim();
    let mut chars = trimmed.chars();
    
    // Find opening quote
    let quote = chars.next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    
    let mut result = String::new();
    for ch in chars {
        if ch == quote {
            return Some(result);
        }
        result.push(ch);
    }
    None
}

/// Extract Go imports.
fn extract_go_imports(content: &str) -> Result<Vec<Import>, ImportError> {
    let mut imports = Vec::new();
    let mut in_import_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("import (") {
            in_import_block = true;
            continue;
        }

        if in_import_block {
            if trimmed.starts_with(')') {
                in_import_block = false;
                continue;
            }

            let target = trimmed.trim_matches('"').to_string();
            if !target.is_empty() && !target.contains(' ') {
                imports.push(Import {
                    target: target.clone(),
                    is_local: target.starts_with('.'),
                    source_line: trimmed.to_string(),
                });
            }
            continue;
        }

        // Single-line import
        if trimmed.starts_with("import ") && !trimmed.contains('(') {
            let target = trimmed
                .strip_prefix("import ")
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .to_string();

            if !target.is_empty() {
                imports.push(Import {
                    target: target.clone(),
                    is_local: target.starts_with('.'),
                    source_line: trimmed.to_string(),
                });
            }
        }
    }

    Ok(imports)
}

/// Analyze dependencies across multiple files.
///
/// # Arguments
/// * `files` - List of (path, language) tuples
///
/// # Returns
/// Map of file path to import analysis, or an error
pub fn analyze_project_imports(
    files: &[(&Path, &str)],
) -> Result<BTreeMap<String, ImportAnalysis>, ImportError> {
    let mut results = BTreeMap::new();

    for (path, lang) in files {
        let analysis = analyze_file_imports(path, lang)?;
        results.insert(analysis.file_path.clone(), analysis);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Test 1: Error propagation for non-existent file
    #[test]
    fn test_analyze_file_imports_not_found() {
        let path = Path::new("/nonexistent/file.rs");
        let result = analyze_file_imports(path, "rust");
        assert!(result.is_err());
        if let Err(ImportError::IoError { path: p, .. }) = result {
            assert!(p.contains("nonexistent"));
        } else {
            panic!("Expected IoError, got {:?}", result);
        }
    }

    // Test 2: Unsupported language
    #[test]
    fn test_unsupported_language() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"some code").unwrap();
        let result = analyze_file_imports(file.path(), "cobol");
        assert!(matches!(result, Err(ImportError::UnsupportedLanguage { .. })));
    }

    // Test 3: Rust import extraction
    #[test]
    fn test_extract_rust_imports() {
        let code = r#"
use std::collections::HashMap;
use serde::Deserialize;
use crate::utils::helper;
mod config;
"#;
        let result = extract_rust_imports(code);
        assert!(result.is_ok());
        let imports = result.unwrap();
        assert_eq!(imports.len(), 4);

        // Check external deps
        assert!(imports.iter().any(|i| i.target == "std" && !i.is_local));
        assert!(imports.iter().any(|i| i.target == "serde" && !i.is_local));

        // Check local deps
        assert!(imports.iter().any(|i| i.target == "crate" && i.is_local));
        assert!(imports.iter().any(|i| i.target == "config" && i.is_local));
    }

    // Test 4: Python import extraction
    #[test]
    fn test_extract_python_imports() {
        let code = r#"
import os
import sys
from pathlib import Path
from .local import helper
"#;
        let result = extract_python_imports(code);
        assert!(result.is_ok());
        let imports = result.unwrap();
        assert_eq!(imports.len(), 4);

        assert!(imports.iter().any(|i| i.target == "os" && !i.is_local));
        assert!(imports.iter().any(|i| i.target == ".local" && i.is_local));
    }

    // Test 5: JavaScript import extraction
    #[test]
    fn test_extract_js_imports() {
        let code = r#"
import React from 'react';
import { useState } from "react";
const fs = require('fs');
import helper from './local';
"#;
        let result = extract_js_imports(code);
        assert!(result.is_ok());
        let imports = result.unwrap();
        assert!(imports.len() >= 3);

        assert!(imports.iter().any(|i| i.target == "react" && !i.is_local));
        assert!(imports.iter().any(|i| i.target == "./local" && i.is_local));
    }

    // Test 6: Go import extraction
    #[test]
    fn test_extract_go_imports() {
        let code = r#"
import "fmt"
import (
    "os"
    "github.com/some/pkg"
)
"#;
        let result = extract_go_imports(code);
        assert!(result.is_ok());
        let imports = result.unwrap();
        assert_eq!(imports.len(), 3);

        assert!(imports.iter().any(|i| i.target == "fmt"));
        assert!(imports.iter().any(|i| i.target == "os"));
        assert!(imports.iter().any(|i| i.target == "github.com/some/pkg"));
    }

    // Test 7: Pattern error handling
    #[test]
    fn test_pattern_error_conversion() {
        let err = ImportError::PatternError {
            message: "test error".to_string(),
        };
        assert!(err.to_string().contains("test error"));
    }

    // Test 8: IO Error conversion
    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let import_err: ImportError = io_err.into();
        assert!(import_err.to_string().contains("not found"));
    }

    // Test 9: Resolution error display
    #[test]
    fn test_resolution_error_display() {
        let err = ImportError::ResolutionError {
            dependency: "my-crate".to_string(),
            message: "version conflict".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("my-crate"));
        assert!(msg.contains("version conflict"));
    }

    // Test 10: Full analysis with deduplication
    #[test]
    fn test_import_analysis_deduplication() {
        let code = r#"
use std::collections::HashMap;
use std::vec::Vec;
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(code.as_bytes()).unwrap();

        let result = analyze_file_imports(file.path(), "rust");
        assert!(result.is_ok());
        let analysis = result.unwrap();

        // std should appear in external deps (deduplicated)
        assert!(analysis.external_deps.contains(&"std".to_string()));
    }

    // Test 11: Empty file handling
    #[test]
    fn test_empty_file() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"").unwrap();
        let result = analyze_file_imports(file.path(), "rust");
        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.imports.is_empty());
    }

    // Test 12: Analyze project imports
    #[test]
    fn test_analyze_project_imports() {
        let mut file1 = NamedTempFile::new().unwrap();
        file1.write_all(b"use std::io;").unwrap();

        let mut file2 = NamedTempFile::new().unwrap();
        file2.write_all(b"import os").unwrap();

        let files = vec![(file1.path(), "rust"), (file2.path(), "python")];
        let result = analyze_project_imports(&files);
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(map.len(), 2);
    }
}
