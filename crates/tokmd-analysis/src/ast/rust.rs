use super::capability::{AstCapability, AstLanguage};
use std::error::Error;
use std::fmt;
use tree_sitter::{Node, Parser};

pub const TREE_SITTER_RUST_CRATE: &str = "tree-sitter-rust";
pub const RUST_CAPABILITY: AstCapability =
    AstCapability::parser_backed_shadow(AstLanguage::Rust, TREE_SITTER_RUST_CRATE);
pub static CAPABILITIES: &[AstCapability] = &[RUST_CAPABILITY];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustAstShadow {
    pub has_error: bool,
    pub landmarks: Vec<RustLandmark>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RustLandmark {
    pub kind: RustLandmarkKind,
    pub name: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RustLandmarkKind {
    ControlFlow,
    Function,
    Import,
}

#[derive(Debug)]
pub enum RustAstError {
    Language(tree_sitter::LanguageError),
    ParseFailed,
}

impl fmt::Display for RustAstError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Language(error) => write!(f, "failed to load Rust Tree-sitter language: {error}"),
            Self::ParseFailed => f.write_str("failed to parse Rust source"),
        }
    }
}

impl Error for RustAstError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Language(error) => Some(error),
            Self::ParseFailed => None,
        }
    }
}

pub fn parse_rust_landmarks(source: &str) -> Result<RustAstShadow, RustAstError> {
    let mut parser = Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser
        .set_language(&language.into())
        .map_err(RustAstError::Language)?;
    let tree = parser
        .parse(source, None)
        .ok_or(RustAstError::ParseFailed)?;

    let mut landmarks = Vec::new();
    collect_landmarks(tree.root_node(), source.as_bytes(), &mut landmarks);
    landmarks.sort_by(|left, right| {
        left.start_byte
            .cmp(&right.start_byte)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.name.cmp(&right.name))
    });

    Ok(RustAstShadow {
        has_error: tree.root_node().has_error(),
        landmarks,
    })
}

fn collect_landmarks(node: Node<'_>, source: &[u8], landmarks: &mut Vec<RustLandmark>) {
    match node.kind() {
        "function_item" => {
            if let Some(name) = function_name(node, source) {
                push_landmark(node, RustLandmarkKind::Function, name, landmarks);
            }
        }
        "use_declaration" => {
            if let Some(name) = use_declaration_name(node, source) {
                push_landmark(node, RustLandmarkKind::Import, name, landmarks);
            }
        }
        kind => {
            if let Some(name) = control_flow_name(kind) {
                push_landmark(
                    node,
                    RustLandmarkKind::ControlFlow,
                    name.to_owned(),
                    landmarks,
                );
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_landmarks(child, source, landmarks);
    }
}

fn function_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    node.child_by_field_name("name")
        .and_then(|name| name.utf8_text(source).ok())
        .map(str::to_owned)
}

fn use_declaration_name(node: Node<'_>, source: &[u8]) -> Option<String> {
    normalized_node_text(node, source).map(|text| {
        text.strip_prefix("use ")
            .unwrap_or(&text)
            .trim_end_matches(';')
            .trim()
            .to_owned()
    })
}

fn normalized_node_text(node: Node<'_>, source: &[u8]) -> Option<String> {
    node.utf8_text(source)
        .ok()
        .map(|text| text.split_whitespace().collect::<Vec<_>>().join(" "))
}

fn control_flow_name(kind: &str) -> Option<&'static str> {
    match kind {
        "if_expression" => Some("if"),
        "match_expression" => Some("match"),
        "for_expression" => Some("for"),
        "while_expression" => Some("while"),
        "loop_expression" => Some("loop"),
        _ => None,
    }
}

fn push_landmark(
    node: Node<'_>,
    kind: RustLandmarkKind,
    name: String,
    landmarks: &mut Vec<RustLandmark>,
) {
    let start = node.start_position();
    let end = node.end_position();
    landmarks.push(RustLandmark {
        kind,
        name,
        start_byte: node.start_byte(),
        end_byte: node.end_byte(),
        start_line: start.row + 1,
        end_line: end.row + 1,
    });
}

#[cfg(test)]
mod tests {
    use super::{RustLandmarkKind, parse_rust_landmarks};

    #[test]
    fn parses_top_level_and_impl_function_landmarks() {
        let source = r#"
fn top_level() {}

impl Widget {
    pub fn method(&self) {}
}

async fn compute() {}
"#;

        let shadow = parse_rust_landmarks(source).expect("Rust source should parse");

        assert!(!shadow.has_error);
        assert_eq!(
            shadow
                .landmarks
                .iter()
                .map(|landmark| (landmark.kind, landmark.name.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (RustLandmarkKind::Function, "top_level"),
                (RustLandmarkKind::Function, "method"),
                (RustLandmarkKind::Function, "compute"),
            ]
        );
        assert!(
            shadow
                .landmarks
                .windows(2)
                .all(|pair| pair[0].start_byte < pair[1].start_byte)
        );
    }

    #[test]
    fn reports_parse_errors_without_dropping_valid_landmarks() {
        let source = "fn ok() {}\nfn broken(";

        let shadow = parse_rust_landmarks(source).expect("Tree-sitter recovers from syntax errors");

        assert!(shadow.has_error);
        assert_eq!(shadow.landmarks.len(), 1);
        assert_eq!(shadow.landmarks[0].name, "ok");
    }

    #[test]
    fn records_one_based_line_numbers() {
        let source = "\n\nfn third_line() {\n}\n";

        let shadow = parse_rust_landmarks(source).expect("Rust source should parse");

        assert_eq!(shadow.landmarks.len(), 1);
        assert_eq!(shadow.landmarks[0].start_line, 3);
        assert_eq!(shadow.landmarks[0].end_line, 4);
    }

    #[test]
    fn parses_import_and_simple_control_flow_landmarks() {
        let source = r#"
use std::{
    fs,
    path::Path,
};

fn compute(value: i32) {
    if value > 0 {
        for item in 0..value {
            while item > 1 {
                break;
            }
        }
    }

    match value {
        0 => loop {
            break;
        },
        _ => {}
    }
}
"#;

        let shadow = parse_rust_landmarks(source).expect("Rust source should parse");

        assert_eq!(
            shadow
                .landmarks
                .iter()
                .map(|landmark| (landmark.kind, landmark.name.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (RustLandmarkKind::Import, "std::{ fs, path::Path, }"),
                (RustLandmarkKind::Function, "compute"),
                (RustLandmarkKind::ControlFlow, "if"),
                (RustLandmarkKind::ControlFlow, "for"),
                (RustLandmarkKind::ControlFlow, "while"),
                (RustLandmarkKind::ControlFlow, "match"),
                (RustLandmarkKind::ControlFlow, "loop"),
            ]
        );
    }
}
