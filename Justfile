# Common dev tasks for tokmd (works on macOS/Linux/Windows as long as cargo is on PATH).

default: ci

fmt:
    cargo fmt -- --check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test --all-features

ci: fmt clippy test

# Packaging sanity (what actually ships)
package:
    cargo package -p tokmd --list

# Publishing helpers (via xtask)
publish-plan:
    cargo xtask publish --plan --verbose

publish-dry:
    cargo xtask publish --dry-run

# Fast dry-run (skip tests/checks, just validate packaging)
publish-dry-fast:
    cargo xtask publish --dry-run --skip-checks

publish:
    cargo xtask publish --yes

publish-tag:
    cargo xtask publish --yes --tag

install:
    cargo install --path crates/tokmd --force
