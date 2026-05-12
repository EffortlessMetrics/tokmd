use super::super::analyze_functions;

// ========================
// Rust tests
// ========================

#[test]
fn rust_simple_function() {
    let code = r#"
fn main() {
    println!("Hello");
}
"#;
    let metrics = analyze_functions(code, "rust");
    assert_eq!(metrics.function_count, 1);
    assert_eq!(metrics.max_function_length, 3);
}

#[test]
fn rust_multiple_functions() {
    let code = r#"
fn main() {
    helper();
}

fn helper() {
    // do something
}

pub fn public_helper() {
    // public
}
"#;
    let metrics = analyze_functions(code, "rust");
    assert_eq!(metrics.function_count, 3);
}

#[test]
fn rust_async_function() {
    let code = r#"
async fn fetch_data() {
    // async work
}

pub async fn public_async() {
    // public async
}
"#;
    let metrics = analyze_functions(code, "rust");
    assert_eq!(metrics.function_count, 2);
}

#[test]
fn rust_nested_braces() {
    let code = r#"
fn complex() {
    if true {
        for i in 0..10 {
            println!("{}", i);
        }
    }
}
"#;
    let metrics = analyze_functions(code, "rust");
    assert_eq!(metrics.function_count, 1);
    assert_eq!(metrics.max_function_length, 7);
}

#[test]
fn rust_language_alias() {
    let code = "fn test() {}";
    let metrics = analyze_functions(code, "rs");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn rust_pub_in_path_function() {
    let code = r#"
pub(in crate::foo) fn bar() {
    println!("hello");
}
"#;
    let metrics = analyze_functions(code, "rust");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn rust_extern_c_function() {
    let code = r#"
extern "C" fn callback() {
    println!("called from C");
}
"#;
    let metrics = analyze_functions(code, "rust");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn rust_pub_crate_unsafe_async_function() {
    let code = r#"
pub(crate) unsafe async fn baz() {
    println!("unsafe async");
}
"#;
    let metrics = analyze_functions(code, "rust");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn rust_raw_identifier_function() {
    let code = r#"
pub(crate) unsafe fn r#match() {
    println!("raw ident");
}
"#;
    let metrics = analyze_functions(code, "rust");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn rust_pub_super_const_function() {
    let code = r#"
pub(super) const fn helper() -> u32 {
    42
}
"#;
    let metrics = analyze_functions(code, "rust");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn rust_leading_underscore_function_name() {
    let code = "fn _private_helper() {}";
    let metrics = analyze_functions(code, "rust");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn rust_unicode_function_name() {
    let code = r#"
fn café() {
    println!("unicode");
}

fn 你好() {
    println!("chinese");
}
"#;
    let metrics = analyze_functions(code, "rust");
    assert_eq!(metrics.function_count, 2);
}

// ========================
// Python tests
// ========================

#[test]
fn python_simple_function() {
    let code = r#"
def main():
    print("Hello")
"#;
    let metrics = analyze_functions(code, "python");
    assert_eq!(metrics.function_count, 1);
    assert_eq!(metrics.max_function_length, 2);
}

#[test]
fn python_multiple_functions() {
    let code = r#"
def main():
    helper()

def helper():
    pass

def another():
    return 42
"#;
    let metrics = analyze_functions(code, "python");
    assert_eq!(metrics.function_count, 3);
}

#[test]
fn python_async_function() {
    let code = r#"
async def fetch():
    await something()
"#;
    let metrics = analyze_functions(code, "python");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn python_nested_blocks() {
    let code = r#"
def complex():
    if True:
        for i in range(10):
            print(i)
    return None
"#;
    let metrics = analyze_functions(code, "python");
    assert_eq!(metrics.function_count, 1);
    assert_eq!(metrics.max_function_length, 5);
}

#[test]
fn python_function_with_comments() {
    let code = r#"
def main():
    # This is a comment
    pass

    # Another comment
"#;
    let metrics = analyze_functions(code, "python");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn python_language_alias() {
    let code = "def test():\n    pass";
    let metrics = analyze_functions(code, "py");
    assert_eq!(metrics.function_count, 1);
}

// ========================
// JavaScript tests
// ========================

#[test]
fn js_function_declaration() {
    let code = r#"
function main() {
    console.log("Hello");
}
"#;
    let metrics = analyze_functions(code, "javascript");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn js_async_function() {
    let code = r#"
async function fetchData() {
    await fetch();
}
"#;
    let metrics = analyze_functions(code, "javascript");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn js_arrow_function() {
    let code = r#"
const add = (a, b) => {
    return a + b;
}
"#;
    let metrics = analyze_functions(code, "javascript");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn js_async_arrow_function() {
    let code = r#"
const fetchData = async () => {
    await fetch();
}
"#;
    let metrics = analyze_functions(code, "javascript");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn js_export_function() {
    let code = r#"
export function helper() {
    return 42;
}

export const util = () => {
    return true;
}
"#;
    let metrics = analyze_functions(code, "javascript");
    assert_eq!(metrics.function_count, 2);
}

#[test]
fn js_method_syntax() {
    let code = r#"
handleClick() {
    this.setState({});
}
"#;
    let metrics = analyze_functions(code, "javascript");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn js_language_aliases() {
    let code = "function test() {}";
    assert_eq!(analyze_functions(code, "js").function_count, 1);
    assert_eq!(analyze_functions(code, "typescript").function_count, 1);
    assert_eq!(analyze_functions(code, "ts").function_count, 1);
    assert_eq!(analyze_functions(code, "jsx").function_count, 1);
    assert_eq!(analyze_functions(code, "tsx").function_count, 1);
}

// ========================
// Go tests
// ========================

#[test]
fn go_simple_function() {
    let code = r#"
func main() {
    fmt.Println("Hello")
}
"#;
    let metrics = analyze_functions(code, "go");
    assert_eq!(metrics.function_count, 1);
}

#[test]
fn go_multiple_functions() {
    let code = r#"
func main() {
    helper()
}

func helper() {
    // do something
}
"#;
    let metrics = analyze_functions(code, "go");
    assert_eq!(metrics.function_count, 2);
}

#[test]
fn go_nested_braces() {
    let code = r#"
func complex() {
    if true {
        for i := 0; i < 10; i++ {
            fmt.Println(i)
        }
    }
}
"#;
    let metrics = analyze_functions(code, "go");
    assert_eq!(metrics.function_count, 1);
    assert_eq!(metrics.max_function_length, 7);
}

// ========================
// Edge cases
// ========================

#[test]
fn empty_content() {
    let metrics = analyze_functions("", "rust");
    assert_eq!(metrics.function_count, 0);
    assert_eq!(metrics.max_function_length, 0);
    assert_eq!(metrics.avg_function_length, 0.0);
    assert_eq!(metrics.functions_over_threshold, 0);
}

#[test]
fn no_functions() {
    let code = r#"
// Just a comment
const x = 5;
"#;
    let metrics = analyze_functions(code, "javascript");
    assert_eq!(metrics.function_count, 0);
}

#[test]
fn unknown_language() {
    let code = "fn main() {}";
    let metrics = analyze_functions(code, "cobol");
    assert_eq!(metrics.function_count, 0);
}

#[test]
fn case_insensitive_language() {
    let code = "fn main() {}";
    assert_eq!(analyze_functions(code, "RUST").function_count, 1);
    assert_eq!(analyze_functions(code, "Rust").function_count, 1);
    assert_eq!(analyze_functions(code, "RuSt").function_count, 1);
}

// ========================
// Metrics calculation tests
// ========================

#[test]
fn avg_function_length_calculation() {
    // Two functions: one with 3 lines, one with 5 lines
    let code = r#"
fn short() {
    x
}

fn longer() {
    a
    b
    c
}
"#;
    let metrics = analyze_functions(code, "rust");
    assert_eq!(metrics.function_count, 2);
    // short: 3 lines, longer: 5 lines, avg = 4.0
    assert!((metrics.avg_function_length - 4.0).abs() < 0.01);
}

#[test]
fn functions_over_threshold() {
    // Create a function with >100 lines
    let mut code = String::from("fn very_long() {\n");
    for i in 0..105 {
        code.push_str(&format!("    line{};\n", i));
    }
    code.push_str("}\n");

    let metrics = analyze_functions(&code, "rust");
    assert_eq!(metrics.function_count, 1);
    assert!(metrics.max_function_length > 100);
    assert_eq!(metrics.functions_over_threshold, 1);
}

#[test]
fn mixed_function_lengths() {
    let mut code = String::new();

    // Short function (3 lines)
    code.push_str("fn short() {\n    x\n}\n\n");

    // Medium function (50 lines)
    code.push_str("fn medium() {\n");
    for _ in 0..48 {
        code.push_str("    line;\n");
    }
    code.push_str("}\n\n");

    // Long function (150 lines)
    code.push_str("fn long() {\n");
    for _ in 0..148 {
        code.push_str("    line;\n");
    }
    code.push_str("}\n");

    let metrics = analyze_functions(&code, "rust");
    assert_eq!(metrics.function_count, 3);
    assert_eq!(metrics.functions_over_threshold, 1); // Only the 150-line function
    assert_eq!(metrics.max_function_length, 150);
}
