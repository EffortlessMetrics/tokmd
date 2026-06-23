use super::facts::{
    SyntaxCallSite, SyntaxExport, SyntaxFacts, SyntaxImport, SyntaxRiskSeam, SyntaxSpan,
    SyntaxSymbol,
};
use tree_sitter::Node;

#[must_use]
pub fn extract_python_facts(root: Node<'_>, source: &str) -> SyntaxFacts {
    let mut facts = SyntaxFacts::default();
    push_module_symbol(root, &mut facts);
    visit_node(root, source, &mut facts);
    facts.normalize();
    facts
}

fn visit_node(node: Node<'_>, source: &str, facts: &mut SyntaxFacts) {
    match node.kind() {
        "class_definition" => push_named_symbol(node, source, "class", facts),
        "function_definition" => push_named_symbol(node, source, "function", facts),
        "import_statement" | "import_from_statement" => push_import(node, source, facts),
        "call" => push_call(node, source, facts),
        "if_statement" => push_entrypoint_if(node, source, facts),
        "raise_statement" => push_risk("exception_raise", node_text(source, node), node, facts),
        "except_clause" => push_risk("exception_handler", node_text(source, node), node, facts),
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit_node(child, source, facts);
    }
}

fn push_module_symbol(root: Node<'_>, facts: &mut SyntaxFacts) {
    facts.symbols.push(SyntaxSymbol {
        kind: "module".to_owned(),
        name: "<module>".to_owned(),
        span: SyntaxSpan::from_node(root),
        exported: true,
        public_surface: true,
    });
}

fn push_named_symbol(node: Node<'_>, source: &str, kind: &str, facts: &mut SyntaxFacts) {
    let name = node
        .child_by_field_name("name")
        .and_then(|name| node_text_checked(source, name))
        .map(compact_text)
        .or_else(|| first_identifier_text(node, source))
        .unwrap_or_else(|| "<anonymous>".to_owned());
    let public_surface = !name.starts_with('_') || looks_native_or_boundary(&name);
    let span = SyntaxSpan::from_node(node);

    facts.symbols.push(SyntaxSymbol {
        kind: kind.to_owned(),
        name: name.clone(),
        span,
        exported: public_surface,
        public_surface,
    });

    if public_surface {
        facts.exports.push(SyntaxExport {
            kind: kind.to_owned(),
            name: name.clone(),
            span,
        });
    }

    if looks_native_or_boundary(&name) || looks_native_or_boundary(node_text(source, node)) {
        push_risk("native_boundary_hint", name.as_str(), node, facts);
    }
}

fn push_import(node: Node<'_>, source: &str, facts: &mut SyntaxFacts) {
    let text = compact_text(node_text(source, node));
    let (kind, module, imported) = if let Some(rest) = text.strip_prefix("from ") {
        let (module, imported) = rest.split_once(" import ").unwrap_or((rest, ""));
        (
            "from_import",
            Some(module.trim().to_owned()),
            split_imported_names(imported),
        )
    } else {
        (
            "import",
            None,
            split_imported_names(text.strip_prefix("import ").unwrap_or(&text)),
        )
    };
    let span = SyntaxSpan::from_node(node);
    let native_import = module.as_deref().is_some_and(looks_native_or_boundary)
        || imported
            .iter()
            .any(|name| looks_native_or_boundary(name.as_str()));

    facts.imports.push(SyntaxImport {
        kind: kind.to_owned(),
        module,
        imported: imported.clone(),
        dynamic: false,
        span,
    });

    if native_import {
        push_risk("native_boundary_hint", text.as_str(), node, facts);
    }
}

fn push_call(node: Node<'_>, source: &str, facts: &mut SyntaxFacts) {
    let callee = node
        .child_by_field_name("function")
        .and_then(|function| node_text_checked(source, function))
        .map(compact_text)
        .or_else(|| first_named_child_text(node, source))
        .unwrap_or_else(|| "<unknown>".to_owned());
    let span = SyntaxSpan::from_node(node);
    let dynamic = is_dynamic_callee(&callee);

    facts.call_sites.push(SyntaxCallSite {
        kind: "call".to_owned(),
        callee: callee.clone(),
        dynamic,
        span,
    });

    if is_entrypoint_callee(&callee) {
        push_risk("entrypoint", callee.as_str(), node, facts);
    }
    if is_subprocess_callee(&callee) {
        push_risk("subprocess_call", callee.as_str(), node, facts);
        push_guard_evidence(node, source, facts);
    }
    if is_dynamic_import_callee(&callee) {
        push_risk("dynamic_import", callee.as_str(), node, facts);
        push_guard_evidence(node, source, facts);
    }
    if is_eval_callee(&callee) {
        push_risk("dynamic_eval", callee.as_str(), node, facts);
        push_guard_evidence(node, source, facts);
    }
    if dynamic {
        push_risk("dynamic_call", callee.as_str(), node, facts);
        push_guard_evidence(node, source, facts);
    }
    if is_file_io_callee(&callee) {
        push_risk("file_io", callee.as_str(), node, facts);
        push_guard_evidence(node, source, facts);
    }
    if looks_native_or_boundary(&callee) || looks_native_or_boundary(node_text(source, node)) {
        push_risk("native_boundary_hint", callee.as_str(), node, facts);
    }
}

fn push_entrypoint_if(node: Node<'_>, source: &str, facts: &mut SyntaxFacts) {
    let text = node_text(source, node);
    if text.contains("__name__") && text.contains("__main__") {
        push_risk("entrypoint", "__name__ == \"__main__\"", node, facts);
    }
}

fn push_risk(kind: &str, evidence: &str, node: Node<'_>, facts: &mut SyntaxFacts) {
    facts.risk_seams.push(SyntaxRiskSeam {
        kind: kind.to_owned(),
        evidence: compact_text(evidence),
        span: SyntaxSpan::from_node(node),
        test_context: false,
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
        test_context: false,
    });
}

fn nearest_guard(mut node: Node<'_>) -> Option<Node<'_>> {
    for _ in 0..8 {
        node = node.parent()?;
        if matches!(
            node.kind(),
            "if_statement" | "try_statement" | "with_statement"
        ) {
            return Some(node);
        }
    }
    None
}

fn node_text<'source>(source: &'source str, node: Node<'_>) -> &'source str {
    source.get(node.byte_range()).unwrap_or("")
}

fn node_text_checked<'source>(source: &'source str, node: Node<'_>) -> Option<&'source str> {
    source.get(node.byte_range())
}

fn first_identifier_text(node: Node<'_>, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if matches!(child.kind(), "identifier") {
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

fn split_imported_names(text: &str) -> Vec<String> {
    let mut names = text
        .split(',')
        .filter_map(|name| {
            let name = name.trim();
            (!name.is_empty()).then(|| compact_text(name))
        })
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
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

fn is_entrypoint_callee(callee: &str) -> bool {
    matches!(callee, "main" | "app.run" | "asyncio.run")
}

fn is_subprocess_callee(callee: &str) -> bool {
    callee == "subprocess.run"
        || callee == "subprocess.Popen"
        || callee == "os.system"
        || callee == "os.popen"
}

fn is_dynamic_import_callee(callee: &str) -> bool {
    matches!(callee, "__import__" | "importlib.import_module")
}

fn is_eval_callee(callee: &str) -> bool {
    matches!(callee, "eval" | "exec" | "compile")
}

fn is_file_io_callee(callee: &str) -> bool {
    matches!(callee, "open" | "io.open" | "Path.open") || callee.ends_with(".open")
}

fn is_dynamic_callee(callee: &str) -> bool {
    is_eval_callee(callee)
        || is_dynamic_import_callee(callee)
        || callee.contains('[')
        || callee == "getattr"
}

fn looks_native_or_boundary(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("ctypes")
        || lower.contains("cffi")
        || lower
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .any(|part| part == "ffi")
        || lower.contains("native")
        || lower.contains("cdll")
        || lower.contains("pydll")
}

#[cfg(test)]
mod tests {
    use super::extract_python_facts;
    use tree_sitter::Parser;

    #[test]
    fn extracts_dynamic_and_native_python_seams() {
        let facts = parse_facts(
            r#"
import ctypes
from ctypes import c_void_p
import importlib
import subprocess

class NativeBridge:
    def call(self, name):
        if name:
            return importlib.import_module(name)
        raise RuntimeError("missing name")

def main(command):
    try:
        with open("/tmp/tokmd.log", "w") as handle:
            handle.write("run")
        subprocess.run(command, check=True)
    except OSError:
        return eval(command)
"#,
        );

        assert!(
            facts
                .symbols
                .iter()
                .any(|symbol| symbol.kind == "module" && symbol.name == "<module>")
        );
        assert!(
            facts
                .symbols
                .iter()
                .any(|symbol| symbol.kind == "class" && symbol.name == "NativeBridge")
        );
        assert!(
            facts
                .imports
                .iter()
                .any(|import| import.imported.iter().any(|name| name == "ctypes"))
        );
        assert!(
            facts
                .call_sites
                .iter()
                .any(|call| call.callee == "subprocess.run")
        );

        let risk_kinds = facts
            .risk_seams
            .iter()
            .map(|risk| risk.kind.as_str())
            .collect::<Vec<_>>();
        for expected in [
            "native_boundary_hint",
            "dynamic_import",
            "dynamic_call",
            "subprocess_call",
            "dynamic_eval",
            "file_io",
            "exception_raise",
            "exception_handler",
            "guard_evidence",
        ] {
            assert!(risk_kinds.contains(&expected), "{expected}");
        }
    }

    fn parse_facts(source: &str) -> super::SyntaxFacts {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Python parser should load");
        let tree = parser.parse(source, None).expect("source should parse");
        extract_python_facts(tree.root_node(), source)
    }
}
