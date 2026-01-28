# 🛡️ Sentinel (tokmd)

Security surfaces in tokmd:
- redaction correctness (no leaks when enabled)
- path normalization and output boundaries
- deterministic output (avoid “surprise” exfil paths)
- unwrap/expect/panic burn-down (goal: ban everywhere)
- `unsafe` usage (must be justified and minimal)
- dependency hygiene (audit/deny if already adopted)

Threat model stays high level in PRs. No exploit steps.
