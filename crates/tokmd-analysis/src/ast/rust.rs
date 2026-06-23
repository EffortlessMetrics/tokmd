use super::capability::{AstCapability, AstLanguage};
use super::facts::{
    SyntaxCallSite, SyntaxExport, SyntaxFacts, SyntaxImport, SyntaxRiskSeam, SyntaxSpan,
    SyntaxSymbol,
};
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

#[must_use]
pub fn extract_rust_facts(root: Node<'_>, source: &str) -> SyntaxFacts {
    let mut facts = SyntaxFacts::default();
    visit_syntax_node(root, source, &mut facts);
    facts.normalize();
    facts
}

fn visit_syntax_node(node: Node<'_>, source: &str, facts: &mut SyntaxFacts) {
    match node.kind() {
        "function_item" => push_rust_symbol(node, source, "function", facts),
        "struct_item" => push_rust_symbol(node, source, "struct", facts),
        "enum_item" => push_rust_symbol(node, source, "enum", facts),
        "trait_item" => push_rust_symbol(node, source, "trait", facts),
        "use_declaration" => push_rust_import(node, source, facts),
        "call_expression" => push_rust_call(node, source, facts),
        "macro_invocation" => push_rust_macro(node, source, facts),
        "index_expression" => {
            push_rust_risk("indexing", node_text(source, node), node, source, facts);
            push_guard_evidence(node, source, facts);
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_syntax_node(child, source, facts);
    }
}

fn push_rust_symbol(node: Node<'_>, source: &str, kind: &str, facts: &mut SyntaxFacts) {
    let name = node
        .child_by_field_name("name")
        .and_then(|name| node_text_checked(source, name))
        .map(compact_text)
        .or_else(|| first_identifier_text(node, source))
        .unwrap_or_else(|| "<anonymous>".to_owned());
    let exported = is_public_surface(node, source);
    let span = SyntaxSpan::from_node(node);

    facts.symbols.push(SyntaxSymbol {
        kind: kind.to_owned(),
        name: name.clone(),
        span,
        exported,
        public_surface: exported,
    });

    if exported {
        facts.exports.push(SyntaxExport {
            kind: kind.to_owned(),
            name,
            span,
        });
    }
}

fn push_rust_import(node: Node<'_>, source: &str, facts: &mut SyntaxFacts) {
    if let Some(imported) = normalized_node_text_str(node, source) {
        let imported = imported
            .strip_prefix("use ")
            .unwrap_or(&imported)
            .trim_end_matches(';')
            .trim()
            .to_owned();
        facts.imports.push(SyntaxImport {
            kind: "use".to_owned(),
            module: None,
            imported: vec![imported],
            dynamic: false,
            span: SyntaxSpan::from_node(node),
        });
    }
}

fn push_rust_call(node: Node<'_>, source: &str, facts: &mut SyntaxFacts) {
    let callee = node
        .child_by_field_name("function")
        .and_then(|function| node_text_checked(source, function))
        .map(compact_text)
        .or_else(|| first_named_child_text(node, source))
        .unwrap_or_else(|| "<unknown>".to_owned());
    let span = SyntaxSpan::from_node(node);

    facts.call_sites.push(SyntaxCallSite {
        kind: "call".to_owned(),
        callee: callee.clone(),
        dynamic: false,
        span,
    });

    if is_capacity_callee(&callee) {
        push_rust_risk("capacity_allocation", callee.as_str(), node, source, facts);
        push_guard_evidence(node, source, facts);
    }
    if is_fallible_conversion_callee(&callee) {
        push_rust_risk("fallible_conversion", callee.as_str(), node, source, facts);
    }
    if let Some(method) = method_name_from_callee(&callee) {
        match method.as_str() {
            "unwrap" | "unwrap_unchecked" => {
                push_rust_risk("unwrap", callee.as_str(), node, source, facts);
                push_guard_evidence(node, source, facts);
            }
            "expect" => {
                if callee.contains("try_from") || callee.contains("try_into") {
                    push_rust_risk(
                        "fallible_conversion_expect",
                        callee.as_str(),
                        node,
                        source,
                        facts,
                    );
                } else {
                    push_rust_risk("expect", callee.as_str(), node, source, facts);
                }
                push_guard_evidence(node, source, facts);
            }
            _ => {}
        }
    }
}

fn push_rust_macro(node: Node<'_>, source: &str, facts: &mut SyntaxFacts) {
    let macro_name = node
        .child_by_field_name("macro")
        .and_then(|macro_node| node_text_checked(source, macro_node))
        .map(|name| name.trim_end_matches('!').to_owned())
        .or_else(|| macro_name_from_text(node_text(source, node)))
        .map(|name| last_path_segment(name.as_str()).to_owned())
        .unwrap_or_else(|| "<unknown>".to_owned());
    let span = SyntaxSpan::from_node(node);

    facts.call_sites.push(SyntaxCallSite {
        kind: "macro".to_owned(),
        callee: format!("{macro_name}!"),
        dynamic: false,
        span,
    });

    if let Some(kind) = risky_macro_kind(macro_name.as_str()) {
        push_rust_risk(kind, node_text(source, node), node, source, facts);
        push_guard_evidence(node, source, facts);
    }
}

fn push_rust_risk(
    kind: &str,
    evidence: &str,
    node: Node<'_>,
    source: &str,
    facts: &mut SyntaxFacts,
) {
    facts.risk_seams.push(SyntaxRiskSeam {
        kind: kind.to_owned(),
        evidence: compact_text(evidence),
        span: SyntaxSpan::from_node(node),
        test_context: is_rust_test_context(node, source),
    });
}

fn push_guard_evidence(node: Node<'_>, source: &str, facts: &mut SyntaxFacts) {
    let Some(guard) = nearest_guard(node) else {
        return;
    };
    facts.risk_seams.push(SyntaxRiskSeam {
        kind: "guard_evidence".to_owned(),
        evidence: compact_text(node_text(source, guard)),
        span: SyntaxSpan::from_node(guard),
        test_context: is_rust_test_context(node, source),
    });
}

fn is_rust_test_context(mut node: Node<'_>, source: &str) -> bool {
    loop {
        if is_rust_test_container(node, source) {
            return true;
        }
        node = match node.parent() {
            Some(parent) => parent,
            None => return false,
        };
    }
}

fn is_rust_test_container(node: Node<'_>, source: &str) -> bool {
    match node.kind() {
        "mod_item" => {
            is_tests_module(node, source)
                || outer_attributes_match(node, source, attribute_text_is_cfg_test)
        }
        "function_item" => {
            outer_attributes_match(node, source, |source, attribute| {
                attribute_text_is_test(source, attribute)
                    || attribute_text_is_cfg_test(source, attribute)
            }) || node_has_test_attribute(node, source)
        }
        _ => false,
    }
}

fn outer_attributes_match(
    node: Node<'_>,
    source: &str,
    predicate: impl Fn(&str, Node<'_>) -> bool,
) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    let mut cursor = parent.walk();
    let mut attributes = Vec::new();
    for child in parent.children(&mut cursor) {
        if child.start_byte() == node.start_byte() && child.kind() == node.kind() {
            return !attributes.is_empty()
                && attributes
                    .iter()
                    .all(|attribute| predicate(source, *attribute));
        }
        if child.kind() == "attribute_item" {
            attributes.push(child);
        } else {
            attributes.clear();
        }
    }
    false
}

fn is_tests_module(node: Node<'_>, source: &str) -> bool {
    node.child_by_field_name("name")
        .is_some_and(|name| node_text(source, name) == "tests")
}

fn node_has_test_attribute(node: Node<'_>, source: &str) -> bool {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .any(|child| child.kind() == "attribute_item" && attribute_text_is_test(source, child))
}

fn attribute_text_is_test(source: &str, attribute: Node<'_>) -> bool {
    let text = compact_text(node_text(source, attribute));
    text == "#[test]" || text.starts_with("#[test(") || text.starts_with("#[test,")
}

fn attribute_text_is_cfg_test(source: &str, attribute: Node<'_>) -> bool {
    let compact: String = node_text(source, attribute).split_whitespace().collect();
    compact.contains("cfg(test)") || compact.contains("cfg(all(test")
}

fn nearest_guard(mut node: Node<'_>) -> Option<Node<'_>> {
    for _ in 0..8 {
        node = node.parent()?;
        if matches!(node.kind(), "if_expression" | "match_expression") {
            return Some(node);
        }
    }
    None
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

fn node_text<'source>(source: &'source str, node: Node<'_>) -> &'source str {
    source.get(node.byte_range()).unwrap_or("")
}

fn node_text_checked<'source>(source: &'source str, node: Node<'_>) -> Option<&'source str> {
    source.get(node.byte_range())
}

fn normalized_node_text_str(node: Node<'_>, source: &str) -> Option<String> {
    node_text_checked(source, node).map(compact_text)
}

fn first_identifier_text(node: Node<'_>, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if matches!(
            child.kind(),
            "identifier" | "type_identifier" | "field_identifier"
        ) {
            return Some(compact_text(node_text(source, child)));
        }
    }
    None
}

fn first_named_child_text(node: Node<'_>, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .next()
        .map(|child| compact_text(node_text(source, child)))
}

fn method_name_from_callee(callee: &str) -> Option<String> {
    callee
        .rsplit_once('.')
        .map(|(_, method)| method.trim().to_owned())
        .filter(|method| !method.is_empty())
}

fn last_path_segment(text: &str) -> &str {
    text.rsplit_once("::").map_or(text, |(_, segment)| segment)
}

fn macro_name_from_text(text: &str) -> Option<String> {
    text.split_once('!')
        .map(|(name, _)| name.trim().to_owned())
        .filter(|name| !name.is_empty())
}

fn compact_text(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() > 120 {
        let shortened = compact.chars().take(117).collect::<String>();
        format!("{shortened}...")
    } else {
        compact
    }
}

fn is_public_surface(node: Node<'_>, source: &str) -> bool {
    let text = node_text(source, node);
    text.contains("pub ")
        || text.contains("pub(")
        || text.contains("extern \"")
        || text.contains("#[no_mangle]")
        || text.contains("#[export_name")
}

fn is_capacity_callee(callee: &str) -> bool {
    callee.ends_with("::with_capacity")
        || callee.ends_with("::reserve")
        || callee.ends_with("::reserve_exact")
        || callee.ends_with("::try_reserve")
        || callee.ends_with("::try_reserve_exact")
        || callee.ends_with(".reserve")
        || callee.ends_with(".reserve_exact")
        || callee.ends_with(".try_reserve")
        || callee.ends_with(".try_reserve_exact")
}

fn is_fallible_conversion_callee(callee: &str) -> bool {
    callee.ends_with("::try_from") || callee.ends_with("::try_into")
}

fn risky_macro_kind(name: &str) -> Option<&'static str> {
    match name {
        "panic" => Some("panic_macro"),
        "assert" | "assert_eq" | "assert_ne" | "debug_assert" | "debug_assert_eq"
        | "debug_assert_ne" => Some("assert_macro"),
        "unreachable" => Some("unreachable_macro"),
        "todo" => Some("todo_macro"),
        "unimplemented" => Some("unimplemented_macro"),
        _ => None,
    }
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
    use super::{RustLandmarkKind, extract_rust_facts, parse_rust_landmarks};
    use tree_sitter::Parser;

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

    #[test]
    fn extracts_call_expression_method_risks_from_locked_grammar() {
        let facts = parse_facts(
            r#"
fn main(value: Option<i32>, count: i64, index: usize) -> i32 {
    let count = usize::try_from(count).expect("count fits");
    let mut values = Vec::with_capacity(count);
    values.reserve(index);
    if index < values.len() {
        return values[index];
    }
    value.unwrap()
}
"#,
        );

        let risk_kinds = facts
            .risk_seams
            .iter()
            .map(|risk| risk.kind.as_str())
            .collect::<Vec<_>>();
        for expected in [
            "fallible_conversion",
            "fallible_conversion_expect",
            "capacity_allocation",
            "indexing",
            "guard_evidence",
            "unwrap",
        ] {
            assert!(risk_kinds.contains(&expected), "{expected}");
        }
    }

    #[test]
    fn normalizes_path_qualified_macro_names() {
        let facts = parse_facts(
            r#"
fn main() {
    std::panic!("boom");
}
"#,
        );

        assert!(facts.call_sites.iter().any(|call| call.callee == "panic!"));
        assert!(
            facts
                .risk_seams
                .iter()
                .any(|risk| risk.kind == "panic_macro")
        );
    }

    #[test]
    fn marks_test_module_assertions_as_test_context() {
        let facts = parse_facts(
            r#"
pub fn production() {
    None.unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn unit_test() {
        assert!(true);
    }
}
"#,
        );

        let production = facts
            .risk_seams
            .iter()
            .find(|risk| risk.kind == "unwrap")
            .expect("production unwrap seam");
        assert!(!production.test_context);

        let assertion = facts
            .risk_seams
            .iter()
            .find(|risk| risk.kind == "assert_macro")
            .expect("test assertion seam");
        assert!(assertion.test_context);
    }

    fn parse_facts(source: &str) -> super::SyntaxFacts {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Rust parser should load");
        let tree = parser.parse(source, None).expect("source should parse");
        extract_rust_facts(tree.root_node(), source)
    }
}
