//! Semantic no-panic checker (rail B of the panic-safety policy stack).
//!
//! This task parses every workspace `.rs` file with `syn`, finds panic-family
//! expressions, and matches them against `policy/no-panic-allowlist.toml`.
//! In strict mode, findings without an allowlist entry, stale entries
//! (entries that no longer match a finding), and expired entries all fail the
//! gate. The default advisory mode treats unallowlisted findings as
//! reportable-but-non-blocking while panic-family debt is being burned down,
//! and only fails on schema/shape errors, expired entries, and stale entries.
//!
//! Allowlist identity is `path + family + selector`, where `selector` is the
//! tuple `(kind, container, callee, receiver_fingerprint)`. Line/column are
//! advisory and never used for matching, so reformatting source files does not
//! invalidate entries.
//!
//! See `docs/NO_PANIC_POLICY.md` for the policy contract.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use cargo_metadata::MetadataCommand;
use chrono::{NaiveDate, Utc};
use proc_macro2::Span;
use serde::{Deserialize, Serialize};
use syn::spanned::Spanned;
use syn::visit::Visit;
use walkdir::WalkDir;

use crate::cli::{NoPanicArgs, NoPanicProposeArgs};

const SCHEMA_VERSION: &str = "0.3";
const ALLOWLIST_PATH: &str = "policy/no-panic-allowlist.toml";

const ALLOWED_CLASSIFICATIONS: &[&str] =
    &["production", "test_helper", "fixture", "tooling", "ffi"];

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

pub fn run_check(args: NoPanicArgs) -> Result<()> {
    let root = workspace_root()?;
    let findings = scan_workspace(&root)?;
    let allowlist = read_allowlist(&root.join(ALLOWLIST_PATH))?;

    let report = evaluate(&findings, &allowlist)?;

    if args.json {
        let json = serde_json::to_string_pretty(&report)?;
        println!("{json}");
    } else {
        println!("{}", report.summary());
        if !args.strict && !report.unallowlisted.is_empty() {
            println!(
                "no-panic policy: advisory mode (re-run with --strict to fail on \
                 unallowlisted findings; staged behind workspace lint inheritance)."
            );
        }
    }

    let blocking = if args.strict {
        report.has_errors()
    } else {
        report.has_blocking_errors()
    };

    if blocking {
        let iter: Box<dyn Iterator<Item = String>> = if args.strict {
            Box::new(report.errors_iter())
        } else {
            Box::new(report.blocking_errors_iter())
        };
        for err in iter {
            eprintln!("no-panic policy error: {err}");
        }
        let count = if args.strict {
            report.error_count()
        } else {
            report.blocking_error_count()
        };
        bail!("no-panic policy check failed with {count} error(s)");
    }

    Ok(())
}

pub fn run_propose(args: NoPanicProposeArgs) -> Result<()> {
    let root = workspace_root()?;
    let findings = scan_workspace(&root)?;
    let allowlist = read_allowlist(&root.join(ALLOWLIST_PATH))?;

    let allowed: BTreeSet<Identity> = allowlist
        .allow
        .iter()
        .map(|entry| entry.identity())
        .collect();

    let mut next_id = next_id(&allowlist);
    let mut buf = String::new();
    buf.push_str("# Proposed no-panic allowlist entries.\n");
    buf.push_str(
        "# Copy entries into policy/no-panic-allowlist.toml after filling in\n\
         # owner, classification, explanation, and a future expires date.\n\n",
    );
    buf.push_str("schema_version = \"0.3\"\n\n");

    let mut count = 0usize;
    for finding in &findings {
        let identity = finding.identity();
        if allowed.contains(&identity) {
            continue;
        }
        count += 1;
        buf.push_str("[[allow]]\n");
        buf.push_str(&format!("id = \"panic-{:04}\"\n", next_id));
        next_id += 1;
        buf.push_str(&format!(
            "path = {}\n",
            toml_string(&finding.path.to_string_lossy())
        ));
        buf.push_str(&format!(
            "family = {}\n",
            toml_string(finding.family.as_str())
        ));
        buf.push_str(
            "classification = \"TODO\"   # production | test_helper | fixture | tooling | ffi\n",
        );
        buf.push_str("owner = \"TODO\"\n");
        buf.push_str("explanation = \"TODO: why this panic-family debt is acceptable for now\"\n");
        buf.push_str("expires = \"TODO\"          # ISO-8601 (e.g. 2026-12-31)\n\n");

        buf.push_str("[allow.selector]\n");
        buf.push_str(&format!(
            "kind = {}\n",
            toml_string(finding.selector.kind.as_str())
        ));
        buf.push_str(&format!(
            "container = {}\n",
            toml_string(&finding.selector.container)
        ));
        buf.push_str(&format!(
            "callee = {}\n",
            toml_string(&finding.selector.callee)
        ));
        buf.push_str(&format!(
            "receiver_fingerprint = {}\n\n",
            toml_string(&finding.selector.receiver_fingerprint)
        ));

        buf.push_str("[allow.last_seen]\n");
        buf.push_str(&format!("line = {}\n", finding.last_seen.line));
        buf.push_str(&format!("column = {}\n\n", finding.last_seen.column));
    }

    let output = if args.output.is_absolute() {
        args.output.clone()
    } else {
        root.join(&args.output)
    };
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create directory {}", parent.display()))?;
    }
    fs::write(&output, buf).with_context(|| format!("write {}", output.display()))?;

    println!(
        "no-panic-propose: wrote {} proposed entr{} to {}",
        count,
        if count == 1 { "y" } else { "ies" },
        output.display()
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Allowlist file parsing
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct AllowlistFile {
    schema_version: String,
    #[serde(default)]
    allow: Vec<AllowEntry>,
}

#[derive(Debug, Deserialize)]
struct AllowEntry {
    id: String,
    path: String,
    family: String,
    classification: String,
    owner: String,
    explanation: String,
    expires: String,
    selector: SelectorTable,
    #[serde(default)]
    #[allow(dead_code)] // last_seen is advisory; never used for matching.
    last_seen: Option<LastSeenTable>,
}

#[derive(Debug, Deserialize)]
struct SelectorTable {
    kind: String,
    container: String,
    callee: String,
    receiver_fingerprint: String,
}

#[derive(Debug, Deserialize)]
struct LastSeenTable {
    #[allow(dead_code)]
    line: usize,
    #[allow(dead_code)]
    column: usize,
}

impl AllowEntry {
    fn identity(&self) -> Identity {
        Identity {
            path: PathBuf::from(&self.path),
            family: Family::from_str(&self.family).unwrap_or(Family::Unknown),
            kind: SelectorKind::from_str(&self.selector.kind).unwrap_or(SelectorKind::Unknown),
            container: self.selector.container.clone(),
            callee: self.selector.callee.clone(),
            receiver_fingerprint: self.selector.receiver_fingerprint.clone(),
        }
    }
}

fn read_allowlist(path: &Path) -> Result<AllowlistFile> {
    if !path.exists() {
        return Ok(AllowlistFile {
            schema_version: SCHEMA_VERSION.to_string(),
            allow: Vec::new(),
        });
    }
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let parsed: AllowlistFile = toml::from_str(&content)
        .with_context(|| format!("parse no-panic allowlist at {}", path.display()))?;
    if parsed.schema_version != SCHEMA_VERSION {
        bail!(
            "{} schema_version must be {SCHEMA_VERSION}, got {:?}",
            path.display(),
            parsed.schema_version
        );
    }
    Ok(parsed)
}

fn next_id(allowlist: &AllowlistFile) -> usize {
    let mut max = 0usize;
    for entry in &allowlist.allow {
        if let Some(stripped) = entry.id.strip_prefix("panic-")
            && let Ok(n) = stripped.parse::<usize>()
        {
            max = max.max(n);
        }
    }
    max + 1
}

// ---------------------------------------------------------------------------
// Identity / family / selector kinds
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
enum Family {
    Unwrap,
    Expect,
    GetUnwrap,
    PanicMacro,
    Todo,
    Unimplemented,
    Unreachable,
    Indexing,
    StringSlice,
    Unknown,
}

impl Family {
    fn as_str(self) -> &'static str {
        match self {
            Family::Unwrap => "unwrap",
            Family::Expect => "expect",
            Family::GetUnwrap => "get_unwrap",
            Family::PanicMacro => "panic_macro",
            Family::Todo => "todo",
            Family::Unimplemented => "unimplemented",
            Family::Unreachable => "unreachable",
            Family::Indexing => "indexing",
            Family::StringSlice => "string_slice",
            Family::Unknown => "unknown",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "unwrap" => Family::Unwrap,
            "expect" => Family::Expect,
            "get_unwrap" => Family::GetUnwrap,
            "panic_macro" => Family::PanicMacro,
            "todo" => Family::Todo,
            "unimplemented" => Family::Unimplemented,
            "unreachable" => Family::Unreachable,
            "indexing" => Family::Indexing,
            "string_slice" => Family::StringSlice,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
enum SelectorKind {
    MethodCall,
    MacroInvocation,
    Indexing,
    Unknown,
}

impl SelectorKind {
    fn as_str(self) -> &'static str {
        match self {
            SelectorKind::MethodCall => "method_call",
            SelectorKind::MacroInvocation => "macro_invocation",
            SelectorKind::Indexing => "indexing",
            SelectorKind::Unknown => "unknown",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "method_call" => SelectorKind::MethodCall,
            "macro_invocation" => SelectorKind::MacroInvocation,
            "indexing" => SelectorKind::Indexing,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Identity {
    path: PathBuf,
    family: Family,
    kind: SelectorKind,
    container: String,
    callee: String,
    receiver_fingerprint: String,
}

#[derive(Debug, Clone, Serialize)]
struct Selector {
    kind: SelectorKind,
    container: String,
    callee: String,
    receiver_fingerprint: String,
}

#[derive(Debug, Clone, Serialize)]
struct LastSeen {
    line: usize,
    column: usize,
}

#[derive(Debug, Clone, Serialize)]
struct Finding {
    path: PathBuf,
    family: Family,
    selector: Selector,
    last_seen: LastSeen,
}

impl Finding {
    fn identity(&self) -> Identity {
        Identity {
            path: self.path.clone(),
            family: self.family,
            kind: self.selector.kind,
            container: self.selector.container.clone(),
            callee: self.selector.callee.clone(),
            receiver_fingerprint: self.selector.receiver_fingerprint.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Scanning
// ---------------------------------------------------------------------------

fn workspace_root() -> Result<PathBuf> {
    let mut command = MetadataCommand::new();
    command.no_deps();
    let metadata = command.exec().context("load cargo metadata")?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

fn workspace_member_roots(root: &Path) -> Result<Vec<PathBuf>> {
    let mut command = MetadataCommand::new();
    command.no_deps();
    let metadata = command.exec().context("load cargo metadata")?;
    let mut roots = Vec::new();
    for package in metadata.workspace_packages() {
        let manifest_path = package.manifest_path.clone().into_std_path_buf();
        if let Some(parent) = manifest_path.parent() {
            let canonical = parent.to_path_buf();
            if canonical.starts_with(root) {
                roots.push(canonical);
            }
        }
    }
    roots.sort();
    roots.dedup();
    Ok(roots)
}

fn scan_workspace(root: &Path) -> Result<Vec<Finding>> {
    let crate_roots = workspace_member_roots(root)?;
    let mut findings = Vec::new();
    let mut seen_files: BTreeSet<PathBuf> = BTreeSet::new();

    for crate_root in &crate_roots {
        for entry in WalkDir::new(crate_root)
            .into_iter()
            .filter_entry(|e| !is_excluded_dir(e.file_name()))
        {
            let entry = entry.with_context(|| format!("walk {}", crate_root.display()))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            if !seen_files.insert(canonical.clone()) {
                continue;
            }
            let rel = path.strip_prefix(root).unwrap_or(path).to_path_buf();
            let mut file_findings = scan_file(&rel, path)?;
            findings.append(&mut file_findings);
        }
    }
    findings.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.last_seen.line.cmp(&b.last_seen.line))
            .then(a.last_seen.column.cmp(&b.last_seen.column))
            .then(a.family.cmp(&b.family))
    });
    Ok(findings)
}

fn is_excluded_dir(name: &std::ffi::OsStr) -> bool {
    matches!(
        name.to_string_lossy().as_ref(),
        "target" | ".git" | "node_modules" | "vendor"
    )
}

fn scan_file(rel_path: &Path, abs_path: &Path) -> Result<Vec<Finding>> {
    let source = fs::read_to_string(abs_path)
        .with_context(|| format!("read source file {}", abs_path.display()))?;
    let syntax = match syn::parse_file(&source) {
        Ok(syntax) => syntax,
        Err(_) => return Ok(Vec::new()),
    };

    let mut visitor = PanicVisitor {
        rel_path: rel_path.to_path_buf(),
        container_stack: Vec::new(),
        findings: Vec::new(),
    };
    visitor.visit_file(&syntax);
    Ok(visitor.findings)
}

struct PanicVisitor {
    rel_path: PathBuf,
    container_stack: Vec<String>,
    findings: Vec<Finding>,
}

impl PanicVisitor {
    fn current_container(&self) -> String {
        if self.container_stack.is_empty() {
            "<top>".to_string()
        } else {
            self.container_stack.join("::")
        }
    }

    fn record(&mut self, family: Family, selector: Selector, span: Span) {
        let start = span.start();
        self.findings.push(Finding {
            path: self.rel_path.clone(),
            family,
            selector,
            last_seen: LastSeen {
                line: start.line,
                column: start.column,
            },
        });
    }
}

impl<'ast> Visit<'ast> for PanicVisitor {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.container_stack.push(item.sig.ident.to_string());
        syn::visit::visit_item_fn(self, item);
        self.container_stack.pop();
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.container_stack.push(item.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, item);
        self.container_stack.pop();
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        self.container_stack.push(item.sig.ident.to_string());
        syn::visit::visit_trait_item_fn(self, item);
        self.container_stack.pop();
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let type_name = type_name_of(&item.self_ty);
        self.container_stack.push(type_name);
        syn::visit::visit_item_impl(self, item);
        self.container_stack.pop();
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let method = call.method.to_string();
        let family = match method.as_str() {
            "unwrap" => Some(Family::Unwrap),
            "expect" => Some(Family::Expect),
            "get_unwrap" => Some(Family::GetUnwrap),
            _ => None,
        };
        if let Some(family) = family {
            let receiver = fingerprint_expr(&call.receiver);
            let selector = Selector {
                kind: SelectorKind::MethodCall,
                container: self.current_container(),
                callee: method,
                receiver_fingerprint: receiver,
            };
            self.record(family, selector, call.span());
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_macro(&mut self, mac: &'ast syn::ExprMacro) {
        if let Some(family) = panic_macro_family(&mac.mac.path) {
            let callee = path_string(&mac.mac.path);
            let fingerprint = truncate_fingerprint(&mac.mac.tokens.to_string());
            let selector = Selector {
                kind: SelectorKind::MacroInvocation,
                container: self.current_container(),
                callee,
                receiver_fingerprint: fingerprint,
            };
            self.record(family, selector, mac.span());
        }
        syn::visit::visit_expr_macro(self, mac);
    }

    fn visit_stmt_macro(&mut self, mac: &'ast syn::StmtMacro) {
        if let Some(family) = panic_macro_family(&mac.mac.path) {
            let callee = path_string(&mac.mac.path);
            let fingerprint = truncate_fingerprint(&mac.mac.tokens.to_string());
            let selector = Selector {
                kind: SelectorKind::MacroInvocation,
                container: self.current_container(),
                callee,
                receiver_fingerprint: fingerprint,
            };
            self.record(family, selector, mac.span());
        }
        syn::visit::visit_stmt_macro(self, mac);
    }

    fn visit_expr_index(&mut self, idx: &'ast syn::ExprIndex) {
        let receiver = fingerprint_expr(&idx.expr);
        let index = fingerprint_expr(&idx.index);
        // Treat range indexing as string_slice; element indexing as indexing.
        let family = if matches!(idx.index.as_ref(), syn::Expr::Range(_)) {
            Family::StringSlice
        } else {
            Family::Indexing
        };
        let selector = Selector {
            kind: SelectorKind::Indexing,
            container: self.current_container(),
            callee: "[]".to_string(),
            receiver_fingerprint: format!("{}[{}]", receiver, index),
        };
        self.record(family, selector, idx.span());
        syn::visit::visit_expr_index(self, idx);
    }
}

fn panic_macro_family(path: &syn::Path) -> Option<Family> {
    let last = path.segments.last()?;
    match last.ident.to_string().as_str() {
        "panic" => Some(Family::PanicMacro),
        "todo" => Some(Family::Todo),
        "unimplemented" => Some(Family::Unimplemented),
        "unreachable" => Some(Family::Unreachable),
        _ => None,
    }
}

fn path_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn type_name_of(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(p) => path_string(&p.path),
        other => collapse_whitespace(&quote_to_string(other)),
    }
}

fn quote_to_string<T: quote::ToTokens>(item: &T) -> String {
    let mut tokens = proc_macro2::TokenStream::new();
    item.to_tokens(&mut tokens);
    tokens.to_string()
}

fn fingerprint_expr(expr: &syn::Expr) -> String {
    let raw = quote_to_string(expr);
    truncate_fingerprint(&raw)
}

fn truncate_fingerprint(raw: &str) -> String {
    let collapsed = collapse_whitespace(raw);
    const MAX: usize = 160;
    if collapsed.chars().count() <= MAX {
        collapsed
    } else {
        let head: String = collapsed.chars().take(MAX).collect();
        format!("{head}…")
    }
}

fn collapse_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Serialize)]
struct Report {
    finding_count: usize,
    matched_count: usize,
    #[serde(rename = "unallowlisted_findings")]
    unallowlisted: Vec<Finding>,
    #[serde(rename = "stale_entries")]
    stale: Vec<String>,
    #[serde(rename = "expired_entries")]
    expired: Vec<String>,
    #[serde(rename = "shape_errors")]
    shape: Vec<String>,
}

impl Report {
    fn errors_iter(&self) -> impl Iterator<Item = String> + '_ {
        let unalloc = self.unallowlisted.iter().map(|f| {
            format!(
                "unallowlisted {} at {}:{}:{} ({} {} via {})",
                f.family.as_str(),
                f.path.display(),
                f.last_seen.line,
                f.last_seen.column,
                f.selector.kind.as_str(),
                f.selector.callee,
                f.selector.container,
            )
        });
        let stale = self.stale.iter().map(|s| format!("stale entry: {s}"));
        let expired = self.expired.iter().map(|s| format!("expired entry: {s}"));
        let shape = self.shape.iter().cloned();
        unalloc.chain(stale).chain(expired).chain(shape)
    }

    fn error_count(&self) -> usize {
        self.unallowlisted.len() + self.stale.len() + self.expired.len() + self.shape.len()
    }

    fn has_errors(&self) -> bool {
        self.error_count() > 0
    }

    /// Errors that block the gate even outside `--strict` mode: schema/shape
    /// problems, expired entries, and stale entries. Unallowlisted findings
    /// are intentionally advisory until panic-family debt is burned down.
    fn blocking_errors_iter(&self) -> impl Iterator<Item = String> + '_ {
        let stale = self.stale.iter().map(|s| format!("stale entry: {s}"));
        let expired = self.expired.iter().map(|s| format!("expired entry: {s}"));
        let shape = self.shape.iter().cloned();
        stale.chain(expired).chain(shape)
    }

    fn blocking_error_count(&self) -> usize {
        self.stale.len() + self.expired.len() + self.shape.len()
    }

    fn has_blocking_errors(&self) -> bool {
        self.blocking_error_count() > 0
    }

    fn summary(&self) -> String {
        format!(
            "no-panic policy: {} finding(s), {} matched, {} unallowlisted, {} stale, {} expired, {} shape error(s)",
            self.finding_count,
            self.matched_count,
            self.unallowlisted.len(),
            self.stale.len(),
            self.expired.len(),
            self.shape.len(),
        )
    }
}

fn evaluate(findings: &[Finding], allowlist: &AllowlistFile) -> Result<Report> {
    let mut report = Report {
        finding_count: findings.len(),
        ..Report::default()
    };

    // Shape validation -------------------------------------------------------
    let today = Utc::now().date_naive();
    let mut seen_ids: BTreeSet<&str> = BTreeSet::new();
    let mut entry_index: BTreeMap<Identity, &AllowEntry> = BTreeMap::new();
    for entry in &allowlist.allow {
        if !seen_ids.insert(entry.id.as_str()) {
            report.shape.push(format!("duplicate id {}", entry.id));
        }
        if entry.id.is_empty() {
            report.shape.push("entry with empty id".to_string());
        }
        if entry.path.is_empty() {
            report
                .shape
                .push(format!("entry {} has empty path", entry.id));
        }
        if Family::from_str(&entry.family).is_none() {
            report.shape.push(format!(
                "entry {} has unknown family {:?}",
                entry.id, entry.family
            ));
        }
        if SelectorKind::from_str(&entry.selector.kind).is_none() {
            report.shape.push(format!(
                "entry {} selector.kind must be one of method_call|macro_invocation|indexing, got {:?}",
                entry.id, entry.selector.kind
            ));
        }
        if entry.selector.container.is_empty() {
            report.shape.push(format!(
                "entry {} selector.container is empty (use \"<top>\" for module scope)",
                entry.id
            ));
        }
        if entry.selector.callee.is_empty() {
            report
                .shape
                .push(format!("entry {} selector.callee is empty", entry.id));
        }
        if entry.owner.is_empty() {
            report
                .shape
                .push(format!("entry {} has empty owner", entry.id));
        }
        if entry.explanation.is_empty() {
            report
                .shape
                .push(format!("entry {} has empty explanation", entry.id));
        }
        if !ALLOWED_CLASSIFICATIONS.contains(&entry.classification.as_str()) {
            report.shape.push(format!(
                "entry {} classification must be one of {:?}, got {:?}",
                entry.id, ALLOWED_CLASSIFICATIONS, entry.classification
            ));
        }
        match NaiveDate::parse_from_str(&entry.expires, "%Y-%m-%d") {
            Ok(date) => {
                if date < today {
                    report.expired.push(format!(
                        "{} expired on {} ({})",
                        entry.id, entry.expires, entry.path
                    ));
                }
            }
            Err(err) => {
                report.shape.push(format!(
                    "entry {} has invalid expires {:?}: {err}",
                    entry.id, entry.expires
                ));
            }
        }
        let identity = entry.identity();
        if entry_index.insert(identity, entry).is_some() {
            report.shape.push(format!(
                "entry {} has the same identity as another entry",
                entry.id
            ));
        }
    }

    // Match findings against allowlist --------------------------------------
    let mut matched_identities: BTreeSet<Identity> = BTreeSet::new();
    for finding in findings {
        let identity = finding.identity();
        if entry_index.contains_key(&identity) {
            matched_identities.insert(identity);
            report.matched_count += 1;
        } else {
            report.unallowlisted.push(finding.clone());
        }
    }

    // Detect stale entries (allowlist entries with no matching finding) ----
    for (identity, entry) in &entry_index {
        if !matched_identities.contains(identity) {
            report.stale.push(format!(
                "{} (path={} family={} container={} callee={}) no longer matches any finding",
                entry.id, entry.path, entry.family, entry.selector.container, entry.selector.callee,
            ));
        }
    }

    Ok(report)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn toml_string(s: &str) -> String {
    // Render a TOML basic string with required escaping.
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Vec<Finding> {
        let syntax = syn::parse_file(source).expect("parse test source");
        let mut visitor = PanicVisitor {
            rel_path: PathBuf::from("test.rs"),
            container_stack: Vec::new(),
            findings: Vec::new(),
        };
        visitor.visit_file(&syntax);
        visitor.findings
    }

    #[test]
    fn detects_unwrap_in_function() {
        let findings = parse(
            r#"
            fn read_thing(path: &str) -> String {
                std::fs::read_to_string(path).unwrap()
            }
            "#,
        );
        assert_eq!(findings.len(), 1, "{findings:?}");
        let f = &findings[0];
        assert_eq!(f.family, Family::Unwrap);
        assert_eq!(f.selector.kind, SelectorKind::MethodCall);
        assert_eq!(f.selector.callee, "unwrap");
        assert_eq!(f.selector.container, "read_thing");
        assert!(f.selector.receiver_fingerprint.contains("read_to_string"));
    }

    #[test]
    fn detects_panic_macro_in_impl_method() {
        let findings = parse(
            r#"
            struct Thing;
            impl Thing {
                fn boom(&self) {
                    panic!("nope");
                }
            }
            "#,
        );
        assert_eq!(findings.len(), 1);
        let f = &findings[0];
        assert_eq!(f.family, Family::PanicMacro);
        assert_eq!(f.selector.kind, SelectorKind::MacroInvocation);
        assert_eq!(f.selector.container, "Thing::boom");
    }

    #[test]
    fn detects_indexing_and_string_slice() {
        let findings = parse(
            r#"
            fn split(s: &str) -> &str {
                let _ = s.as_bytes()[0];
                &s[1..3]
            }
            "#,
        );
        let families: Vec<Family> = findings.iter().map(|f| f.family).collect();
        assert!(families.contains(&Family::Indexing), "{families:?}");
        assert!(families.contains(&Family::StringSlice), "{families:?}");
    }

    #[test]
    fn ignores_safe_code() {
        let findings = parse(
            r#"
            fn ok(x: Option<i32>) -> Option<i32> {
                x.map(|v| v + 1)
            }
            "#,
        );
        assert!(findings.is_empty(), "{findings:?}");
    }

    #[test]
    fn matches_allowlist_by_selector_not_line() {
        let source = r#"
            fn fixture() -> String {
                std::fs::read_to_string("x").unwrap()
            }
        "#;
        let findings = parse(source);
        let finding = findings.into_iter().next().expect("finding");

        let allowlist_toml = format!(
            r#"
schema_version = "0.3"
[[allow]]
id = "panic-0001"
path = "test.rs"
family = "unwrap"
classification = "test_helper"
owner = "self"
explanation = "fixture builder"
expires = "2999-01-01"

[allow.selector]
kind = "method_call"
container = "fixture"
callee = "unwrap"
receiver_fingerprint = {receiver}
"#,
            receiver = toml_string(&finding.selector.receiver_fingerprint),
        );
        let allowlist: AllowlistFile = toml::from_str(&allowlist_toml).expect("parse allowlist");
        let report = evaluate(&[finding], &allowlist).expect("evaluate");
        assert_eq!(report.unallowlisted.len(), 0, "{report:?}");
        assert_eq!(report.matched_count, 1);
        assert!(report.stale.is_empty(), "{report:?}");
    }

    #[test]
    fn flags_stale_entry() {
        let allowlist_toml = r#"
schema_version = "0.3"
[[allow]]
id = "panic-0002"
path = "missing.rs"
family = "unwrap"
classification = "tooling"
owner = "self"
explanation = "no longer used"
expires = "2999-01-01"

[allow.selector]
kind = "method_call"
container = "gone"
callee = "unwrap"
receiver_fingerprint = "no::longer::here()"
"#;
        let allowlist: AllowlistFile = toml::from_str(allowlist_toml).expect("parse");
        let report = evaluate(&[], &allowlist).expect("evaluate");
        assert_eq!(report.stale.len(), 1, "{report:?}");
        assert!(report.has_errors());
    }

    #[test]
    fn flags_expired_entry() {
        let allowlist_toml = r#"
schema_version = "0.3"
[[allow]]
id = "panic-0003"
path = "test.rs"
family = "unwrap"
classification = "tooling"
owner = "self"
explanation = "old"
expires = "2000-01-01"

[allow.selector]
kind = "method_call"
container = "f"
callee = "unwrap"
receiver_fingerprint = "x"
"#;
        let allowlist: AllowlistFile = toml::from_str(allowlist_toml).expect("parse");
        let report = evaluate(&[], &allowlist).expect("evaluate");
        // Stale + expired both fire (selector is unmatched and date is past).
        assert!(!report.expired.is_empty(), "{report:?}");
    }

    #[test]
    fn flags_unallowlisted_finding() {
        let findings = parse(
            r#"
            fn boom() { panic!("x"); }
            "#,
        );
        let allowlist = AllowlistFile {
            schema_version: SCHEMA_VERSION.to_string(),
            allow: Vec::new(),
        };
        let report = evaluate(&findings, &allowlist).expect("evaluate");
        assert_eq!(report.unallowlisted.len(), 1);
        assert!(report.has_errors());
    }

    #[test]
    fn rejects_unknown_classification() {
        let allowlist_toml = r#"
schema_version = "0.3"
[[allow]]
id = "panic-0004"
path = "test.rs"
family = "unwrap"
classification = "wrong_value"
owner = "self"
explanation = "x"
expires = "2999-01-01"

[allow.selector]
kind = "method_call"
container = "f"
callee = "unwrap"
receiver_fingerprint = "x"
"#;
        let allowlist: AllowlistFile = toml::from_str(allowlist_toml).expect("parse");
        let report = evaluate(&[], &allowlist).expect("evaluate");
        assert!(
            report
                .shape
                .iter()
                .any(|s| s.contains("classification must be one of")),
            "{report:?}"
        );
    }

    #[test]
    fn next_id_picks_next_available() {
        let allowlist = AllowlistFile {
            schema_version: SCHEMA_VERSION.to_string(),
            allow: vec![AllowEntry {
                id: "panic-0007".into(),
                path: "x".into(),
                family: "unwrap".into(),
                classification: "tooling".into(),
                owner: "x".into(),
                explanation: "x".into(),
                expires: "2999-01-01".into(),
                selector: SelectorTable {
                    kind: "method_call".into(),
                    container: "f".into(),
                    callee: "unwrap".into(),
                    receiver_fingerprint: "x".into(),
                },
                last_seen: None,
            }],
        };
        assert_eq!(next_id(&allowlist), 8);
    }
}
