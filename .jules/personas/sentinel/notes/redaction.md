# Redaction Note

For boundary hardening of redaction functions like `redact_path`, while shortening lengths and limiting character sets (e.g. `<= 4` and ASCII alphabetic) reduces leakage risk compared to a loose alphanumeric check, relying on a strict explicit allowlist is the preferred approach for contract-bearing / security-boundary surfaces in this project.
