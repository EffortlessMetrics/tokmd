//! # tokmd-parsing
//!
//! **Tier 2 (Adapter Layer)**
//!
//! Language-aware parsing utilities for tokmd analysis.
//! Provides parsing and normalization logic for source code analysis.

#![forbid(unsafe_code)]

use std::path::Path;

/// Errors that can occur during parsing operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// File could not be read.
    IoError { path: String, message: String },
    /// Content is not valid UTF-8.
    InvalidUtf8 { path: String },
    /// Unsupported language for parsing.
    UnsupportedLanguage { lang: String },
    /// Regex pattern compilation failed.
    PatternError { message: String },
    /// No content available to parse.
    EmptyContent,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError { path, message } => {
                write!(f, "IO error reading '{}': {}", path, message)
            }
            Self::InvalidUtf8 { path } => write!(f, "Invalid UTF-8 in file: {}", path),
            Self::UnsupportedLanguage { lang } => write!(f, "Unsupported language: {}", lang),
            Self::PatternError { message } => write!(f, "Pattern error: {}", message),
            Self::EmptyContent => write!(f, "No content available to parse"),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError {
            path: "unknown".to_string(),
            message: err.to_string(),
        }
    }
}

/// Parse a source file and extract structural information.
///
/// # Arguments
/// * `path` - Path to the source file
/// * `lang` - Programming language identifier
///
/// # Returns
/// Parsed content or a ParseError
pub fn parse_source_file(path: &Path, lang: &str) -> Result<ParsedSource, ParseError> {
    // Convert unwrap to Result propagation - Site 1
    let content = std::fs::read_to_string(path).map_err(|e| ParseError::IoError {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;

    if content.is_empty() {
        return Err(ParseError::EmptyContent);
    }

    // Convert unwrap to Result propagation - Site 2
    let parsed = parse_content(&content, lang).map_err(|_| ParseError::UnsupportedLanguage {
        lang: lang.to_string(),
    })?;

    Ok(parsed)
}

/// Parse content string based on language.
///
/// # Arguments
/// * `content` - Source code content
/// * `lang` - Programming language identifier
///
/// # Returns
/// Parsed content or an error indicator
fn parse_content(content: &str, lang: &str) -> Result<ParsedSource, ()> {
    match lang.to_ascii_lowercase().as_str() {
        "rust" | "rs" => parse_rust(content),
        "python" | "py" => parse_python(content),
        "javascript" | "js" | "typescript" | "ts" => parse_javascript(content),
        _ => Err(()),
    }
}

/// Parse Rust source code.
fn parse_rust(content: &str) -> Result<ParsedSource, ()> {
    // Simple parsing: count functions and structs
    let functions = content.matches("fn ").count();
    let structs = content.matches("struct ").count();
    let impls = content.matches("impl ").count();

    Ok(ParsedSource {
        language: "rust".to_string(),
        function_count: functions,
        struct_count: structs,
        impl_count: impls,
    })
}

/// Parse Python source code.
fn parse_python(content: &str) -> Result<ParsedSource, ()> {
    // Simple parsing: count functions and classes
    let functions = content.matches("def ").count();
    let classes = content.matches("class ").count();

    Ok(ParsedSource {
        language: "python".to_string(),
        function_count: functions,
        struct_count: classes,
        impl_count: 0,
    })
}

/// Parse JavaScript/TypeScript source code.
fn parse_javascript(content: &str) -> Result<ParsedSource, ()> {
    // Simple parsing: count functions
    let functions = content.matches("function ").count()
        + content.matches("=>" ).count()
        + content.matches("const ").count();

    Ok(ParsedSource {
        language: "javascript".to_string(),
        function_count: functions,
        struct_count: content.matches("class ").count(),
        impl_count: 0,
    })
}

/// Represents parsed source code information.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedSource {
    /// Programming language
    pub language: String,
    /// Number of functions detected
    pub function_count: usize,
    /// Number of structs/classes detected
    pub struct_count: usize,
    /// Number of impl blocks (Rust-specific)
    pub impl_count: usize,
}

/// Extract comments from source code.
///
/// # Arguments
/// * `content` - Source code content
/// * `lang` - Programming language identifier
///
/// # Returns
/// Vector of comment strings
pub fn extract_comments(content: &str, lang: &str) -> Result<Vec<String>, ParseError> {
    let lang = lang.to_ascii_lowercase();

    match lang.as_str() {
        "rust" | "rs" => {
            // Convert unwrap to Result propagation - Site 3
            let comments = extract_rust_comments(content)
                .map_err(|e| ParseError::PatternError { message: e })?;
            Ok(comments)
        }
        "python" | "py" => {
            // Convert unwrap to Result propagation - Site 4
            let comments = extract_python_comments(content)
                .map_err(|e| ParseError::PatternError { message: e })?;
            Ok(comments)
        }
        _ => Err(ParseError::UnsupportedLanguage { lang: lang.clone() }),
    }
}

fn extract_rust_comments(content: &str) -> Result<Vec<String>, String> {
    let mut comments = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            comments.push(trimmed[2..].trim().to_string());
        }
    }
    Ok(comments)
}

fn extract_python_comments(content: &str) -> Result<Vec<String>, String> {
    let mut comments = Vec::new();
    for line in content.lines() {
        if let Some(idx) = line.find('#') {
            comments.push(line[idx + 1..].trim().to_string());
        }
    }
    Ok(comments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Test 1: Error propagation for non-existent file
    #[test]
    fn test_parse_source_file_not_found() {
        let path = Path::new("/nonexistent/file.rs");
        let result = parse_source_file(path, "rust");
        assert!(result.is_err());
        if let Err(ParseError::IoError { path: p, .. }) = result {
            assert!(p.contains("nonexistent"));
        } else {
            panic!("Expected IoError, got {:?}", result);
        }
    }

    // Test 2: Error propagation for empty content
    #[test]
    fn test_parse_source_file_empty() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"").unwrap();
        let result = parse_source_file(file.path(), "rust");
        assert!(matches!(result, Err(ParseError::EmptyContent)));
    }

    // Test 3: Successful parsing of Rust file
    #[test]
    fn test_parse_rust_source() {
        let code = r#"
fn main() {}
fn helper() {}
struct Point { x: i32 }
impl Point {}
"#;
        let result = parse_content(code, "rust");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.function_count, 2);
        assert_eq!(parsed.struct_count, 1);
        assert_eq!(parsed.impl_count, 1);
    }

    // Test 4: Error for unsupported language
    #[test]
    fn test_unsupported_language() {
        let code = "some code";
        let result = parse_content(code, "cobol");
        assert!(result.is_err());
    }

    // Test 5: Extract Rust comments
    #[test]
    fn test_extract_rust_comments() {
        let code = r#"
// This is a comment
fn main() {}
// Another comment
"#;
        let result = extract_comments(code, "rust");
        assert!(result.is_ok());
        let comments = result.unwrap();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0], "This is a comment");
        assert_eq!(comments[1], "Another comment");
    }

    // Test 6: Extract Python comments
    #[test]
    fn test_extract_python_comments() {
        let code = r#"
# Python comment
def main():
    pass
# Another python comment
"#;
        let result = extract_comments(code, "python");
        assert!(result.is_ok());
        let comments = result.unwrap();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0], "Python comment");
        assert_eq!(comments[1], "Another python comment");
    }

    // Test 7: Python parsing
    #[test]
    fn test_parse_python_source() {
        let code = r#"
def main():
    pass
class Point:
    pass
"#;
        let result = parse_content(code, "python");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.function_count, 1);
        assert_eq!(parsed.struct_count, 1);
    }

    // Test 8: JavaScript parsing
    #[test]
    fn test_parse_javascript_source() {
        let code = r#"
function main() {}
const add = () => {};
class Point {}
"#;
        let result = parse_content(code, "javascript");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.function_count >= 2);
        assert_eq!(parsed.struct_count, 1);
    }

    // Test 9: Error propagation test - boundary case
    #[test]
    fn test_parse_error_display() {
        let err = ParseError::UnsupportedLanguage {
            lang: "cobol".to_string(),
        };
        assert!(err.to_string().contains("cobol"));
    }

    // Test 10: IO Error from std::io::Error
    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let parse_err: ParseError = io_err.into();
        assert!(parse_err.to_string().contains("file not found"));
    }
}
