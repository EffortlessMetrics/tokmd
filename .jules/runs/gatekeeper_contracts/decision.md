Option A (recommended)
Add `changelog_documents_cockpit_schema_version`, `changelog_documents_context_schema_version`, `changelog_documents_context_bundle_schema_version`, `changelog_documents_handoff_schema_version`, `changelog_documents_tool_schema_version` tests to `xtask/tests/docs_schema_w72.rs` so that we verify that `CHANGELOG.md` properly references bumps in *all* schema versions.
* Why it fits this repo and shard: It directly improves our contract determinism testing by enforcing that documentation about schema bumps stays in sync with code bumps. It falls perfectly into the `xtask` path allowed by our boundaries.
* Trade-offs: Structure/Governance (better contract/schema alignment testing) over Velocity (slight increase in tests).

Option B
Manually verify and only add one missing test.
* When to choose it instead: If the other ones are already covered elsewhere.
* Trade-offs: Not a comprehensive solution.
