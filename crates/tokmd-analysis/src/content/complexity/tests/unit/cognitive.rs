use super::super::estimate_cognitive_complexity;

// ============================================================================
// Cognitive Complexity Tests
// ============================================================================

#[test]
fn cognitive_empty_content() {
    let result = estimate_cognitive_complexity("", "rust");
    assert_eq!(result.function_count, 0);
    assert_eq!(result.total, 0);
    assert_eq!(result.max, 0);
    assert_eq!(result.avg, 0.0);
}

#[test]
fn cognitive_unsupported_language() {
    let result = estimate_cognitive_complexity("some code", "unknown_lang");
    assert_eq!(result.function_count, 0);
}

#[test]
fn cognitive_rust_simple_function() {
    let code = r#"
fn hello() {
    println!("Hello, world!");
}
"#;
    let result = estimate_cognitive_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    assert_eq!(result.total, 0); // No control structures
}

#[test]
fn cognitive_rust_nested_if() {
    // Cognitive complexity adds nesting penalty
    // if x > 0: +1 (nesting 0)
    // if x > 10: +1 + 1 (nesting 1) = +2
    // if x > 100: +1 + 2 (nesting 2) = +3
    // Total: 1 + 2 + 3 = 6
    let code = r#"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                return x * 2;
            }
        }
    }
    0
}
"#;
    let result = estimate_cognitive_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // Should have nesting penalty
    assert!(
        result.total >= 3,
        "Expected cognitive >= 3, got {}",
        result.total
    );
}

#[test]
fn cognitive_rust_loops_with_nesting() {
    let code = r#"
fn process() {
    for i in 0..10 {
        while i > 0 {
            loop {
                break;
            }
        }
    }
}
"#;
    let result = estimate_cognitive_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // for: +1, while: +1+1=+2, loop: +1+2=+3 = 6 total
    assert!(
        result.total >= 3,
        "Expected cognitive >= 3, got {}",
        result.total
    );
}

#[test]
fn cognitive_rust_logical_sequence() {
    let code = r#"
fn check(a: bool, b: bool, c: bool, d: bool) {
    if a && b && c || d {
        println!("complex");
    }
}
"#;
    let result = estimate_cognitive_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // if: +1, logical sequence: +1
    assert!(result.total >= 2);
}

#[test]
fn cognitive_rust_labeled_break() {
    let code = r#"
fn labeled() {
    'outer: for i in 0..10 {
        for j in 0..10 {
            if j == 5 {
                break 'outer;
            }
        }
    }
}
"#;
    let result = estimate_cognitive_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // for: +1, for: +2, if: +3, break 'outer: +1
    assert!(
        result.total >= 4,
        "Expected cognitive >= 4, got {}",
        result.total
    );
}

#[test]
fn cognitive_python_nested() {
    let code = r#"
def complex():
    if True:
        for i in range(10):
            while True:
                break
"#;
    let result = estimate_cognitive_complexity(code, "python");
    assert_eq!(result.function_count, 1);
    assert!(result.total >= 3);
}

#[test]
fn cognitive_js_nested() {
    let code = r#"
function complex() {
    if (true) {
        for (let i = 0; i < 10; i++) {
            while (true) {
                break;
            }
        }
    }
}
"#;
    let result = estimate_cognitive_complexity(code, "javascript");
    assert_eq!(result.function_count, 1);
    assert!(result.total >= 3);
}

#[test]
fn cognitive_high_complexity_detection() {
    // Create a function with high cognitive complexity (> 15)
    let code = r#"
fn very_complex(x: i32) -> i32 {
    if x > 0 {
        if x > 1 {
            if x > 2 {
                if x > 3 {
                    if x > 4 {
                        if x > 5 {
                            if x > 6 {
                                return x;
                            }
                        }
                    }
                }
            }
        }
    }
    0
}
"#;
    let result = estimate_cognitive_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // Deep nesting should produce high cognitive complexity
    assert!(
        result.max > 10,
        "Expected high cognitive, got {}",
        result.max
    );
}

#[test]
fn cognitive_multiple_functions() {
    let code = r#"
fn simple() {
    println!("simple");
}

fn moderate() {
    if true {
        for i in 0..5 {
            println!("{}", i);
        }
    }
}
"#;
    let result = estimate_cognitive_complexity(code, "rust");
    assert_eq!(result.function_count, 2);
    assert!(result.avg > 0.0);
}
