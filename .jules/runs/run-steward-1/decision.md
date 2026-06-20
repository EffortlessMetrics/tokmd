# Decision

## Option A (recommended)
Update the `ROADMAP.md` table to include the 1.12.0, 1.13.0, and 1.13.1 releases, fixing the publishing-evidence drift where the latest releases are entirely missing from the historical summary table in the roadmap. This aligns perfectly with the "Steward 🚢" persona targeting "release metadata or changelog mismatch" and docs drift.

- **Structure**: High. Fixes an obvious drift in the documentation artifact.
- **Velocity**: High. Simple markdown fix.
- **Governance**: High. Maintains consistency in project metadata.

## Option B
Find another small documentation fix, like ensuring all URLs in `docs/` point to the correct places or checking for other mismatches.

- **Structure**: Medium.
- **Velocity**: Medium. Requires more search to find targets.
- **Governance**: Medium.

## Decision
I'll go with Option A. `ROADMAP.md` is clearly missing the 1.12.0, 1.13.0, and 1.13.1 versions in its "Status Summary" table. I'll add them based on the CHANGELOG.md contents.
