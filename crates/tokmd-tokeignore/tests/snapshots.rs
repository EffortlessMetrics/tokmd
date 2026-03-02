//! Insta snapshot tests for tokmd-tokeignore template generation.
//!
//! Pins the exact content of each profile template so regressions
//! in ignore patterns are caught at review time.

use tempfile::TempDir;
use tokmd_config::{InitArgs, InitProfile};
use tokmd_tokeignore::init_tokeignore;

fn generate_template(profile: InitProfile) -> String {
    let temp = TempDir::new().unwrap();
    let args = InitArgs {
        dir: temp.path().to_path_buf(),
        template: profile,
        force: false,
        print: false,
        non_interactive: true,
    };
    init_tokeignore(&args).unwrap();
    std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap()
}

#[test]
fn snapshot_template_default() {
    let content = generate_template(InitProfile::Default);
    insta::assert_snapshot!("template_default", content);
}

#[test]
fn snapshot_template_rust() {
    let content = generate_template(InitProfile::Rust);
    insta::assert_snapshot!("template_rust", content);
}

#[test]
fn snapshot_template_node() {
    let content = generate_template(InitProfile::Node);
    insta::assert_snapshot!("template_node", content);
}

#[test]
fn snapshot_template_mono() {
    let content = generate_template(InitProfile::Mono);
    insta::assert_snapshot!("template_mono", content);
}

#[test]
fn snapshot_template_python() {
    let content = generate_template(InitProfile::Python);
    insta::assert_snapshot!("template_python", content);
}

#[test]
fn snapshot_template_go() {
    let content = generate_template(InitProfile::Go);
    insta::assert_snapshot!("template_go", content);
}

#[test]
fn snapshot_template_cpp() {
    let content = generate_template(InitProfile::Cpp);
    insta::assert_snapshot!("template_cpp", content);
}
