use super::capability::AstLanguage;
use super::facts::{SyntaxFacts, SyntaxRiskSeam, SyntaxSpan, SyntaxSymbol};
use serde_json::{Value, json};

const PANIC_SEAM_KINDS: &[&str] = &[
    "unwrap",
    "expect",
    "fallible_conversion_expect",
    "indexing",
    "capacity_allocation",
    "panic_macro",
    "assert_macro",
    "unreachable_macro",
    "todo_macro",
    "unimplemented_macro",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GuardStatus {
    Guarded,
    Unguarded,
}

impl GuardStatus {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Guarded => "guarded",
            Self::Unguarded => "unguarded",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputSource {
    Parameter,
    Constant,
    Internal,
    JsArgSuspect,
}

impl InputSource {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Parameter => "parameter",
            Self::Constant => "constant",
            Self::Internal => "internal",
            Self::JsArgSuspect => "js_arg_suspect",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FailureMode {
    Abort,
    OutOfBounds,
    CapacityOverflow,
    AssertionTrap,
    Unknown,
}

impl FailureMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Abort => "abort",
            Self::OutOfBounds => "out_of_bounds",
            Self::CapacityOverflow => "capacity_overflow",
            Self::AssertionTrap => "assertion_trap",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PanicSeamEntry {
    kind: String,
    evidence: String,
    span: SyntaxSpan,
    entry_symbol: Option<String>,
    public_surface: bool,
    guard_status: GuardStatus,
    input_source: InputSource,
    input_hints: Vec<String>,
    failure_mode: FailureMode,
}

impl PanicSeamEntry {
    fn to_value(&self) -> Value {
        json!({
            "kind": self.kind,
            "evidence": self.evidence,
            "span": self.span.to_value(),
            "entry_symbol": self.entry_symbol.as_deref(),
            "public_surface": self.public_surface,
            "guard_status": self.guard_status.as_str(),
            "input_source": self.input_source.as_str(),
            "input_hints": self.input_hints,
            "failure_mode": self.failure_mode.as_str(),
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct PanicSeamCounts {
    total: usize,
    unguarded: usize,
    guarded: usize,
    public_surface: usize,
    js_arg_suspect: usize,
}

impl PanicSeamCounts {
    fn to_value(&self) -> Value {
        json!({
            "total": self.total,
            "unguarded": self.unguarded,
            "guarded": self.guarded,
            "public_surface": self.public_surface,
            "js_arg_suspect": self.js_arg_suspect,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PanicSeamSummary {
    entries: Vec<PanicSeamEntry>,
    counts: PanicSeamCounts,
}

impl PanicSeamSummary {
    #[must_use]
    pub fn to_value(&self) -> Value {
        json!({
            "non_claims": [
                "syntax-only seam inventory; does not prove public reachability, interprocedural paths, runtime guard correctness, or bug presence"
            ],
            "entries": self.entries.iter().map(PanicSeamEntry::to_value).collect::<Vec<_>>(),
            "counts": self.counts.to_value(),
        })
    }
}

#[must_use]
pub fn derive_panic_seam_summary(
    facts: &SyntaxFacts,
    language: Option<AstLanguage>,
) -> Option<PanicSeamSummary> {
    let panic_seams: Vec<&SyntaxRiskSeam> = facts
        .risk_seams
        .iter()
        .filter(|seam| PANIC_SEAM_KINDS.contains(&seam.kind.as_str()))
        .collect();

    if panic_seams.is_empty() {
        return None;
    }

    let guard_spans: Vec<SyntaxSpan> = facts
        .risk_seams
        .iter()
        .filter(|seam| seam.kind == "guard_evidence")
        .map(|seam| seam.span)
        .collect();

    let mut entries = Vec::new();
    let mut counts = PanicSeamCounts::default();

    for seam in panic_seams {
        let containing = containing_symbol(&facts.symbols, seam.span);
        let guard_status = guard_status_for(seam.span, &guard_spans);
        let input_hints = input_hints_from_evidence(&seam.evidence, containing);
        let input_source =
            classify_input_source(language, containing, &input_hints, &seam.evidence);
        let failure_mode = failure_mode_for(&seam.kind);
        let public_surface = containing
            .as_ref()
            .is_some_and(|symbol| symbol.public_surface);

        counts.total += 1;
        match guard_status {
            GuardStatus::Guarded => counts.guarded += 1,
            GuardStatus::Unguarded => counts.unguarded += 1,
        }
        if public_surface {
            counts.public_surface += 1;
        }
        if input_source == InputSource::JsArgSuspect {
            counts.js_arg_suspect += 1;
        }

        entries.push(PanicSeamEntry {
            kind: seam.kind.clone(),
            evidence: seam.evidence.clone(),
            span: seam.span,
            entry_symbol: containing.as_ref().map(|symbol| symbol.name.clone()),
            public_surface,
            guard_status,
            input_source,
            input_hints,
            failure_mode,
        });
    }

    entries.sort_by(|left, right| {
        left.span
            .start_line
            .cmp(&right.span.start_line)
            .then_with(|| left.span.start_column.cmp(&right.span.start_column))
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.evidence.cmp(&right.evidence))
    });

    Some(PanicSeamSummary { entries, counts })
}

fn containing_symbol(symbols: &[SyntaxSymbol], seam_span: SyntaxSpan) -> Option<&SyntaxSymbol> {
    symbols
        .iter()
        .filter(|symbol| {
            symbol.span.start_line <= seam_span.start_line
                && symbol.span.end_line >= seam_span.start_line
        })
        .max_by_key(|symbol| symbol.span.start_line)
}

fn guard_status_for(seam_span: SyntaxSpan, guard_spans: &[SyntaxSpan]) -> GuardStatus {
    let guarded = guard_spans.iter().any(|guard| {
        guard.start_line <= seam_span.start_line && guard.end_line >= seam_span.start_line
    });
    if guarded {
        GuardStatus::Guarded
    } else {
        GuardStatus::Unguarded
    }
}

fn input_hints_from_evidence(evidence: &str, containing: Option<&SyntaxSymbol>) -> Vec<String> {
    let mut hints = Vec::new();
    let params = containing
        .map(|symbol| symbol.parameters.as_slice())
        .unwrap_or(&[]);

    for param in params {
        if evidence_contains_identifier(evidence, param) {
            hints.push(param.clone());
        }
    }

    if hints.is_empty() {
        for token in evidence
            .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
            .filter(|token| looks_like_parameter(token))
        {
            if !hints.iter().any(|hint| hint == token) {
                hints.push(token.to_owned());
            }
        }
    }

    hints.sort();
    hints.dedup();
    hints
}

fn evidence_contains_identifier(evidence: &str, identifier: &str) -> bool {
    evidence
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .any(|token| token == identifier)
}

fn looks_like_parameter(token: &str) -> bool {
    if token.is_empty()
        || token
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
    {
        return false;
    }
    !matches!(
        token,
        "if" | "else"
            | "match"
            | "let"
            | "mut"
            | "return"
            | "self"
            | "Self"
            | "true"
            | "false"
            | "Some"
            | "None"
            | "Ok"
            | "Err"
            | "usize"
            | "isize"
            | "i64"
            | "u64"
            | "i32"
            | "u32"
            | "Vec"
            | "CString"
            | "try_from"
            | "try_into"
            | "with_capacity"
            | "reserve"
            | "expect"
            | "unwrap"
            | "new"
            | "len"
            | "is_null"
            | "panic"
            | "assert"
    )
}

fn classify_input_source(
    language: Option<AstLanguage>,
    containing: Option<&SyntaxSymbol>,
    hints: &[String],
    evidence: &str,
) -> InputSource {
    if evidence.contains('"') && hints.is_empty() {
        return InputSource::Constant;
    }

    if hints.is_empty() {
        return InputSource::Internal;
    }

    if language == Some(AstLanguage::Rust) && containing.is_some_and(|symbol| symbol.ffi_entry) {
        return InputSource::JsArgSuspect;
    }

    InputSource::Parameter
}

fn failure_mode_for(kind: &str) -> FailureMode {
    match kind {
        "indexing" => FailureMode::OutOfBounds,
        "capacity_allocation" => FailureMode::CapacityOverflow,
        "assert_macro" => FailureMode::AssertionTrap,
        "unwrap"
        | "expect"
        | "fallible_conversion_expect"
        | "panic_macro"
        | "unreachable_macro"
        | "todo_macro"
        | "unimplemented_macro" => FailureMode::Abort,
        _ => FailureMode::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::derive_panic_seam_summary;
    use crate::ast::{SyntaxParseOptions, parse_syntax_receipt};

    #[test]
    fn panic_seam_fixture_surfaces_guarded_and_unguarded_entries() {
        let receipt = parse_syntax_receipt(
            "src/runtime/panic_seams.rs",
            include_str!("../../../../fixtures/syntax/rust/panic_seams.rs"),
            SyntaxParseOptions::default(),
        );
        let summary = derive_panic_seam_summary(&receipt.facts, receipt.language)
            .expect("fixture should emit panic seams");

        let value = summary.to_value();
        let entries = value["entries"].as_array().expect("entries array");
        assert!(!entries.is_empty());
        assert_eq!(
            value["counts"]["total"].as_u64(),
            Some(entries.len() as u64)
        );

        let guarded_indexing = entries
            .iter()
            .find(|entry| entry["kind"] == "indexing" && entry["guard_status"] == "guarded");
        assert!(
            guarded_indexing.is_some(),
            "indexing inside if guard should be marked guarded"
        );

        let unguarded_expect = entries.iter().find(|entry| {
            entry["kind"] == "expect" && entry["evidence"].as_str().unwrap().contains("raw")
        });
        assert_eq!(unguarded_expect.unwrap()["guard_status"], "unguarded");
        assert_eq!(unguarded_expect.unwrap()["input_source"], "parameter");
        assert_eq!(unguarded_expect.unwrap()["entry_symbol"], "load_packet");
    }

    #[test]
    fn ffi_entry_marks_js_arg_suspect_for_unguarded_assert_seam() {
        let receipt = parse_syntax_receipt(
            "src/runtime/panic_seams.rs",
            include_str!("../../../../fixtures/syntax/rust/panic_seams.rs"),
            SyntaxParseOptions::default(),
        );
        let summary = derive_panic_seam_summary(&receipt.facts, receipt.language)
            .expect("fixture should emit panic seams");

        let summary_value = summary.to_value();
        let ffi_assert = summary_value["entries"]
            .as_array()
            .expect("entries")
            .iter()
            .find(|entry| entry["kind"] == "assert_macro" && entry["entry_symbol"] == "ffi_entry")
            .expect("ffi_entry assert seam");
        assert_eq!(ffi_assert["guard_status"], "unguarded");
        assert_eq!(ffi_assert["input_source"], "js_arg_suspect");
        assert!(
            ffi_assert["input_hints"]
                .as_array()
                .unwrap()
                .iter()
                .any(|hint| hint == "ptr")
        );
        assert_eq!(ffi_assert["public_surface"], true);
    }

    #[test]
    fn empty_facts_emit_no_summary() {
        use crate::ast::SyntaxFacts;
        assert!(derive_panic_seam_summary(&SyntaxFacts::default(), None).is_none());
    }
}
