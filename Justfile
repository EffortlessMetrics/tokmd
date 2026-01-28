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

# Publishing helpers (requires cargo-workspaces)
setup-publish:
    cargo install cargo-workspaces

publish-dry: setup-publish
    cargo ws publish --from-git --dry-run

publish: setup-publish
    cargo ws publish --from-git --publish-interval 10 --yes

install:
    cargo install --path crates/tokmd --force
