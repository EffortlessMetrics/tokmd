use super::policy_ast::{
    EXPECTED_SCHEMA, FixtureBlobRule, ProofPolicy, RETIRED_TOKMD_CONFIG, ScopeKind,
    WorkspaceAreaAllow,
};
use globset::Glob;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PolicyViolation {
    pub path: String,
    pub message: String,
}

impl PolicyViolation {
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

pub fn validate_policy(policy: &ProofPolicy) -> Vec<PolicyViolation> {
    let mut violations = Vec::new();

    if policy.schema != EXPECTED_SCHEMA {
        violations.push(PolicyViolation::new(
            "schema",
            format!("expected schema {EXPECTED_SCHEMA}, found {}", policy.schema),
        ));
    }

    validate_executor(policy, &mut violations);
    validate_scopes(policy, &mut violations);
    validate_workspace_allowlists(&policy.allow.workspace_area, &mut violations);
    validate_fixture_blob_rules(&policy.forbid.fixture_blob, &mut violations);
    validate_dependency_boundaries(policy, &mut violations);

    violations
}

fn validate_executor(policy: &ProofPolicy, violations: &mut Vec<PolicyViolation>) {
    if let Some(family) = policy.executor.family.as_deref() {
        if family.trim().is_empty() {
            violations.push(PolicyViolation::new(
                "executor.family",
                "executor family must not be empty",
            ));
        } else if family != "coverage" {
            violations.push(PolicyViolation::new(
                "executor.family",
                format!("unsupported executor family `{family}`; expected `coverage`"),
            ));
        }
    }

    if let Some(max_dry_run_commands) = policy.executor.max_dry_run_commands
        && max_dry_run_commands == 0
    {
        violations.push(PolicyViolation::new(
            "executor.max_dry_run_commands",
            "executor dry-run command limit must be greater than zero",
        ));
    }
}

fn validate_scopes(policy: &ProofPolicy, violations: &mut Vec<PolicyViolation>) {
    let mut names = BTreeSet::new();

    for (index, scope) in policy.scope.iter().enumerate() {
        let base = format!("scope[{index}]");

        if scope.name.trim().is_empty() {
            violations.push(PolicyViolation::new(
                format!("{base}.name"),
                "scope name must not be empty",
            ));
        } else if !names.insert(scope.name.as_str()) {
            violations.push(PolicyViolation::new(
                format!("{base}.name"),
                format!("duplicate scope name `{}`", scope.name),
            ));
        }

        validate_glob_list(&format!("{base}.paths"), &scope.paths, true, violations);
        for (path_index, pattern) in scope.paths.iter().enumerate() {
            if pattern.contains(RETIRED_TOKMD_CONFIG) {
                violations.push(PolicyViolation::new(
                    format!("{base}.paths[{path_index}]"),
                    format!(
                        "retired `{RETIRED_TOKMD_CONFIG}` paths must not return as proof scopes"
                    ),
                ));
            }
        }

        if scope.proof.is_empty() {
            violations.push(PolicyViolation::new(
                format!("{base}.proof"),
                "proof command list must not be empty",
            ));
        }

        for (proof_index, proof) in scope.proof.iter().enumerate() {
            if proof.trim().is_empty() {
                violations.push(PolicyViolation::new(
                    format!("{base}.proof[{proof_index}]"),
                    "proof command must not be empty",
                ));
            }
            if proof.contains(RETIRED_TOKMD_CONFIG) {
                violations.push(PolicyViolation::new(
                    format!("{base}.proof[{proof_index}]"),
                    format!("retired `{RETIRED_TOKMD_CONFIG}` proof commands must not return"),
                ));
            }
        }

        for (package_index, package) in scope.packages.iter().enumerate() {
            if package.trim().is_empty() {
                violations.push(PolicyViolation::new(
                    format!("{base}.packages[{package_index}]"),
                    "package name must not be empty",
                ));
            }
            if package == RETIRED_TOKMD_CONFIG {
                violations.push(PolicyViolation::new(
                    format!("{base}.packages[{package_index}]"),
                    format!("retired package `{RETIRED_TOKMD_CONFIG}` must not return as a proof scope package"),
                ));
            }
        }

        if scope.kind == ScopeKind::NonRust && is_blank(scope.reason.as_deref()) {
            violations.push(PolicyViolation::new(
                format!("{base}.reason"),
                "non-Rust scopes must explain why they are part of the proof policy",
            ));
        }
    }
}

fn validate_workspace_allowlists(
    allowlists: &[WorkspaceAreaAllow],
    violations: &mut Vec<PolicyViolation>,
) {
    let mut names = BTreeSet::new();

    for (index, allow) in allowlists.iter().enumerate() {
        let base = format!("allow.workspace_area[{index}]");

        if allow.name.trim().is_empty() {
            violations.push(PolicyViolation::new(
                format!("{base}.name"),
                "allowlist name must not be empty",
            ));
        } else if !names.insert(allow.name.as_str()) {
            violations.push(PolicyViolation::new(
                format!("{base}.name"),
                format!("duplicate allowlist name `{}`", allow.name),
            ));
        }

        if allow.reason.trim().is_empty() {
            violations.push(PolicyViolation::new(
                format!("{base}.reason"),
                "allowlist entries must include a reason",
            ));
        }

        validate_glob_list(&format!("{base}.paths"), &allow.paths, true, violations);

        for (discourage_index, discourage) in allow.discourage.iter().enumerate() {
            if discourage.trim().is_empty() {
                violations.push(PolicyViolation::new(
                    format!("{base}.discourage[{discourage_index}]"),
                    "discouraged content notes must not be empty",
                ));
            }
        }
    }
}

fn validate_fixture_blob_rules(rules: &[FixtureBlobRule], violations: &mut Vec<PolicyViolation>) {
    let mut names = BTreeSet::new();

    for (index, rule) in rules.iter().enumerate() {
        let base = format!("forbid.fixture_blob[{index}]");

        if rule.name.trim().is_empty() {
            violations.push(PolicyViolation::new(
                format!("{base}.name"),
                "fixture blob rule name must not be empty",
            ));
        } else if !names.insert(rule.name.as_str()) {
            violations.push(PolicyViolation::new(
                format!("{base}.name"),
                format!("duplicate fixture blob rule name `{}`", rule.name),
            ));
        }

        if rule.reason.trim().is_empty() {
            violations.push(PolicyViolation::new(
                format!("{base}.reason"),
                "fixture blob rules must include a reason",
            ));
        }

        if rule.extensions.is_empty() && rule.markers.is_empty() {
            violations.push(PolicyViolation::new(
                base.clone(),
                "fixture blob rules must forbid at least one extension or marker",
            ));
        }

        for (extension_index, extension) in rule.extensions.iter().enumerate() {
            let trimmed = extension.trim();
            if trimmed.is_empty() {
                violations.push(PolicyViolation::new(
                    format!("{base}.extensions[{extension_index}]"),
                    "forbidden fixture blob extensions must not be empty",
                ));
            } else if trimmed.starts_with('.') {
                violations.push(PolicyViolation::new(
                    format!("{base}.extensions[{extension_index}]"),
                    "forbidden fixture blob extensions should omit the leading dot",
                ));
            }
        }

        for (marker_index, marker) in rule.markers.iter().enumerate() {
            if marker.trim().is_empty() {
                violations.push(PolicyViolation::new(
                    format!("{base}.markers[{marker_index}]"),
                    "forbidden fixture blob markers must not be empty",
                ));
            }
        }

        validate_glob_list(&format!("{base}.allow"), &rule.allow, false, violations);
    }
}

fn validate_dependency_boundaries(policy: &ProofPolicy, violations: &mut Vec<PolicyViolation>) {
    let mut names = BTreeSet::new();
    let mut forbids_retired_config = false;

    for (index, boundary) in policy.dependency_boundary.iter().enumerate() {
        let base = format!("dependency_boundary[{index}]");

        if boundary.name.trim().is_empty() {
            violations.push(PolicyViolation::new(
                format!("{base}.name"),
                "dependency boundary name must not be empty",
            ));
        } else if !names.insert(boundary.name.as_str()) {
            violations.push(PolicyViolation::new(
                format!("{base}.name"),
                format!("duplicate dependency boundary name `{}`", boundary.name),
            ));
        }

        if boundary.reason.trim().is_empty() {
            violations.push(PolicyViolation::new(
                format!("{base}.reason"),
                "dependency boundaries must include a reason",
            ));
        }

        for (package_index, package) in boundary.packages.iter().enumerate() {
            if package.trim().is_empty() {
                violations.push(PolicyViolation::new(
                    format!("{base}.packages[{package_index}]"),
                    "dependency boundary package selectors must not be empty",
                ));
            }
        }

        if boundary.forbid.is_empty() {
            violations.push(PolicyViolation::new(
                format!("{base}.forbid"),
                "dependency boundaries must forbid at least one package",
            ));
        }

        for (forbid_index, forbidden) in boundary.forbid.iter().enumerate() {
            if forbidden.trim().is_empty() {
                violations.push(PolicyViolation::new(
                    format!("{base}.forbid[{forbid_index}]"),
                    "forbidden package names must not be empty",
                ));
            }
            if forbidden == RETIRED_TOKMD_CONFIG {
                forbids_retired_config = true;
            }
        }
    }

    if !forbids_retired_config {
        violations.push(PolicyViolation::new(
            "dependency_boundary",
            format!("policy must keep retired `{RETIRED_TOKMD_CONFIG}` dependency forbidden"),
        ));
    }
}

fn validate_glob_list(
    path: &str,
    globs: &[String],
    require_nonempty: bool,
    violations: &mut Vec<PolicyViolation>,
) {
    if require_nonempty && globs.is_empty() {
        violations.push(PolicyViolation::new(path, "glob list must not be empty"));
    }

    for (index, pattern) in globs.iter().enumerate() {
        if pattern.trim().is_empty() {
            violations.push(PolicyViolation::new(
                format!("{path}[{index}]"),
                "glob must not be empty",
            ));
            continue;
        }

        if let Err(err) = Glob::new(pattern) {
            violations.push(PolicyViolation::new(
                format!("{path}[{index}]"),
                format!("invalid glob `{pattern}`: {err}"),
            ));
        }
    }
}

fn is_blank(value: Option<&str>) -> bool {
    value.map(str::trim).unwrap_or("").is_empty()
}

#[cfg(test)]
mod tests {
    use super::validate_policy;
    use crate::proof::policy::parse_policy_str;

    fn policy_with(extra: &str) -> String {
        format!(
            r#"
schema = "tokmd.proof_policy.v1"

{extra}

[[dependency_boundary]]
name = "retired_tokmd_config_must_not_return"
packages = ["*"]
forbid = ["tokmd-config"]
reason = "tokmd-config is retired."
"#
        )
    }

    fn messages_for(content: &str) -> Vec<String> {
        let policy = parse_policy_str(content).expect("policy should parse");
        validate_policy(&policy)
            .into_iter()
            .map(|violation| violation.message)
            .collect()
    }

    #[test]
    fn valid_policy_has_no_violations() {
        let policy = parse_policy_str(include_str!("../../../ci/proof.toml")).expect("parse");
        assert_eq!(validate_policy(&policy), Vec::new());
    }

    #[test]
    fn rejects_duplicate_scope_names() {
        let messages = messages_for(&policy_with(
            r#"
[[scope]]
name = "core"
kind = "rust"
paths = ["crates/tokmd-core/**"]
proof = ["cargo test -p tokmd-core"]

[[scope]]
name = "core"
kind = "rust"
paths = ["crates/tokmd/**"]
proof = ["cargo test -p tokmd"]
"#,
        ));

        assert!(
            messages
                .iter()
                .any(|msg| msg.contains("duplicate scope name"))
        );
    }

    #[test]
    fn rejects_empty_proof_commands() {
        let messages = messages_for(&policy_with(
            r#"
[[scope]]
name = "core"
kind = "rust"
paths = ["crates/tokmd-core/**"]
proof = [" "]
"#,
        ));

        assert!(
            messages
                .iter()
                .any(|msg| msg.contains("proof command must not be empty"))
        );
    }

    #[test]
    fn rejects_scopes_without_proof_commands() {
        let messages = messages_for(&policy_with(
            r#"
[[scope]]
name = "core"
kind = "rust"
paths = ["crates/tokmd-core/**"]
"#,
        ));

        assert!(
            messages
                .iter()
                .any(|msg| msg.contains("proof command list must not be empty"))
        );
    }

    #[test]
    fn rejects_invalid_globs() {
        let messages = messages_for(&policy_with(
            r#"
[[scope]]
name = "core"
kind = "rust"
paths = ["["]
proof = ["cargo test -p tokmd-core"]
"#,
        ));

        assert!(messages.iter().any(|msg| msg.contains("invalid glob")));
    }

    #[test]
    fn rejects_unsupported_executor_family() {
        let violations = {
            let policy = parse_policy_str(&policy_with(
                r#"
[executor]
family = "mutation"
ci_execution = "explicit_opt_in"
max_dry_run_commands = 1
"#,
            ))
            .expect("policy should parse");
            validate_policy(&policy)
        };

        assert!(violations.iter().any(|violation| {
            violation.path == "executor.family"
                && violation
                    .message
                    .contains("unsupported executor family `mutation`")
        }));
    }

    #[test]
    fn rejects_empty_executor_family() {
        let messages = messages_for(&policy_with(
            r#"
[executor]
family = " "
ci_execution = "explicit_opt_in"
max_dry_run_commands = 1
"#,
        ));

        assert!(
            messages
                .iter()
                .any(|msg| msg.contains("executor family must not be empty"))
        );
    }

    #[test]
    fn rejects_zero_executor_dry_run_limit() {
        let violations = {
            let policy = parse_policy_str(&policy_with(
                r#"
[executor]
family = "coverage"
ci_execution = "explicit_opt_in"
max_dry_run_commands = 0
"#,
            ))
            .expect("policy should parse");
            validate_policy(&policy)
        };

        assert!(violations.iter().any(|violation| {
            violation.path == "executor.max_dry_run_commands"
                && violation
                    .message
                    .contains("executor dry-run command limit must be greater than zero")
        }));
    }

    #[test]
    fn rejects_allowlist_without_reason() {
        let policy = parse_policy_str(&policy_with(
            r#"
[[allow.workspace_area]]
name = "scratch"
paths = ["scratch/**"]
reason = " "
"#,
        ))
        .expect("policy should parse");

        let violations = validate_policy(&policy);

        assert!(violations.iter().any(|violation| {
            violation.path == "allow.workspace_area[0].reason"
                && violation
                    .message
                    .contains("allowlist entries must include a reason")
        }));
    }

    #[test]
    fn rejects_fixture_blob_rule_without_reason() {
        let policy = parse_policy_str(&policy_with(
            r#"
[[forbid.fixture_blob]]
name = "crypto"
extensions = ["pem"]
reason = " "
"#,
        ))
        .expect("policy should parse");

        let violations = validate_policy(&policy);

        assert!(violations.iter().any(|violation| {
            violation.path == "forbid.fixture_blob[0].reason"
                && violation
                    .message
                    .contains("fixture blob rules must include a reason")
        }));
    }

    #[test]
    fn rejects_fixture_blob_rule_without_forbidden_patterns() {
        let messages = messages_for(&policy_with(
            r#"
[[forbid.fixture_blob]]
name = "crypto"
reason = "Crypto material is forbidden."
"#,
        ));

        assert!(messages.iter().any(|msg| {
            msg.contains("fixture blob rules must forbid at least one extension or marker")
        }));
    }

    #[test]
    fn rejects_fixture_blob_allow_invalid_globs() {
        let messages = messages_for(&policy_with(
            r#"
[[forbid.fixture_blob]]
name = "crypto"
extensions = ["pem"]
allow = ["["]
reason = "Crypto material is forbidden."
"#,
        ));

        assert!(messages.iter().any(|msg| msg.contains("invalid glob")));
    }

    #[test]
    fn rejects_tokmd_config_scope_packages() {
        let messages = messages_for(&policy_with(
            r#"
[[scope]]
name = "bad_config"
kind = "rust"
paths = ["crates/tokmd-config/**"]
packages = ["tokmd-config"]
proof = ["cargo test -p tokmd-config"]
"#,
        ));

        assert!(
            messages
                .iter()
                .any(|msg| msg.contains("retired package `tokmd-config`"))
        );
        assert!(
            messages
                .iter()
                .any(|msg| msg.contains("retired `tokmd-config` paths"))
        );
        assert!(
            messages
                .iter()
                .any(|msg| msg.contains("retired `tokmd-config` proof commands"))
        );
    }

    #[test]
    fn requires_retired_config_boundary() {
        let policy = parse_policy_str(
            r#"
schema = "tokmd.proof_policy.v1"

[[dependency_boundary]]
name = "other"
packages = ["*"]
forbid = ["some-old-crate"]
reason = "Old crate."
"#,
        )
        .expect("policy should parse");

        let violations = validate_policy(&policy);

        assert!(violations.iter().any(|violation| {
            violation.path == "dependency_boundary" && violation.message.contains("tokmd-config")
        }));
    }
}
