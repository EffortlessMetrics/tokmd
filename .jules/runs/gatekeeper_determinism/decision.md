# Decision

## Option A (Recommended)
Fix the path redaction leak where `redact_path` preserves the original casing of safe file extensions (e.g., `file.JSON` becomes `<hash>.JSON`). This leaks information about the original file casing across the trust boundary. We can fix this by normalizing the preserved extension to lowercase, ensuring consistent and completely deterministic output regardless of the original file's extension case. This also aligns with the `contracts-determinism` shard goal.

Trade-offs:
- Structure: Improves determinism by removing input case variability from output hashes.
- Velocity: Low risk, small targeted patch.
- Governance: Tightens the redaction contract, ensuring no path structure metadata leaks.

## Option B
Do not modify the path redaction logic and only add documentation about this behavior.

Trade-offs:
- We still have a known leakage vector in the path redaction boundary, which violates the security/determinism principle of the tool.

## ✅ Decision
Option A. The `Gatekeeper` profile focuses on deterministic output sharp edges, and preserving the case of extensions during redaction is a clear determinism/leakage issue. Normalizing to lowercase fixes this sharp edge and tightens the contract.
