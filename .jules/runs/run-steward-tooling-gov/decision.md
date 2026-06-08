# Decision

## Option A (recommended)
**Action:** Update `ci/proof.toml` to include `docs/release-readiness.md` under the `release_metadata` scope.
**Why it fits:** The `release_metadata` scope currently watches `docs/releases/**` but `docs/release-readiness.md` is technically located in `docs/` and therefore does not match the `docs/releases/**` glob. However, `release-readiness.md` is fundamentally a release-surface artifact. If we modify `release-readiness.md`, it should trigger the same publish-surface, version-consistency, and docs verifications as the files in `docs/releases/**`.
**Trade-offs:**
- **Structure:** Tightens proof alignment.
- **Velocity:** Neutral, avoids missing release proof checks for modifications.
- **Governance:** High confidence, exactly fits `governance-release` profile by securing RC-hardening docs alignment.

## Option B
**Action:** Move `docs/release-readiness.md` into `docs/releases/release-readiness.md`.
**Why it fits:** Structurally groups all release artifacts together, making it match the current glob.
**Trade-offs:**
- **Structure:** Better grouping.
- **Velocity:** Negative, requires updating all links to `release-readiness.md` across the entire codebase (`CHANGELOG.md`, `.github`, etc).
- **Governance:** Introduces churn.

## Decision
Choose Option A. Modifying `ci/proof.toml` is a minimal, low-risk, high-confidence change that perfectly fits the `Steward` and `governance-release` profile without introducing unnecessary file movement churn.
