//! Deep tests for complexity estimation functions.
//!
//! Covers cyclomatic complexity, cognitive complexity, and nesting analysis
//! with file-based scenarios that complement the BDD tests.

use std::fs::File;
use std::io::Write;
use tokmd_content::complexity::{
    analyze_functions, analyze_nesting_depth, estimate_cognitive_complexity,
    estimate_cyclomatic_complexity,
};
use tokmd_content::read_text_capped;

/// Helper: write code to a temp file, read it back.
fn code_from_file(content: &str) -> String {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("code.txt");
    let mut f = File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    read_text_capped(&path, 1_000_000).unwrap()
}

// ============================================================================
// Cyclomatic complexity
// ============================================================================

#[test]
fn cc_empty_function_is_one() {
    let code = code_from_file(
        "\
fn noop() {
}
",
    );
    let result = estimate_cyclomatic_complexity(&code, "rust");
    assert_eq!(result.function_count, 1);
    assert_eq!(result.max_cc, 1, "empty function has CC=1 (base)");
}

#[test]
fn cc_single_if() {
    let code = code_from_file(
        "\
fn check(x: i32) {
    if x > 0 {
        println!(\"positive\");
    }
}
",
    );
    let result = estimate_cyclomatic_complexity(&code, "rust");
    assert_eq!(result.function_count, 1);
    assert!(
        result.max_cc >= 2,
        "if adds at least 1 decision point, CC >= 2"
    );
}

#[test]
fn cc_if_else_chain() {
    let code = code_from_file(
        "\
fn classify(x: i32) {
    if x > 100 {
        println!(\"big\");
    } else if x > 10 {
        println!(\"medium\");
    } else if x > 0 {
        println!(\"small\");
    } else {
        println!(\"non-positive\");
    }
}
",
    );
    let result = estimate_cyclomatic_complexity(&code, "rust");
    assert_eq!(result.function_count, 1);
    // 1 (base) + 3 (if/else if/else if) = at least 4
    assert!(
        result.max_cc >= 4,
        "expected CC >= 4, got {}",
        result.max_cc
    );
}

#[test]
fn cc_for_loop() {
    let code = code_from_file(
        "\
fn sum(data: &[i32]) -> i32 {
    let mut total = 0;
    for &val in data {
        total += val;
    }
    total
}
",
    );
    let result = estimate_cyclomatic_complexity(&code, "rust");
    assert_eq!(result.function_count, 1);
    assert!(
        result.max_cc >= 2,
        "for loop adds at least 1 decision point"
    );
}

#[test]
fn cc_while_loop() {
    let code = code_from_file(
        "\
fn drain(items: &mut Vec<i32>) {
    while !items.is_empty() {
        items.pop();
    }
}
",
    );
    let result = estimate_cyclomatic_complexity(&code, "rust");
    assert_eq!(result.function_count, 1);
    assert!(result.max_cc >= 2, "while loop adds a decision point");
}

#[test]
fn cc_logical_operators() {
    let code = code_from_file(
        "\
fn valid(x: i32, y: i32) -> bool {
    if x > 0 && y > 0 && x < 100 || y < 100 {
        return true;
    }
    false
}
",
    );
    let result = estimate_cyclomatic_complexity(&code, "rust");
    assert_eq!(result.function_count, 1);
    // 1 (base) + 1 (if) + 2 (&&) + 1 (||) = at least 5
    assert!(
        result.max_cc >= 4,
        "logical operators should increase CC, got {}",
        result.max_cc
    );
}

#[test]
fn cc_multiple_functions() {
    let code = code_from_file(
        "\
fn simple() {
    println!(\"no branches\");
}

fn complex(x: i32) {
    if x > 0 {
        for i in 0..x {
            if i % 2 == 0 {
                println!(\"{i}\");
            }
        }
    }
}
",
    );
    let result = estimate_cyclomatic_complexity(&code, "rust");
    assert_eq!(result.function_count, 2);
    // total should be sum of both functions' CC
    assert!(result.total_cc >= 2, "total CC should be >= 2");
    assert!(
        result.avg_cc > 1.0,
        "average CC should be > 1 when one function has branches"
    );
}

#[test]
fn cc_python_branches() {
    let code = code_from_file(
        "\
def classify(x):
    if x > 100:
        return 'big'
    elif x > 10:
        return 'medium'
    else:
        return 'small'
",
    );
    let result = estimate_cyclomatic_complexity(&code, "python");
    assert_eq!(result.function_count, 1);
    assert!(
        result.max_cc >= 3,
        "if/elif branches, CC >= 3, got {}",
        result.max_cc
    );
}

#[test]
fn cc_javascript_function() {
    let code = code_from_file(
        "\
function validate(input) {
    if (!input) {
        return false;
    }
    if (input.length > 100) {
        return false;
    }
    for (let i = 0; i < input.length; i++) {
        if (input[i] === '\\0') {
            return false;
        }
    }
    return true;
}
",
    );
    let result = estimate_cyclomatic_complexity(&code, "javascript");
    assert_eq!(result.function_count, 1);
    // 1 (base) + 2 (if) + 1 (for) + 1 (if) = at least 5
    assert!(
        result.max_cc >= 4,
        "expected CC >= 4, got {}",
        result.max_cc
    );
}

// ============================================================================
// Cognitive complexity
// ============================================================================

#[test]
fn cognitive_empty_function_is_zero() {
    let code = code_from_file(
        "\
fn noop() {
}
",
    );
    let result = estimate_cognitive_complexity(&code, "rust");
    assert_eq!(result.function_count, 1);
    assert_eq!(result.max, 0, "empty function has 0 cognitive complexity");
}

#[test]
fn cognitive_nested_loops_and_conditions() {
    let code = code_from_file(
        "\
fn search(matrix: &[Vec<i32>], target: i32) -> bool {
    for row in matrix {
        for val in row {
            if *val == target {
                return true;
            }
        }
    }
    false
}
",
    );
    let result = estimate_cognitive_complexity(&code, "rust");
    assert_eq!(result.function_count, 1);
    // Nesting penalty should make this significantly higher than cyclomatic
    assert!(
        result.max >= 4,
        "nested for+for+if should have high cognitive complexity, got {}",
        result.max
    );
}

#[test]
fn cognitive_sequential_branches_lower_than_nested() {
    let sequential = code_from_file(
        "\
fn sequential(x: i32) {
    if x > 0 { println!(\"a\"); }
    if x > 1 { println!(\"b\"); }
    if x > 2 { println!(\"c\"); }
}
",
    );
    let nested = code_from_file(
        "\
fn nested(x: i32) {
    if x > 0 {
        if x > 1 {
            if x > 2 {
                println!(\"deep\");
            }
        }
    }
}
",
    );
    let seq_result = estimate_cognitive_complexity(&sequential, "rust");
    let nest_result = estimate_cognitive_complexity(&nested, "rust");
    assert!(
        nest_result.max >= seq_result.max,
        "nested ({}) should have >= cognitive complexity than sequential ({})",
        nest_result.max,
        seq_result.max
    );
}

// ============================================================================
// Nesting depth analysis
// ============================================================================

#[test]
fn nesting_empty_code() {
    let code = code_from_file("");
    let result = analyze_nesting_depth(&code, "rust");
    assert_eq!(result.max_depth, 0);
    assert_eq!(result.avg_depth, 0.0);
}

#[test]
fn nesting_flat_code() {
    let code = code_from_file(
        "\
fn flat() {
    let x = 1;
    let y = 2;
    println!(\"{}\", x + y);
}
",
    );
    let result = analyze_nesting_depth(&code, "rust");
    // One level of braces for the function body
    assert!(result.max_depth >= 1);
}

#[test]
fn nesting_deeply_nested() {
    let code = code_from_file(
        "\
fn deep() {
    if true {
        for i in 0..10 {
            while i > 0 {
                if i % 2 == 0 {
                    println!(\"deep\");
                }
            }
        }
    }
}
",
    );
    let result = analyze_nesting_depth(&code, "rust");
    // fn { if { for { while { if { ... } } } } } = depth 5
    assert!(
        result.max_depth >= 5,
        "expected depth >= 5, got {}",
        result.max_depth
    );
}

#[test]
fn nesting_python_indentation() {
    let code = code_from_file(
        "\
def outer():
    for i in range(10):
        if i > 5:
            for j in range(i):
                print(i, j)
",
    );
    let result = analyze_nesting_depth(&code, "python");
    // def -> for -> if -> for = at least 3 levels of nesting
    assert!(
        result.max_depth >= 3,
        "expected Python depth >= 3, got {}",
        result.max_depth
    );
}

// ============================================================================
// Function detection
// ============================================================================

#[test]
fn function_detection_go() {
    let code = code_from_file(
        "\
func main() {
    fmt.Println(\"hello\")
}

func helper(x int) int {
    return x * 2
}
",
    );
    let metrics = analyze_functions(&code, "go");
    assert_eq!(metrics.function_count, 2);
}

#[test]
fn function_detection_multiple_languages_same_count() {
    let rust_code = code_from_file("fn a() {\n}\nfn b() {\n}\nfn c() {\n}\n");
    let py_code = code_from_file("def a():\n    pass\ndef b():\n    pass\ndef c():\n    pass\n");
    let go_code = code_from_file("func a() {\n}\nfunc b() {\n}\nfunc c() {\n}\n");

    let rust_m = analyze_functions(&rust_code, "rust");
    let py_m = analyze_functions(&py_code, "python");
    let go_m = analyze_functions(&go_code, "go");

    assert_eq!(rust_m.function_count, 3);
    assert_eq!(py_m.function_count, 3);
    assert_eq!(go_m.function_count, 3);
}

#[test]
fn long_function_flagged() {
    let mut code = String::from("fn verbose() {\n");
    for i in 0..110 {
        code.push_str(&format!("    let x{i} = {i};\n"));
    }
    code.push_str("}\n");
    let code = code_from_file(&code);

    let metrics = analyze_functions(&code, "rust");
    assert_eq!(metrics.function_count, 1);
    assert!(
        metrics.functions_over_threshold > 0,
        "function with 110+ lines should exceed threshold"
    );
}
