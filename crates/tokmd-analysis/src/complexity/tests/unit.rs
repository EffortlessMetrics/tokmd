use super::risk::estimate_cyclomatic;
use super::*;

#[test]
fn test_count_rust_functions() {
    let code = r#"
fn simple() {
    println!("hello");
}

pub fn public_fn() {
    let x = 1;
    let y = 2;
}

pub async fn async_fn() {
    todo!()
}
"#;
    let lines: Vec<&str> = code.lines().collect();
    let (count, _max_len) = count_rust_functions(&lines);
    assert_eq!(count, 3);
}

#[test]
fn test_count_python_functions() {
    let code = r#"
def foo():
    pass

async def bar():
    await something()

def baz():
    x = 1
    y = 2
    return x + y
"#;
    let lines: Vec<&str> = code.lines().collect();
    let (count, _max_len) = count_python_functions(&lines);
    assert_eq!(count, 3);
}

#[test]
fn test_estimate_cyclomatic_rust() {
    let code = r#"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            x * 2
        } else {
            x + 1
        }
    } else {
        match x {
            -1 => 0,
            _ => x.abs(),
        }
    }
}
"#;
    let cyclo = estimate_cyclomatic("rust", code);
    // Base 1 + 2 ifs + 1 match = 4
    assert_eq!(cyclo, 4);
}

#[test]
fn test_estimate_cyclomatic_rust_no_else_if_double_count() {
    // "else if" should only count once (as "if"), not as both "if" and "else if"
    let code = r#"
fn branchy(x: i32) -> i32 {
    if x > 0 {
        1
    } else if x < 0 {
        -1
    } else if x == 0 {
        0
    } else {
        42
    }
}
"#;
    let cyclo = estimate_cyclomatic("rust", code);
    // Base 1 + 3 ifs (the initial "if" + 2 "else if" each matched by "if ")
    assert_eq!(cyclo, 4);
}

#[test]
fn test_estimate_cyclomatic_js_no_switch_double_count() {
    // "switch" removed; only "case" contributes
    let code = r#"
function classify(x) {
    switch (x) {
        case 1: return "one";
        case 2: return "two";
        case 3: return "three";
        default: return "other";
    }
}
"#;
    let cyclo = estimate_cyclomatic("javascript", code);
    // Base 1 + 3 cases = 4
    assert_eq!(cyclo, 4);
}

#[test]
fn test_classify_risk() {
    assert_eq!(
        classify_risk_extended(5, 10, 5, None, None),
        ComplexityRisk::Low
    );
    assert_eq!(
        classify_risk_extended(25, 30, 15, None, None),
        ComplexityRisk::Moderate
    );
    assert_eq!(
        classify_risk_extended(30, 60, 25, None, None),
        ComplexityRisk::High
    );
    assert_eq!(
        classify_risk_extended(60, 120, 60, None, None),
        ComplexityRisk::Critical
    );
}

#[test]
fn test_classify_risk_with_cognitive() {
    // Low cognitive should not change low risk
    assert_eq!(
        classify_risk_extended(5, 10, 5, Some(10), Some(2)),
        ComplexityRisk::Low
    );
    // High cognitive should increase risk
    assert!(matches!(
        classify_risk_extended(5, 10, 5, Some(60), Some(6)),
        ComplexityRisk::Moderate | ComplexityRisk::High
    ));
    // High nesting should increase risk
    assert!(matches!(
        classify_risk_extended(5, 10, 5, Some(10), Some(9)),
        ComplexityRisk::Moderate | ComplexityRisk::High
    ));
}

#[test]
fn test_is_complexity_lang() {
    assert!(is_complexity_lang("Rust"));
    assert!(is_complexity_lang("javascript"));
    assert!(is_complexity_lang("Python"));
    assert!(!is_complexity_lang("Markdown"));
    assert!(!is_complexity_lang("JSON"));
}

#[test]
fn test_is_rust_fn_start_extended() {
    // Standard cases
    assert!(is_rust_fn_start("fn foo()"));
    assert!(is_rust_fn_start("pub fn foo()"));
    assert!(is_rust_fn_start("pub(crate) fn foo()"));
    assert!(is_rust_fn_start("pub(super) fn foo()"));
    assert!(is_rust_fn_start("async fn foo()"));
    assert!(is_rust_fn_start("pub async fn foo()"));
    assert!(is_rust_fn_start("unsafe fn foo()"));
    assert!(is_rust_fn_start("const fn foo()"));

    // Extended: pub(in path) visibility
    assert!(is_rust_fn_start("pub(in crate::foo) fn bar()"));
    assert!(is_rust_fn_start("pub(in crate::foo::bar) fn baz()"));

    // Extended: extern "ABI" functions
    assert!(is_rust_fn_start(r#"extern "C" fn callback()"#));
    assert!(is_rust_fn_start(r#"pub extern "C" fn callback()"#));
    assert!(is_rust_fn_start(r#"pub unsafe extern "C" fn callback()"#));

    // Extended: multi-qualifier combos
    assert!(is_rust_fn_start("pub(crate) unsafe async fn baz()"));
    assert!(is_rust_fn_start("pub(super) const fn helper()"));

    // Negative cases
    assert!(!is_rust_fn_start("let fn_name = 5;"));
    assert!(!is_rust_fn_start("// fn foo()"));
    assert!(!is_rust_fn_start("struct Foo {"));
}

#[test]
fn test_detect_fn_rust_qualifiers() {
    let code = r#"
pub(crate) async fn crate_async() {
    todo!()
}

pub(super) async fn super_async() {
    todo!()
}

pub(crate) unsafe fn crate_unsafe() {
    todo!()
}

pub unsafe fn public_unsafe() {
    todo!()
}

pub(crate) const fn crate_const() -> u32 {
    42
}

pub const fn public_const() -> u32 {
    0
}
"#;
    let lines: Vec<&str> = code.lines().collect();
    let spans = detect_fn_spans_rust(&lines);
    let names: Vec<&str> = spans.iter().map(|(_, _, n)| n.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "crate_async",
            "super_async",
            "crate_unsafe",
            "public_unsafe",
            "crate_const",
            "public_const",
        ]
    );

    // Also verify count_rust_functions picks them all up
    let (count, _) = count_rust_functions(&lines);
    assert_eq!(count, 6);
}

#[test]
fn test_detect_fn_python_decorators() {
    let code = r#"
@staticmethod
def plain_static():
    pass

@app.route("/")
@login_required
def index():
    return "hello"

def no_decorator():
    pass
"#;
    let lines: Vec<&str> = code.lines().collect();
    let spans = detect_fn_spans_python(&lines);
    assert_eq!(spans.len(), 3);

    // First function: @staticmethod + def plain_static
    let (start, _end, ref name) = spans[0];
    assert_eq!(name, "plain_static");
    // The span should start at the decorator line
    assert!(lines[start].trim().starts_with('@'));

    // Second function: two decorators + def index
    let (start2, _end2, ref name2) = spans[1];
    assert_eq!(name2, "index");
    assert!(lines[start2].trim().starts_with('@'));

    // Third function: no decorator
    let (start3, _end3, ref name3) = spans[2];
    assert_eq!(name3, "no_decorator");
    assert!(lines[start3].trim().starts_with("def "));
}

#[test]
fn test_detect_fn_c_style_no_preprocessor() {
    let code = r#"
#define THING(x) { }
#define MACRO(a, b) { a + b; }

int main(int argc, char** argv) {
    return 0;
}
"#;
    let lines: Vec<&str> = code.lines().collect();
    let spans = detect_fn_spans_c_style(&lines);
    // Should only detect main, not #define macros
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].2, "main");
}

#[test]
fn test_compute_technical_debt_ratio() {
    let export = ExportData {
        rows: vec![FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 1000,
            comments: 0,
            blanks: 0,
            lines: 1000,
            bytes: 1000,
            tokens: 250,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: tokmd_types::ChildIncludeMode::Separate,
    };

    let files = vec![FileComplexity {
        path: "src/lib.rs".to_string(),
        module: "src".to_string(),
        function_count: 3,
        max_function_length: 20,
        cyclomatic_complexity: 12,
        cognitive_complexity: Some(8),
        max_nesting: Some(2),
        risk_level: ComplexityRisk::Moderate,
        functions: None,
    }];

    let debt = compute_technical_debt_ratio(&export, &files).expect("debt ratio");
    assert_eq!(debt.complexity_points, 20);
    assert!((debt.ratio - 20.0).abs() < f64::EPSILON);
    assert!((debt.code_kloc - 1.0).abs() < f64::EPSILON);
    assert_eq!(debt.level, TechnicalDebtLevel::Low);
}

#[test]
fn test_compute_technical_debt_ratio_none_for_zero_code() {
    let export = ExportData {
        rows: vec![FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 0,
            comments: 0,
            blanks: 0,
            lines: 0,
            bytes: 0,
            tokens: 0,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: tokmd_types::ChildIncludeMode::Separate,
    };

    let files = vec![FileComplexity {
        path: "src/lib.rs".to_string(),
        module: "src".to_string(),
        function_count: 1,
        max_function_length: 1,
        cyclomatic_complexity: 1,
        cognitive_complexity: Some(1),
        max_nesting: Some(1),
        risk_level: ComplexityRisk::Low,
        functions: None,
    }];

    assert!(compute_technical_debt_ratio(&export, &files).is_none());
}

#[test]
fn bounded_complexity_warnings_report_default_file_cap() {
    let export = ExportData {
        rows: vec![FileRow {
            path: "src/lib.rs".to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 40_000,
            comments: 0,
            blanks: 0,
            lines: 40_000,
            bytes: DEFAULT_MAX_FILE_BYTES as usize + 4096,
            tokens: 10_000,
        }],
        module_roots: vec![],
        module_depth: 1,
        children: tokmd_types::ChildIncludeMode::Separate,
    };
    let warnings = bounded_complexity_warnings(
        Path::new("."),
        &[PathBuf::from("src/lib.rs")],
        &export,
        &AnalysisLimits::default(),
    );

    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("default max_file_bytes=131072")),
        "{warnings:?}"
    );
    assert!(
        warnings
            .iter()
            .any(|warning| warning.contains("complexity metrics are partial")),
        "{warnings:?}"
    );
}

#[test]
fn test_detect_fn_python_decorators_extended() {
    let code = r#"
@app.route("/")
# This is a comment between decorators
@login_required

# Another comment
def index():
    return "hello"

@nested_decorator
# Indented comment
def nested():
    pass
"#;
    let lines: Vec<&str> = code.lines().collect();
    let spans = detect_fn_spans_python(&lines);
    assert_eq!(spans.len(), 2);

    // First function: index
    let (start, _end, ref name) = spans[0];
    assert_eq!(name, "index");
    // Should start at @app.route
    assert!(lines[start].trim().starts_with("@app.route"));

    // Second function: nested
    let (start2, _end2, ref name2) = spans[1];
    assert_eq!(name2, "nested");
    // Should start at @nested_decorator
    assert!(lines[start2].trim().starts_with("@nested_decorator"));
}

#[test]
fn test_detect_fn_go_basic() {
    let code = r#"
package main

func main() {
    println("hello")
}

func add(a int, b int) int {
    return a + b
}
"#;
    let lines: Vec<&str> = code.lines().collect();
    let spans = detect_fn_spans_go(&lines);
    let names: Vec<&str> = spans.iter().map(|(_, _, n)| n.as_str()).collect();
    assert_eq!(names, vec!["main", "add"]);
}

#[test]
fn test_detect_fn_go_with_receiver() {
    let code = r#"
type T struct{}

func (t *T) Method() string {
    return "method"
}

func (T) ValueMethod() int {
    return 0
}
"#;
    let lines: Vec<&str> = code.lines().collect();
    let spans = detect_fn_spans_go(&lines);
    let names: Vec<&str> = spans.iter().map(|(_, _, n)| n.as_str()).collect();
    assert_eq!(names, vec!["Method", "ValueMethod"]);
}

#[test]
fn test_detect_fn_go_unknown_name() {
    // Malformed: func keyword with no identifier; brace-end search still skips it.
    let code = "func\n";
    let lines: Vec<&str> = code.lines().collect();
    let spans = detect_fn_spans_go(&lines);
    assert!(spans.is_empty(), "no brace, no span");
}

#[test]
fn test_detect_fn_go_open_brace_only_advances() {
    // Function header with no matching close brace should be skipped without
    // advancing into an infinite loop.
    let code = r#"
func incomplete() {
"#;
    let lines: Vec<&str> = code.lines().collect();
    let spans = detect_fn_spans_go(&lines);
    assert!(spans.is_empty());
}

#[test]
fn test_detect_fn_js_basic() {
    let code = r#"
function foo() {
    return 1;
}

async function bar() {
    return 2;
}

export function baz() {
    return 3;
}

export async function qux() {
    return 4;
}
"#;
    let lines: Vec<&str> = code.lines().collect();
    let spans = detect_fn_spans_js(&lines);
    let names: Vec<&str> = spans.iter().map(|(_, _, n)| n.as_str()).collect();
    assert_eq!(names, vec!["foo", "bar", "baz", "qux"]);
}

#[test]
fn test_detect_fn_js_arrow_with_brace() {
    let code = r#"
const greet = (name) => {
    return "hi " + name;
};

const noop = () => {};
"#;
    let lines: Vec<&str> = code.lines().collect();
    let spans = detect_fn_spans_js(&lines);
    // Both arrow functions should be detected; names come from text before '('.
    assert_eq!(spans.len(), 2);
}

#[test]
fn test_detect_fn_js_skips_line_comment() {
    let code = r#"
// function commentedOut() { return 1; }
function real() {
    return 1;
}
"#;
    let lines: Vec<&str> = code.lines().collect();
    let spans = detect_fn_spans_js(&lines);
    let names: Vec<&str> = spans.iter().map(|(_, _, n)| n.as_str()).collect();
    assert_eq!(names, vec!["real"]);
}

#[test]
fn test_detect_fn_js_anonymous_fallback() {
    // `(...) => { ... }` with no identifier before `(` is anonymous.
    let code = "((x) => {\n  return x;\n})(1);\n";
    let lines: Vec<&str> = code.lines().collect();
    let spans = detect_fn_spans_js(&lines);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].2, "<anonymous>");
}

#[test]
fn test_detect_fn_js_dollar_underscore_names() {
    let code = "function $foo_bar() {\n  return 1;\n}\n";
    let lines: Vec<&str> = code.lines().collect();
    let spans = detect_fn_spans_js(&lines);
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].2, "$foo_bar");
}

#[test]
fn test_extract_function_details_dispatches_languages() {
    use super::details::extract_function_details;

    // Each language should produce at least one detail entry, exercising the
    // language-specific dispatcher in extract_function_details.
    let rust_code = "pub fn foo() -> i32 {\n  if true { 1 } else { 2 }\n}\n";
    assert!(!extract_function_details("rust", rust_code).is_empty());

    let js_code = "function foo() {\n  return 1;\n}\n";
    assert!(!extract_function_details("javascript", js_code).is_empty());
    assert!(!extract_function_details("typescript", js_code).is_empty());

    let py_code = "def foo():\n    return 1\n";
    assert!(!extract_function_details("python", py_code).is_empty());

    let go_code = "func main() {\n  println(\"hi\")\n}\n";
    assert!(!extract_function_details("go", go_code).is_empty());

    let c_code = "int main() {\n  return 0;\n}\n";
    assert!(!extract_function_details("c", c_code).is_empty());

    // Unknown language returns no details (matches the `_ => Vec::new()` arm).
    assert!(extract_function_details("brainfuck", "+++.\n").is_empty());
}
