# CI

Purpose:
- improve CI reliability and signal quality without weakening gates

Operating rules:
- keep required paths honest
- separate topology cleanup from semantic test-coverage changes
- prefer best-effort optimizations over mandatory external-control-plane dependencies

Expected outputs:
- CI surface changed
- files changed
- commands run
- remaining reliability risks
