use crate::cli::AffectedArgs;
use crate::proof::policy::load_policy;
use crate::proof::policy_ast::{ProofPolicy, Scope, ScopeKind};
use crate::proof::validate::validate_policy;
use anyhow::{Context, Result, bail};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize)]
pub(crate) struct AffectedReport {
    pub(crate) schema: String,
    pub(crate) ok: bool,
    pub(crate) base: String,
    pub(crate) head: String,
    pub(crate) changed_files: Vec<String>,
    pub(crate) scopes: Vec<AffectedScope>,
    pub(crate) unknown_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AffectedScope {
    pub(crate) name: String,
    pub(crate) kind: ScopeKind,
    pub(crate) reason: String,
    pub(crate) matched_files: Vec<String>,
    pub(crate) packages: Vec<String>,
    pub(crate) proof: Vec<String>,
    pub(crate) mutation: bool,
    pub(crate) coverage: bool,
}

#[derive(Debug)]
struct CompiledScope<'a> {
    scope: &'a Scope,
    paths: GlobSet,
}

pub fn run(args: AffectedArgs) -> Result<()> {
    let policy = load_checked_policy(&args.policy)?;
    let changed_files = changed_files(&args.base, &args.head)?;
    let report = affected_report(&policy, &args.base, &args.head, changed_files)?;

    if let Some(path) = &args.json_output {
        write_json_output(path, &report)?;
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human_report(&report);
    }

    if report.ok {
        Ok(())
    } else {
        bail!(
            "affected proof scope discovery found {} unknown file(s)",
            report.unknown_files.len()
        )
    }
}

fn write_json_output(path: &Path, report: &AffectedReport) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(report)?)?;
    Ok(())
}

pub(crate) fn load_checked_policy(path: &Path) -> Result<ProofPolicy> {
    let policy = load_policy(path)?;
    let violations = validate_policy(&policy);
    if !violations.is_empty() {
        for violation in &violations {
            eprintln!("::error file={}::{}", path.display(), violation.message);
        }
        bail!(
            "proof policy is invalid; run `cargo xtask proof-policy --check` before affected/proof planning"
        );
    }
    Ok(policy)
}

pub(crate) fn changed_files(base: &str, head: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--name-only", base, head, "--"])
        .output()
        .with_context(|| format!("failed to run `git diff --name-only {base} {head} --`"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "`git diff --name-only {base} {head} --` failed: {}",
            stderr.trim()
        );
    }

    let mut files = String::from_utf8(output.stdout)
        .context("`git diff --name-only` produced non-UTF-8 output")?
        .lines()
        .map(normalize_path)
        .filter(|path| !path.is_empty())
        .collect::<Vec<_>>();

    files.sort();
    files.dedup();
    Ok(files)
}

pub(crate) fn affected_report(
    policy: &ProofPolicy,
    base: &str,
    head: &str,
    mut changed_files: Vec<String>,
) -> Result<AffectedReport> {
    changed_files.sort();
    changed_files.dedup();

    let compiled_scopes = compile_scopes(&policy.scope)?;
    let mut matched_by_scope = BTreeMap::<String, (&Scope, BTreeSet<String>)>::new();
    let mut unknown_files = Vec::new();

    for file in &changed_files {
        let mut matched = false;

        for compiled in &compiled_scopes {
            if compiled.paths.is_match(file) {
                matched = true;
                matched_by_scope
                    .entry(compiled.scope.name.clone())
                    .or_insert_with(|| (compiled.scope, BTreeSet::new()))
                    .1
                    .insert(file.clone());
            }
        }

        if !matched {
            unknown_files.push(file.clone());
        }
    }

    unknown_files.sort();

    let scopes = matched_by_scope
        .into_iter()
        .map(|(_name, (scope, matches))| affected_scope(scope, matches))
        .collect();

    let fail_on_unknown_non_rust = policy.defaults.fail_on_unknown_non_rust.unwrap_or(false);
    let unknown_non_rust_count = unknown_files
        .iter()
        .filter(|file| !is_rust_source_or_manifest(file))
        .count();
    let ok = !(fail_on_unknown_non_rust && unknown_non_rust_count > 0);

    Ok(AffectedReport {
        schema: "tokmd.affected.v1".to_string(),
        ok,
        base: base.to_string(),
        head: head.to_string(),
        changed_files,
        scopes,
        unknown_files,
    })
}

fn compile_scopes(scopes: &[Scope]) -> Result<Vec<CompiledScope<'_>>> {
    scopes
        .iter()
        .map(|scope| {
            let mut builder = GlobSetBuilder::new();
            for pattern in &scope.paths {
                builder.add(
                    Glob::new(pattern)
                        .with_context(|| format!("invalid scope glob `{pattern}`"))?,
                );
            }

            Ok(CompiledScope {
                scope,
                paths: builder
                    .build()
                    .with_context(|| format!("failed to compile scope `{}` globs", scope.name))?,
            })
        })
        .collect()
}

fn affected_scope(scope: &Scope, matches: BTreeSet<String>) -> AffectedScope {
    AffectedScope {
        name: scope.name.clone(),
        kind: scope.kind.clone(),
        reason: match_reason(&matches),
        matched_files: matches.into_iter().collect(),
        packages: sorted(scope.packages.clone()),
        proof: sorted(scope.proof.clone()),
        mutation: scope.mutation,
        coverage: scope.coverage,
    }
}

fn match_reason(matches: &BTreeSet<String>) -> String {
    let mut files = matches.iter();
    let first = files.next().cloned().unwrap_or_default();
    if matches.len() == 1 {
        format!("matched {first}")
    } else {
        let preview = matches
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        format!("matched {} file(s): {preview}", matches.len())
    }
}

fn sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn normalize_path(path: impl AsRef<str>) -> String {
    path.as_ref().replace('\\', "/")
}

fn is_rust_source_or_manifest(path: &str) -> bool {
    path.ends_with(".rs")
        || matches!(path, "Cargo.toml" | "Cargo.lock")
        || path.ends_with("/Cargo.toml")
}

fn print_human_report(report: &AffectedReport) {
    println!(
        "Affected proof scopes: {} changed file(s), {} scope(s), {} unknown file(s)",
        report.changed_files.len(),
        report.scopes.len(),
        report.unknown_files.len()
    );
    println!("Range: {} -> {}", report.base, report.head);

    for scope in &report.scopes {
        println!("  - {} ({:?}): {}", scope.name, scope.kind, scope.reason);
    }

    for file in &report.unknown_files {
        println!("  - unknown: {file}");
    }
}

#[cfg(test)]
mod tests {
    use super::{affected_report, normalize_path};
    use crate::proof::policy::parse_policy_str;

    fn policy_with_defaults(
        fail_on_unknown_non_rust: bool,
    ) -> crate::proof::policy_ast::ProofPolicy {
        parse_policy_str(&format!(
            r#"
schema = "tokmd.proof_policy.v1"

[defaults]
fail_on_unknown_non_rust = {fail_on_unknown_non_rust}

[[scope]]
name = "core"
kind = "rust"
paths = ["crates/tokmd-core/src/ffi.rs", "crates/tokmd-core/tests/**"]
packages = ["tokmd-core"]
proof = ["cargo test -p tokmd-core ffi"]
mutation = true
coverage = true

[[scope]]
name = "browser"
kind = "non_rust"
paths = ["web/runner/**"]
proof = ["npm --prefix web/runner test"]
reason = "Browser runner tests."

[[scope]]
name = "dependency_graph"
kind = "rust"
paths = ["Cargo.lock", "Cargo.toml", "crates/**/Cargo.toml"]
proof = ["cargo deny --all-features check"]
mutation = false
coverage = false

[[scope]]
name = "fuzz_harnesses"
kind = "rust"
paths = ["fuzz/Cargo.toml", "fuzz/corpus/**", "fuzz/dict/**", "fuzz/fuzz_targets/**"]
packages = ["tokmd-fuzz"]
proof = ["cargo +nightly fuzz list"]
mutation = false
coverage = false

[[dependency_boundary]]
name = "retired_tokmd_config_must_not_return"
packages = ["*"]
forbid = ["tokmd-config"]
reason = "tokmd-config is retired."
"#,
        ))
        .expect("policy should parse")
    }

    #[test]
    fn maps_changed_files_to_sorted_scopes() {
        let policy = policy_with_defaults(true);
        let report = affected_report(
            &policy,
            "base",
            "head",
            vec![
                "web/runner/ingest.test.mjs".to_string(),
                "crates/tokmd-core/src/ffi.rs".to_string(),
            ],
        )
        .expect("affected report");

        assert!(report.ok);
        assert_eq!(report.changed_files.len(), 2);
        assert_eq!(report.scopes[0].name, "browser");
        assert_eq!(
            report.scopes[0].matched_files,
            vec!["web/runner/ingest.test.mjs"]
        );
        assert_eq!(report.scopes[1].name, "core");
        assert_eq!(
            report.scopes[1].matched_files,
            vec!["crates/tokmd-core/src/ffi.rs"]
        );
        assert!(report.unknown_files.is_empty());
    }

    #[test]
    fn no_changes_produces_empty_success_report() {
        let policy = policy_with_defaults(true);
        let report = affected_report(&policy, "HEAD", "HEAD", Vec::new()).expect("report");

        assert!(report.ok);
        assert!(report.changed_files.is_empty());
        assert!(report.scopes.is_empty());
        assert!(report.unknown_files.is_empty());
    }

    #[test]
    fn unknown_non_rust_files_make_report_not_ok_when_policy_requires_it() {
        let policy = policy_with_defaults(true);
        let report = affected_report(
            &policy,
            "base",
            "head",
            vec!["docs/unscoped.md".to_string()],
        )
        .expect("report");

        assert!(!report.ok);
        assert_eq!(report.unknown_files, vec!["docs/unscoped.md"]);
    }

    #[test]
    fn unknown_rust_files_are_reported_without_failing_non_rust_policy() {
        let policy = policy_with_defaults(true);
        let report = affected_report(
            &policy,
            "base",
            "head",
            vec!["crates/new/src/lib.rs".to_string()],
        )
        .expect("report");

        assert!(report.ok);
        assert_eq!(report.unknown_files, vec!["crates/new/src/lib.rs"]);
    }

    #[test]
    fn workspace_lockfile_maps_to_dependency_graph_scope() {
        let policy = policy_with_defaults(true);
        let report = affected_report(&policy, "base", "head", vec!["Cargo.lock".to_string()])
            .expect("report");

        assert!(report.ok);
        assert!(report.unknown_files.is_empty());
        assert_eq!(report.scopes.len(), 1);
        assert_eq!(report.scopes[0].name, "dependency_graph");
        assert_eq!(report.scopes[0].matched_files, vec!["Cargo.lock"]);
        assert_eq!(
            report.scopes[0].proof,
            vec!["cargo deny --all-features check"]
        );
    }

    #[test]
    fn fuzz_corpus_maps_to_fuzz_harness_scope() {
        let policy = policy_with_defaults(true);
        let report = affected_report(
            &policy,
            "base",
            "head",
            vec!["fuzz/corpus/fuzz_run_json/seed_version.txt".to_string()],
        )
        .expect("report");

        assert!(report.ok);
        assert!(report.unknown_files.is_empty());
        assert_eq!(report.scopes.len(), 1);
        assert_eq!(report.scopes[0].name, "fuzz_harnesses");
        assert_eq!(
            report.scopes[0].matched_files,
            vec!["fuzz/corpus/fuzz_run_json/seed_version.txt"]
        );
        assert_eq!(report.scopes[0].packages, vec!["tokmd-fuzz"]);
        assert_eq!(report.scopes[0].proof, vec!["cargo +nightly fuzz list"]);
    }

    #[test]
    fn path_normalization_uses_forward_slashes() {
        assert_eq!(
            normalize_path("crates\\tokmd\\src\\main.rs"),
            "crates/tokmd/src/main.rs"
        );
    }
}
