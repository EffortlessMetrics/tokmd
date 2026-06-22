## 💡 Summary
Hardened the formatting pipeline's path redaction logic to prevent directory traversal leakage. `clean_path` now fully resolves `.` and `..` segments before hashing, ensuring that logically identical paths (like `a/../b` and `b`) produce identical deterministic hashes.

## 🎯 Why
Previously, `clean_path` performed simple string replacement to strip leading `./` and interior `/./`, but it did not fully resolve `..` parent traversal segments. This meant that the paths `b` and `a/../b` would produce different redaction hashes. This creates a leakage vulnerability: a malicious scan or an unredacted upstream output could probe or reveal directory structures through differential hashing of traversal paths.

## 🔎 Evidence
Minimal proof:
- `crates/tokmd-format/src/redact/mod.rs`
- Observed behavior: `short_hash("src/../secrets/db.json")` produced a different hash than `short_hash("secrets/db.json")`.
- Test `redaction_directory_traversal` now proves `src/../secrets/db.json` resolves perfectly to the canonical logical path `secrets/db.json` before hashing.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `clean_path` in `tokmd-format/src/redact/mod.rs` to fully parse and resolve `..` and `.` path segments instead of just doing simple string replacements.
- why it fits this repo and shard: The redaction logic hashes paths to protect the system's directory structure while outputting metrics. If directory traversals are passed without normalization, the resulting hash can leak details since `a/../b` hashes differently than `b`, allowing bad actors to reconstruct directory trees.
- trade-offs: Structure: We add a small path segment resolution algorithm into `clean_path`, which handles segments correctly and ensures stability across inputs. Velocity: Modifying `clean_path` took a few minutes to ensure it resolves things safely. Governance: Complies fully with the Gatekeeper/Sentinel rule of deterministic safety and protecting trust boundaries.

### Option B
- what it is: Just use `std::path::PathBuf::canonicalize()`.
- when to choose it instead: When the files exist on disk locally.
- trade-offs: Canonicalization does I/O, fails if files do not exist, and depends heavily on the OS/filesystem. The `tokmd-format` crate formats purely logical paths which may not exist or might originate from memory buffers. We cannot use `fs::canonicalize`.

## ✅ Decision
Option A. `clean_path` now splits paths by `/` and evaluates segments properly using a fast in-memory stack, preserving determinism without touching the disk.

## 🧱 Changes made (SRP)
- `crates/tokmd-format/src/redact/mod.rs`
- `crates/tokmd-format/tests/test_redaction_leak.rs`

## 🧪 Verification receipts
```text
{"cmd": "cargo test -p tokmd-format", "result": "success"}
{"cmd": "cargo build -p tokmd-format", "result": "success"}
{"cmd": "CI=true cargo test -p tokmd-format", "result": "success"}
{"cmd": "cargo fmt -- --check", "result": "success"}
{"cmd": "cargo clippy -- -D warnings", "result": "success"}
```

## 🧭 Telemetry
- Change shape: Hardening
- Blast radius: Output / schema determinism boundary. Only affects redacted metric pipelines.
- Risk class: Low risk. Deterministic logical path resolution is standard and verified by tests. No IO introduced.
- Rollback: Revert the PR.
- Gates run: `cargo test`, `cargo clippy`, `cargo fmt`.

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_redaction/envelope.json`
- `.jules/runs/sentinel_redaction/decision.md`
- `.jules/runs/sentinel_redaction/receipts.jsonl`
- `.jules/runs/sentinel_redaction/result.json`
- `.jules/runs/sentinel_redaction/pr_body.md`

## 🔜 Follow-ups
None.
