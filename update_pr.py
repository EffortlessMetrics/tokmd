import re

with open(".jules/runs/palette_binding_dx/pr_body.md", "r") as f:
    body = f.read()

body = body.replace("across internal Node/Wasm bindings failures.", "across internal Node bindings failures (Wasm is split into a follow-up to stay under the 125 LEM CI budget).")
body = body.replace("- `tokmd-wasm` stringified internal parsing errors to unparseable strings, swallowed circular JS object references with `failed to serialize JS arguments`, and failed to maintain the core envelope error protocol correctly.\n", "")
body = body.replace("for both `tokmd-node` and `tokmd-wasm`", "for `tokmd-node`")
body = body.replace("- `crates/tokmd-wasm/src/lib.rs` - Prefix JSON decoding, extraction, and validation exceptions with bracketed string error codes matching core's format.\n", "")
body = body.replace("cargo build --verbose -p tokmd-node -p tokmd-wasm\ncd crates/tokmd-node && cargo test --verbose\ncd web/runner && npm run test", "cargo build --verbose -p tokmd-node\ncd crates/tokmd-node && cargo test --verbose")
body = body.replace("## 🔜 Follow-ups\nNone.", "## 🔜 Follow-ups\nCreate a follow-up PR for `tokmd-wasm` to bypass the 125 LEM hard limit.")

with open(".jules/runs/palette_binding_dx/pr_body.md", "w") as f:
    f.write(body)
