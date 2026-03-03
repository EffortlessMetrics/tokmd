//! Extended golden snapshot tests for tokmd-tokeignore template generation.
//!
//! Tests template content stability across profiles and verifies
//! force-overwrite produces identical content.

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

// ── Rust project snapshots ──────────────────────────────────────────

#[test]
fn snapshot_rust_project_template() {
    let content = generate_template(InitProfile::Rust);
    insta::assert_snapshot!("rust_project", content);
}

// ── Python project snapshots ────────────────────────────────────────

#[test]
fn snapshot_python_project_template() {
    let content = generate_template(InitProfile::Python);
    insta::assert_snapshot!("python_project", content);
}

// ── JavaScript / Node project snapshots ─────────────────────────────

#[test]
fn snapshot_node_project_template() {
    let content = generate_template(InitProfile::Node);
    insta::assert_snapshot!("node_project", content);
}

// ── Go project snapshots ────────────────────────────────────────────

#[test]
fn snapshot_go_project_template() {
    let content = generate_template(InitProfile::Go);
    insta::assert_snapshot!("go_project", content);
}

// ── C++ project snapshots ───────────────────────────────────────────

#[test]
fn snapshot_cpp_project_template() {
    let content = generate_template(InitProfile::Cpp);
    insta::assert_snapshot!("cpp_project", content);
}

// ── Monorepo snapshots ──────────────────────────────────────────────

#[test]
fn snapshot_monorepo_template() {
    let content = generate_template(InitProfile::Mono);
    insta::assert_snapshot!("monorepo_project", content);
}

// ── Default template snapshot ───────────────────────────────────────

#[test]
fn snapshot_default_template() {
    let content = generate_template(InitProfile::Default);
    insta::assert_snapshot!("default_project", content);
}

// ── Force-overwrite produces identical template ─────────────────────

#[test]
fn snapshot_force_overwrite_identical() {
    let temp = TempDir::new().unwrap();
    // Write initial file
    std::fs::write(temp.path().join(".tokeignore"), "old content\n").unwrap();
    let args = InitArgs {
        dir: temp.path().to_path_buf(),
        template: InitProfile::Rust,
        force: true,
        print: false,
        non_interactive: true,
    };
    init_tokeignore(&args).unwrap();
    let content = std::fs::read_to_string(temp.path().join(".tokeignore")).unwrap();
    insta::assert_snapshot!("force_overwrite_rust", content);
}
