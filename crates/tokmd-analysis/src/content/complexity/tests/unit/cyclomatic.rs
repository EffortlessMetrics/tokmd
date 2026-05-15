use super::super::estimate_cyclomatic_complexity;

// ============================================================================
// Cyclomatic Complexity Tests
// ============================================================================

// ========================
// Basic functionality
// ========================

#[test]
fn cc_empty_content() {
    let result = estimate_cyclomatic_complexity("", "rust");
    assert_eq!(result.function_count, 0);
    assert_eq!(result.total_cc, 0);
    assert_eq!(result.max_cc, 0);
    assert_eq!(result.avg_cc, 0.0);
}

#[test]
fn cc_unsupported_language() {
    let result = estimate_cyclomatic_complexity("some code", "unknown_lang");
    assert_eq!(result.function_count, 0);
    assert_eq!(result.total_cc, 0);
}

#[test]
fn cc_no_functions() {
    let rust_code = r#"
        // Just comments
        const X: i32 = 42;
        "#;
    let result = estimate_cyclomatic_complexity(rust_code, "rust");
    assert_eq!(result.function_count, 0);
}

// ========================
// Rust cyclomatic complexity tests
// ========================

#[test]
fn cc_rust_simple_function() {
    let code = r#"
fn hello() {
    println!("Hello, world!");
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    assert_eq!(result.total_cc, 1); // Base complexity only
}

#[test]
fn cc_rust_if_statement() {
    let code = r#"
fn check(x: i32) {
    if x > 0 {
        println!("positive");
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    assert_eq!(result.total_cc, 2); // 1 base + 1 if
}

#[test]
fn cc_rust_if_else_if() {
    let code = r#"
fn check(x: i32) {
    if x > 0 {
        println!("positive");
    } else if x < 0 {
        println!("negative");
    } else {
        println!("zero");
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 if + 1 else if = 3
    assert_eq!(result.total_cc, 3);
}

#[test]
fn cc_rust_match_statement() {
    let code = r#"
fn classify(x: i32) -> &'static str {
    match x {
        0 => "zero",
        1..=10 => "small",
        _ => "large",
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 match + 3 arms (=>) = 5
    assert!(result.total_cc >= 4);
}

#[test]
fn cc_rust_loops() {
    let code = r#"
fn loops() {
    for i in 0..10 {
        println!("{}", i);
    }
    while true {
        break;
    }
    loop {
        break;
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 for + 1 while + 1 loop = 4
    assert_eq!(result.total_cc, 4);
}

#[test]
fn cc_rust_logical_operators() {
    let code = r#"
fn check(a: bool, b: bool, c: bool) {
    if a && b || c {
        println!("complex");
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 if + 1 && + 1 || = 4
    assert_eq!(result.total_cc, 4);
}

#[test]
fn cc_rust_try_operator() {
    let code = r#"
fn fallible() -> Result<(), Error> {
    let x = something()?;
    let y = another()?;
    Ok(())
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // 1 base + 2 try operators = 3
    assert_eq!(result.total_cc, 3);
}

#[test]
fn cc_rust_multiple_functions() {
    let code = r#"
fn simple() {
    println!("simple");
}

fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            x * 2
        } else {
            x
        }
    } else {
        0
    }
}

pub fn another() {
    for i in 0..5 {
        println!("{}", i);
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 3);
    // simple: 1, complex: 1+2 if = 3, another: 1+1 for = 2
    // Total should be at least 6
    assert!(result.total_cc >= 6);
    assert!(result.max_cc >= 3);
}

#[test]
fn cc_rust_pub_async_fn() {
    let code = r#"
pub async fn fetch_data() {
    if let Some(data) = get_data().await {
        println!("{:?}", data);
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 if = 2
    assert_eq!(result.total_cc, 2);
}

// ========================
// Python cyclomatic complexity tests
// ========================

#[test]
fn cc_python_simple_function() {
    let code = r#"
def hello():
    print("Hello")
"#;
    let result = estimate_cyclomatic_complexity(code, "python");
    assert_eq!(result.function_count, 1);
    assert_eq!(result.total_cc, 1);
}

#[test]
fn cc_python_if_elif() {
    let code = r#"
def check(x):
    if x > 0:
        print("positive")
    elif x < 0:
        print("negative")
    else:
        print("zero")
"#;
    let result = estimate_cyclomatic_complexity(code, "python");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 if + 1 elif = 3
    assert_eq!(result.total_cc, 3);
}

#[test]
fn cc_python_loops() {
    let code = r#"
def process(items):
    for item in items:
        print(item)
    while True:
        break
"#;
    let result = estimate_cyclomatic_complexity(code, "python");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 for + 1 while = 3
    assert_eq!(result.total_cc, 3);
}

#[test]
fn cc_python_logical_operators() {
    let code = r#"
def check(a, b, c):
    if a and b or c:
        print("complex")
"#;
    let result = estimate_cyclomatic_complexity(code, "python");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 if + 1 and + 1 or = 4
    assert_eq!(result.total_cc, 4);
}

#[test]
fn cc_python_exception_handling() {
    let code = r#"
def risky():
    try:
        something()
    except ValueError:
        handle()
    except TypeError:
        other()
"#;
    let result = estimate_cyclomatic_complexity(code, "python");
    assert_eq!(result.function_count, 1);
    // 1 base + 2 except = 3
    assert_eq!(result.total_cc, 3);
}

#[test]
fn cc_python_async_def() {
    let code = r#"
async def fetch():
    if data:
        return data
"#;
    let result = estimate_cyclomatic_complexity(code, "python");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 if = 2
    assert_eq!(result.total_cc, 2);
}

// ========================
// JavaScript cyclomatic complexity tests
// ========================

#[test]
fn cc_js_simple_function() {
    let code = r#"
function hello() {
    console.log("Hello");
}
"#;
    let result = estimate_cyclomatic_complexity(code, "javascript");
    assert_eq!(result.function_count, 1);
    assert_eq!(result.total_cc, 1);
}

#[test]
fn cc_js_if_else_if() {
    let code = r#"
function check(x) {
    if (x > 0) {
        console.log("positive");
    } else if (x < 0) {
        console.log("negative");
    } else {
        console.log("zero");
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "javascript");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 if + 1 else if = 3
    assert_eq!(result.total_cc, 3);
}

#[test]
fn cc_js_switch_case() {
    let code = r#"
function classify(x) {
    switch (x) {
        case 0:
            return "zero";
        case 1:
            return "one";
        default:
            return "other";
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "javascript");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 switch + 2 case = 4
    assert_eq!(result.total_cc, 4);
}

#[test]
fn cc_js_ternary_operator() {
    let code = r#"
function max(a, b) {
    return a > b ? a : b;
}
"#;
    let result = estimate_cyclomatic_complexity(code, "javascript");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 ternary = 2
    assert_eq!(result.total_cc, 2);
}

#[test]
fn cc_js_ternary_with_multibyte_chars() {
    // Regression: byte-vs-char index mismatch in count_ternary_op
    // used to panic when multi-byte UTF-8 characters preceded a `?`
    // on the same line.
    let code = "function pick(x) {\n    return x === \"🎉🎉🎉\" ? \"ok\" : \"no\";\n}\n";
    let result = estimate_cyclomatic_complexity(code, "javascript");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 ternary = 2
    assert_eq!(result.total_cc, 2);
}

#[test]
fn cc_js_logical_operators() {
    let code = r#"
function check(a, b) {
    if (a && b || !a) {
        return true;
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "javascript");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 if + 1 && + 1 || = 4
    assert_eq!(result.total_cc, 4);
}

#[test]
fn cc_js_try_catch() {
    let code = r#"
function risky() {
    try {
        something();
    } catch (e) {
        console.error(e);
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "javascript");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 catch = 2
    assert_eq!(result.total_cc, 2);
}

#[test]
fn cc_typescript_same_as_js() {
    let code = r#"
function greet(name: string): void {
    if (name) {
        console.log(`Hello, ${name}`);
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "typescript");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 if = 2
    assert_eq!(result.total_cc, 2);
}

// ========================
// Go cyclomatic complexity tests
// ========================

#[test]
fn cc_go_simple_function() {
    let code = r#"
func hello() {
    fmt.Println("Hello")
}
"#;
    let result = estimate_cyclomatic_complexity(code, "go");
    assert_eq!(result.function_count, 1);
    assert_eq!(result.total_cc, 1);
}

#[test]
fn cc_go_if_else() {
    let code = r#"
func check(x int) {
    if x > 0 {
        fmt.Println("positive")
    } else if x < 0 {
        fmt.Println("negative")
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "go");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 if + 1 else if = 3
    assert_eq!(result.total_cc, 3);
}

#[test]
fn cc_go_switch_case() {
    let code = r#"
func classify(x int) string {
    switch x {
    case 0:
        return "zero"
    case 1:
        return "one"
    default:
        return "other"
    }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "go");
    assert_eq!(result.function_count, 1);
    // 1 base + 1 switch + 2 case = 4
    assert_eq!(result.total_cc, 4);
}

// ========================
// High complexity detection
// ========================

#[test]
fn cc_high_complexity_function() {
    let code = r#"
fn very_complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                for i in 0..x {
                    if i % 2 == 0 && i > 5 || i < 3 {
                        while i > 0 {
                            match i {
                                0 => return 0,
                                1 => return 1,
                                _ => continue,
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
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    assert!(
        result.max_cc > 10,
        "Expected high complexity, got {}",
        result.max_cc
    );
    assert!(!result.high_complexity_functions.is_empty());
    assert_eq!(result.high_complexity_functions[0].name, "very_complex");
}

// ========================
// Edge cases
// ========================

#[test]
fn cc_comments_ignored() {
    let code = r#"
fn example() {
    // if this was real, it would add complexity
    // for loops are cool
    // while true {}
    println!("actual code");
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    assert_eq!(result.total_cc, 1); // Only base complexity
}

#[test]
fn cc_average_complexity() {
    let code = r#"
fn a() { }
fn b() { if true { } }
fn c() { if true { } if true { } }
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 3);
    // a: 1, b: 2, c: 3, total: 6, avg: 2.0
    assert!((result.avg_cc - 2.0).abs() < 0.5);
}

// ========================
// Language aliases
// ========================

#[test]
fn cc_language_aliases() {
    let rust_code = "fn test() { }";

    // Rust aliases
    assert_eq!(
        estimate_cyclomatic_complexity(rust_code, "rust").function_count,
        1
    );
    assert_eq!(
        estimate_cyclomatic_complexity(rust_code, "rs").function_count,
        1
    );
    assert_eq!(
        estimate_cyclomatic_complexity(rust_code, "RUST").function_count,
        1
    );

    // Python aliases
    let py_code = "def test():\n    pass";
    assert_eq!(
        estimate_cyclomatic_complexity(py_code, "python").function_count,
        1
    );
    assert_eq!(
        estimate_cyclomatic_complexity(py_code, "py").function_count,
        1
    );

    // JS/TS aliases
    let js_code = "function test() { }";
    assert_eq!(
        estimate_cyclomatic_complexity(js_code, "javascript").function_count,
        1
    );
    assert_eq!(
        estimate_cyclomatic_complexity(js_code, "js").function_count,
        1
    );
    assert_eq!(
        estimate_cyclomatic_complexity(js_code, "typescript").function_count,
        1
    );
    assert_eq!(
        estimate_cyclomatic_complexity(js_code, "ts").function_count,
        1
    );
}

// ========================
// Function name extraction
// ========================

#[test]
fn cc_extracts_function_names() {
    let code = r#"
fn my_function() {
    if true { }
    if true { }
    if true { }
    if true { }
    if true { }
    if true { }
    if true { }
    if true { }
    if true { }
    if true { }
    if true { }
}
"#;
    let result = estimate_cyclomatic_complexity(code, "rust");
    assert_eq!(result.function_count, 1);
    assert!(!result.high_complexity_functions.is_empty());
    assert_eq!(result.high_complexity_functions[0].name, "my_function");
    assert!(result.high_complexity_functions[0].line > 0);
}
