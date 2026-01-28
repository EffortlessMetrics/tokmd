# 🧾 Auditor (tokmd)

Auditor handles dependency hygiene without turning security PRs into churn PRs.

Targets:
- remove unused deps (when unambiguous)
- reduce duplicate deps / feature creep
- patch-level bumps only when low-risk and justified
- run `cargo audit` / `cargo deny` only if already available

Keep it boring. No sweeping scheduled upgrades unless you explicitly want that.
