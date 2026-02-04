# tokmd justfile - shortcuts for common tasks
# See https://just.systems for installation

# Default recipe: show available recipes
default:
    @just --list

# Preview publish order without executing
publish-plan:
    cargo xtask publish --plan --verbose

# Dry-run validation (runs cargo publish --dry-run)
publish-dry:
    cargo xtask publish --dry-run

# Publish all crates to crates.io
publish:
    cargo xtask publish --yes

# Publish all crates and create git tag
publish-tag:
    cargo xtask publish --yes --tag

# Resume publishing from a specific crate
publish-from crate:
    cargo xtask publish --from {{crate}} --yes

# Validate packaging contents for all publishable crates
package-check:
    cargo xtask publish --dry-run

# Build all crates
build:
    cargo build --all-features

# Build in release mode
build-release:
    cargo build --release --all-features

# Run all tests
test:
    cargo test --all-features --verbose

# Run clippy with strict warnings
lint:
    cargo clippy --all-features -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt -- --check

# Run all checks (fmt, lint, test)
check: fmt-check lint test
