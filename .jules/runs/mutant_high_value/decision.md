## 🧭 Options considered

### Option A (recommended)
- Add stronger testing for `.gitignore`-style file redaction gaps and missing property-based bounds on path normalization.
- The `redact_path` function correctly ignores `.[ext]` files unless they are a `.tar.gz`. However, `.tar.gz` creates `.gz` which isn't a true leak but feels like an edge case worth testing explicitly.
- Improve `test_cyclonedx_redaction.rs` or `test_redaction_leak.rs` by adding boundary conditions around hidden files (`.npmrc`, `.env`, `.gitignore`, `.tar.gz`) to guarantee extensions are not inadvertently leaked, as hidden files typically shouldn't be treated as "no name, just extension" by the redactor.
- **Why it fits:** Reduces uncertainty around security-boundary path redaction.
- **Trade-offs:** Small test addition, purely strengthens the proof surface (Mutant style).

### Option B
- Add a property-based test (proptest) specifically to hit the `redact_path` logic with arbitrary strings, ensuring it never panics and never exceeds a certain length unless multiple compound suffixes are added.
- **Why it fits:** Very exhaustive, fits "Prover".
- **Trade-offs:** May increase test execution time slightly, and the `mutant` coverage might already be decent.

## ✅ Decision
Option A. Adding explicit boundary tests for path redaction around hidden files (`.env`, `.gitignore`) is an obvious gap and provides a very clean, fast mutant-killing proof improvement.
