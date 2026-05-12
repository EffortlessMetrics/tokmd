use super::InitProfile;

pub(super) fn template(profile: InitProfile) -> &'static str {
    match profile {
        InitProfile::Default => TEMPLATE_DEFAULT,
        InitProfile::Rust => TEMPLATE_RUST,
        InitProfile::Node => TEMPLATE_NODE,
        InitProfile::Mono => TEMPLATE_MONO,
        InitProfile::Python => TEMPLATE_PYTHON,
        InitProfile::Go => TEMPLATE_GO,
        InitProfile::Cpp => TEMPLATE_CPP,
    }
}

const TEMPLATE_DEFAULT: &str = r#"# .tokeignore
# Patterns here use gitignore syntax.
#
# Goal: keep LOC summaries focused on *your* code, not build artifacts or vendored blobs.
# Tune aggressively for your repos.

# --- Rust / Cargo ---
target/
**/target/

# --- Node / JS tooling ---
node_modules/
**/node_modules/
dist/
out/
build/
**/build/

# --- Python ---
__pycache__/
**/__pycache__/
.venv/
**/.venv/
venv/
**/venv/
.tox/
**/.tox/

# --- Common vendored / third-party dirs ---
vendor/
**/vendor/
third_party/
**/third_party/
external/
**/external/

# --- Generated code ---
generated/
**/generated/
*.generated.*
*.gen.*

# --- Coverage / reports ---
coverage/
**/coverage/
.coverage
lcov.info

# --- tokmd outputs ---
.runs/
**/.runs/

# --- Tree-sitter (common "big files" when vendored) ---
# Adjust to match your vendor layout.
**/tree-sitter*/src/parser.c
**/tree-sitter*/src/scanner.c
**/tree-sitter*/src/*_scanner.c
"#;

const TEMPLATE_RUST: &str = r#"# .tokeignore (Rust)
# Focus: ignore build outputs and generated artifacts.

target/
**/target/

**/*.rs.bk

# Coverage
coverage/
**/coverage/

# tokmd outputs
.runs/
**/.runs/
"#;

const TEMPLATE_NODE: &str = r#"# .tokeignore (Node)
node_modules/
**/node_modules/
dist/
**/dist/
out/
**/out/
build/
**/build/
coverage/
**/coverage/

# tokmd outputs
.runs/
**/.runs/
"#;

const TEMPLATE_MONO: &str = r#"# .tokeignore (Monorepo)
# A conservative monorepo template. Tune to your reality.

# Rust
target/
**/target/

# Node
node_modules/
**/node_modules/
dist/
**/dist/
out/
**/out/
build/
**/build/

# Python
__pycache__/
**/__pycache__/
.venv/
**/.venv/
venv/
**/venv/
.tox/
**/.tox/

# Common vendored / third-party
vendor/
**/vendor/
third_party/
**/third_party/
external/
**/external/

# Generated code
generated/
**/generated/
*.generated.*
*.gen.*

# Coverage / reports
coverage/
**/coverage/
.coverage
lcov.info

# tokmd outputs
.runs/
**/.runs/

# Tree-sitter vendoring (common big files)
**/tree-sitter*/src/parser.c
**/tree-sitter*/src/scanner.c
**/tree-sitter*/src/*_scanner.c
"#;

const TEMPLATE_PYTHON: &str = r#"# .tokeignore (Python)
__pycache__/
**/__pycache__/
*.pyc
.venv/
**/.venv/
venv/
**/venv/
.tox/
**/.tox/
.pytest_cache/
**/.pytest_cache/
htmlcov/
**/htmlcov/
.coverage

# tokmd outputs
.runs/
**/.runs/
"#;

const TEMPLATE_GO: &str = r#"# .tokeignore (Go)
vendor/
**/vendor/
bin/
**/bin/

# tokmd outputs
.runs/
**/.runs/
"#;

const TEMPLATE_CPP: &str = r#"# .tokeignore (C++)
build/
**/build/
cmake-build-*/
**/cmake-build-*/
out/
**/out/
.cache/
**/.cache/

# tokmd outputs
.runs/
**/.runs/
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn all_templates() -> [&'static str; 7] {
        [
            TEMPLATE_DEFAULT,
            TEMPLATE_RUST,
            TEMPLATE_NODE,
            TEMPLATE_MONO,
            TEMPLATE_PYTHON,
            TEMPLATE_GO,
            TEMPLATE_CPP,
        ]
    }

    #[test]
    fn test_default_template_contains_expected_sections() {
        assert!(TEMPLATE_DEFAULT.contains("# .tokeignore"));
        assert!(TEMPLATE_DEFAULT.contains("target/"));
        assert!(TEMPLATE_DEFAULT.contains("node_modules/"));
        assert!(TEMPLATE_DEFAULT.contains("__pycache__/"));
        assert!(TEMPLATE_DEFAULT.contains(".runs/"));
    }

    #[test]
    fn test_rust_template_is_rust_specific() {
        assert!(TEMPLATE_RUST.contains("(Rust)"));
        assert!(TEMPLATE_RUST.contains("target/"));
        assert!(!TEMPLATE_RUST.contains("node_modules/"));
    }

    #[test]
    fn test_node_template_is_node_specific() {
        assert!(TEMPLATE_NODE.contains("(Node)"));
        assert!(TEMPLATE_NODE.contains("node_modules/"));
        assert!(!TEMPLATE_NODE.contains("__pycache__/"));
    }

    #[test]
    fn test_python_template_is_python_specific() {
        assert!(TEMPLATE_PYTHON.contains("(Python)"));
        assert!(TEMPLATE_PYTHON.contains("__pycache__/"));
        assert!(TEMPLATE_PYTHON.contains(".venv/"));
    }

    #[test]
    fn test_go_template_is_go_specific() {
        assert!(TEMPLATE_GO.contains("(Go)"));
        assert!(TEMPLATE_GO.contains("vendor/"));
    }

    #[test]
    fn test_cpp_template_is_cpp_specific() {
        assert!(TEMPLATE_CPP.contains("(C++)"));
        assert!(TEMPLATE_CPP.contains("cmake-build-*/"));
    }

    #[test]
    fn test_mono_template_covers_multiple_ecosystems() {
        assert!(TEMPLATE_MONO.contains("(Monorepo)"));
        assert!(TEMPLATE_MONO.contains("target/"));
        assert!(TEMPLATE_MONO.contains("node_modules/"));
        assert!(TEMPLATE_MONO.contains("__pycache__/"));
        assert!(TEMPLATE_MONO.contains("vendor/"));
    }

    #[test]
    fn test_template_selects_requested_profile() {
        assert!(template(InitProfile::Rust).contains("(Rust)"));
        assert!(template(InitProfile::Node).contains("(Node)"));
        assert!(template(InitProfile::Mono).contains("(Monorepo)"));
        assert!(template(InitProfile::Python).contains("(Python)"));
        assert!(template(InitProfile::Go).contains("(Go)"));
        assert!(template(InitProfile::Cpp).contains("(C++)"));
    }

    #[test]
    fn test_all_templates_end_with_newline() {
        for template in all_templates() {
            assert!(template.ends_with('\n'), "template should end with newline");
        }
    }

    #[test]
    fn test_all_templates_contain_runs_dir() {
        for template in all_templates() {
            assert!(
                template.contains(".runs/"),
                "every template should exclude .runs/"
            );
        }
    }
}
