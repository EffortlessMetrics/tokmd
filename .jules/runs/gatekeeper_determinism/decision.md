## 🧭 Options considered

### Option A (recommended)
- **What it is**: Ensure CLI execution actually succeeds in determinism tests, and remove fake determinism tests for invalid formats.
- **Why it fits this repo and shard**: The `core-pipeline` shard gate focuses on `contracts-determinism`. We found deterministic regression tests (like `lang_csv_byte_identical`) that were checking determinism of CLI errors because they didn't assert command success. By adding `assert!(o.status.success())` to these tests and removing the invalid format tests, we lock in correct deterministic behavior.
- **Trade-offs**: Structure / Velocity / Governance - Strengthens verification significantly.

### Option B
- **What it is**: Just add `assert!(o.status.success())` and fix the invalid format tests to use valid formats (e.g. change `lang_csv_byte_identical` to `lang_tsv_byte_identical`).
- **When to choose it instead**: If we lacked coverage for `lang tsv`.
- **Trade-offs**: Duplicate coverage since `lang_tsv_is_deterministic` already exists.

## ✅ Decision
Option A. It hardens test coverage, eliminates false positives, and proves the outputs deterministically.
