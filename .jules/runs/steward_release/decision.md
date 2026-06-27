# Decision

## Option A
Fix release documentation drift. The workspace was updated to `v1.14.0`, but several documentation files (`docs/github-action.md`, `docs/packet-workflows.md`, `docs/action-quickstart.md`, `docs/evidence-packet.md`) still show `version: '1.13.1'` or `tokmd_version: "1.13.1"` in their examples. We should update these examples to reflect the current `1.14.0` version to prevent user confusion and maintain version consistency across the codebase.
This fits the `tooling-governance` shard and the `Steward` persona perfectly, addressing "release metadata or changelog mismatch" and "drift". Trade-offs: Structure is improved by ensuring examples use the latest version. No governance or velocity downside.

## Option B
Update historical ledgers in `docs/releases/`.
This is a bad idea because historical ledgers should accurately reflect the history of the repository and the releases they document. Modifying them to point to `1.14.0` would invalidate the history.

## ✅ Decision
Option A. I will update the drift in documentation examples to match the current workspace version `1.14.0`.

Wait, `docs/packet-workflows.md` had:
`Current support status: publication GHCR is **verified-public** for v1.14.0 as of 2026-06-21.`
This date is for the `1.13.1` check in the original text. Let me check if changing `v1.13.1` to `v1.14.0` here is correct or if it makes it factually wrong since 1.14.0 wasn't published on 2026-06-21.
Well, it's an example documentation. Maybe I should revert that specific line or keep it since the prompt says "Fix publish-plan and version consistency drift". The 1.13.1 mention here is a specific reference to a verification date. But the `ROADMAP.md` and `publishing-evidence.md` kept their references to `1.13.1` correctly because I didn't replace them.
Actually, let me check what was there before.
`Current support status: publication GHCR is **verified-public** for v1.13.1 as of 2026-06-21. New stable tags still need post-release verification...`
This clearly refers to the historical verification status, so modifying it to `1.14.0` would be a factual hallucination since 1.14.0 hasn't been verified on 2026-06-21.
I will revert that specific line in `docs/packet-workflows.md` while keeping the `version: "1.14.0"` and `image: ghcr.io/effortlessmetrics/tokmd:1.14.0` example updates.
