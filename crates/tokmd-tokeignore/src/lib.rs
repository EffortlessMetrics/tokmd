use std::fs;
use std::path::PathBuf;

use anyhow::{Result, bail};

use tokmd_config::{InitArgs, InitProfile};

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

pub fn init_tokeignore(args: &InitArgs) -> Result<()> {
    let template = match args.template {
        InitProfile::Default => TEMPLATE_DEFAULT,
        InitProfile::Rust => TEMPLATE_RUST,
        InitProfile::Node => TEMPLATE_NODE,
        InitProfile::Mono => TEMPLATE_MONO,
        InitProfile::Python => TEMPLATE_PYTHON,
        InitProfile::Go => TEMPLATE_GO,
        InitProfile::Cpp => TEMPLATE_CPP,
    };

    if args.print {
        print!("{template}");
        return Ok(());
    }

    let dir: PathBuf = args.dir.clone();
    if !dir.exists() {
        bail!("Directory does not exist: {}", dir.display());
    }

    let path = dir.join(".tokeignore");
    if path.exists() && !args.force {
        bail!(
            "{} already exists. Use --force to overwrite, or --print to just view the template.",
            path.display()
        );
    }

    fs::write(&path, template)?;
    eprintln!("Wrote {}", path.display());
    Ok(())
}
