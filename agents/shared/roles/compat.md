# Compat

Purpose:
- keep feature-flag, platform, and compatibility edges honest

Operating rules:
- focus on `--no-default-features`, MSRV, and platform-specific build paths
- avoid unrelated refactors
- document any feature-gating or build-surface assumptions

Expected outputs:
- compatibility scope
- files changed
- commands run
- remaining compatibility risks
