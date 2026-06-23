use serde_json::{Value, json};
use tree_sitter::Node;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SyntaxSpan {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

impl SyntaxSpan {
    #[must_use]
    pub fn from_node(node: Node<'_>) -> Self {
        let start = node.start_position();
        let end = node.end_position();
        Self {
            start_line: start.row + 1,
            start_column: start.column + 1,
            end_line: end.row + 1,
            end_column: end.column + 1,
        }
    }

    #[must_use]
    pub fn to_value(self) -> Value {
        json!({
            "start_line": self.start_line,
            "start_column": self.start_column,
            "end_line": self.end_line,
            "end_column": self.end_column,
        })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SyntaxSymbol {
    pub kind: String,
    pub name: String,
    pub span: SyntaxSpan,
    pub exported: bool,
    pub public_surface: bool,
}

impl SyntaxSymbol {
    #[must_use]
    pub fn to_value(&self) -> Value {
        json!({
            "kind": self.kind,
            "name": self.name,
            "span": self.span.to_value(),
            "exported": self.exported,
            "public_surface": self.public_surface,
        })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SyntaxImport {
    pub kind: String,
    pub module: Option<String>,
    pub imported: Vec<String>,
    pub dynamic: bool,
    pub span: SyntaxSpan,
}

impl SyntaxImport {
    #[must_use]
    pub fn to_value(&self) -> Value {
        json!({
            "kind": self.kind,
            "module": self.module,
            "imported": self.imported,
            "dynamic": self.dynamic,
            "span": self.span.to_value(),
        })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SyntaxExport {
    pub kind: String,
    pub name: String,
    pub span: SyntaxSpan,
}

impl SyntaxExport {
    #[must_use]
    pub fn to_value(&self) -> Value {
        json!({
            "kind": self.kind,
            "name": self.name,
            "span": self.span.to_value(),
        })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SyntaxCallSite {
    pub kind: String,
    pub callee: String,
    pub dynamic: bool,
    pub span: SyntaxSpan,
}

impl SyntaxCallSite {
    #[must_use]
    pub fn to_value(&self) -> Value {
        json!({
            "kind": self.kind,
            "callee": self.callee,
            "dynamic": self.dynamic,
            "span": self.span.to_value(),
        })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SyntaxRiskSeam {
    pub kind: String,
    pub evidence: String,
    pub span: SyntaxSpan,
    pub test_context: bool,
}

impl SyntaxRiskSeam {
    #[must_use]
    pub fn to_value(&self) -> Value {
        if self.test_context {
            json!({
                "kind": self.kind,
                "evidence": self.evidence,
                "span": self.span.to_value(),
                "test_context": true,
            })
        } else {
            json!({
                "kind": self.kind,
                "evidence": self.evidence,
                "span": self.span.to_value(),
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SyntaxReviewSeverity {
    High,
    Medium,
    Low,
}

impl SyntaxReviewSeverity {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    #[must_use]
    pub const fn score(self) -> u8 {
        match self {
            Self::High => 90,
            Self::Medium => 60,
            Self::Low => 30,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SyntaxReviewSignal {
    pub severity: SyntaxReviewSeverity,
    pub category: String,
    pub kind: String,
    pub reason: String,
    pub evidence: String,
    pub span: SyntaxSpan,
    pub test_context: bool,
}

impl SyntaxReviewSignal {
    #[must_use]
    pub fn effective_severity(&self) -> SyntaxReviewSeverity {
        if self.test_context && self.category == "panic_seam" {
            SyntaxReviewSeverity::Low
        } else {
            self.severity
        }
    }

    #[must_use]
    pub fn effective_reason(&self) -> String {
        if self.test_context && self.category == "panic_seam" {
            format!(
                "{} (test-only context; deprioritized for panic/native review ranking)",
                self.reason
            )
        } else {
            self.reason.clone()
        }
    }

    #[must_use]
    pub fn to_value(&self) -> Value {
        let severity = self.effective_severity();
        if self.test_context {
            json!({
                "category": self.category,
                "severity": severity.as_str(),
                "score": severity.score(),
                "kind": self.kind,
                "reason": self.effective_reason(),
                "evidence": self.evidence,
                "span": self.span.to_value(),
                "test_context": true,
            })
        } else {
            json!({
                "category": self.category,
                "severity": severity.as_str(),
                "score": severity.score(),
                "kind": self.kind,
                "reason": self.effective_reason(),
                "evidence": self.evidence,
                "span": self.span.to_value(),
            })
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SyntaxFacts {
    pub symbols: Vec<SyntaxSymbol>,
    pub imports: Vec<SyntaxImport>,
    pub exports: Vec<SyntaxExport>,
    pub call_sites: Vec<SyntaxCallSite>,
    pub risk_seams: Vec<SyntaxRiskSeam>,
}

impl SyntaxFacts {
    pub fn normalize(&mut self) {
        self.symbols.sort();
        self.symbols.dedup();
        self.imports.sort();
        self.imports.dedup();
        self.exports.sort();
        self.exports.dedup();
        self.call_sites.sort();
        self.call_sites.dedup();
        self.risk_seams.sort();
        self.risk_seams.dedup();
    }

    #[must_use]
    pub fn symbols_value(&self) -> Vec<Value> {
        self.symbols.iter().map(SyntaxSymbol::to_value).collect()
    }

    #[must_use]
    pub fn imports_value(&self) -> Vec<Value> {
        self.imports.iter().map(SyntaxImport::to_value).collect()
    }

    #[must_use]
    pub fn exports_value(&self) -> Vec<Value> {
        self.exports.iter().map(SyntaxExport::to_value).collect()
    }

    #[must_use]
    pub fn call_sites_value(&self) -> Vec<Value> {
        self.call_sites
            .iter()
            .map(SyntaxCallSite::to_value)
            .collect()
    }

    #[must_use]
    pub fn risk_seams_value(&self) -> Vec<Value> {
        self.risk_seams
            .iter()
            .map(SyntaxRiskSeam::to_value)
            .collect()
    }

    #[must_use]
    pub fn review_signals(&self) -> Vec<SyntaxReviewSignal> {
        let mut signals = Vec::new();

        for symbol in &self.symbols {
            if symbol.public_surface {
                signals.push(SyntaxReviewSignal {
                    severity: SyntaxReviewSeverity::Medium,
                    category: "public_surface".to_owned(),
                    kind: symbol.kind.clone(),
                    reason: "public/API-ish symbol should be reviewed before private implementation details"
                        .to_owned(),
                    evidence: symbol.name.clone(),
                    span: symbol.span,
                    test_context: false,
                });
            } else if symbol.exported {
                signals.push(SyntaxReviewSignal {
                    severity: SyntaxReviewSeverity::Low,
                    category: "exported_symbol".to_owned(),
                    kind: symbol.kind.clone(),
                    reason: "exported symbol contributes to review surface".to_owned(),
                    evidence: symbol.name.clone(),
                    span: symbol.span,
                    test_context: false,
                });
            }
        }

        for import in &self.imports {
            if import.dynamic {
                signals.push(SyntaxReviewSignal {
                    severity: SyntaxReviewSeverity::Medium,
                    category: "dynamic_import".to_owned(),
                    kind: import.kind.clone(),
                    reason: "dynamic import changes runtime load behavior".to_owned(),
                    evidence: import
                        .module
                        .clone()
                        .unwrap_or_else(|| "dynamic import".to_owned()),
                    span: import.span,
                    test_context: false,
                });
            }

            if import
                .module
                .as_deref()
                .is_some_and(looks_native_boundary_evidence)
                || import
                    .imported
                    .iter()
                    .any(|name| looks_native_boundary_evidence(name))
            {
                signals.push(SyntaxReviewSignal {
                    severity: SyntaxReviewSeverity::High,
                    category: "native_boundary".to_owned(),
                    kind: import.kind.clone(),
                    reason: "import names a native, FFI, or binding-ish boundary".to_owned(),
                    evidence: import
                        .module
                        .clone()
                        .unwrap_or_else(|| import.imported.join(", ")),
                    span: import.span,
                    test_context: false,
                });
            }
        }

        for call_site in &self.call_sites {
            if call_site.dynamic {
                signals.push(SyntaxReviewSignal {
                    severity: SyntaxReviewSeverity::Medium,
                    category: "dynamic_execution".to_owned(),
                    kind: call_site.kind.clone(),
                    reason: "dynamic call site may obscure runtime target".to_owned(),
                    evidence: call_site.callee.clone(),
                    span: call_site.span,
                    test_context: false,
                });
            }
            if looks_native_boundary_evidence(&call_site.callee) {
                signals.push(SyntaxReviewSignal {
                    severity: SyntaxReviewSeverity::High,
                    category: "native_boundary".to_owned(),
                    kind: call_site.kind.clone(),
                    reason: "call site names a native, FFI, or binding-ish boundary".to_owned(),
                    evidence: call_site.callee.clone(),
                    span: call_site.span,
                    test_context: false,
                });
            }
        }

        for seam in &self.risk_seams {
            let (severity, category, reason) = review_signal_for_risk_kind(&seam.kind);
            signals.push(SyntaxReviewSignal {
                severity,
                category: category.to_owned(),
                kind: seam.kind.clone(),
                reason: reason.to_owned(),
                evidence: seam.evidence.clone(),
                span: seam.span,
                test_context: seam.test_context,
            });
        }

        signals.sort_by(|left, right| {
            right
                .effective_severity()
                .score()
                .cmp(&left.effective_severity().score())
                .then_with(|| left.category.cmp(&right.category))
                .then_with(|| left.kind.cmp(&right.kind))
                .then_with(|| left.evidence.cmp(&right.evidence))
                .then_with(|| left.span.cmp(&right.span))
        });
        signals.dedup();
        signals
    }

    #[must_use]
    pub fn review_signals_value(&self) -> Vec<Value> {
        self.review_signals()
            .iter()
            .map(SyntaxReviewSignal::to_value)
            .collect()
    }
}

fn review_signal_for_risk_kind(kind: &str) -> (SyntaxReviewSeverity, &'static str, &'static str) {
    match kind {
        "native_boundary_hint" => (
            SyntaxReviewSeverity::High,
            "native_boundary",
            "native, FFI, or binding-ish boundary hint",
        ),
        "subprocess_call" => (
            SyntaxReviewSeverity::High,
            "process_boundary",
            "subprocess or shell boundary should be reviewed early",
        ),
        "dynamic_eval" => (
            SyntaxReviewSeverity::High,
            "dynamic_execution",
            "dynamic code execution should be reviewed early",
        ),
        "unwrap"
        | "expect"
        | "fallible_conversion_expect"
        | "indexing"
        | "capacity_allocation"
        | "panic_macro"
        | "assert_macro"
        | "unreachable_macro"
        | "todo_macro" => (
            SyntaxReviewSeverity::High,
            "panic_seam",
            "panic, assertion, indexing, or allocation seam can abort or trap",
        ),
        "risky_cast" | "non_null_assertion" => (
            SyntaxReviewSeverity::Medium,
            "type_assertion",
            "type assertion or non-null assertion can hide runtime mismatch",
        ),
        "dynamic_call" => (
            SyntaxReviewSeverity::Medium,
            "dynamic_execution",
            "dynamic call may obscure runtime target",
        ),
        "dynamic_import" => (
            SyntaxReviewSeverity::Medium,
            "dynamic_import",
            "dynamic import changes runtime load behavior",
        ),
        "file_io" => (
            SyntaxReviewSeverity::Medium,
            "io_boundary",
            "file I/O boundary may depend on runtime inputs",
        ),
        "exception_raise" | "exception_handler" => (
            SyntaxReviewSeverity::Medium,
            "exception_path",
            "exception path affects error and recovery behavior",
        ),
        "entrypoint" => (
            SyntaxReviewSeverity::Medium,
            "entrypoint",
            "entrypoint-like code concentrates review impact",
        ),
        "fallible_conversion" => (
            SyntaxReviewSeverity::Medium,
            "fallible_conversion",
            "fallible conversion changes type or bounds assumptions",
        ),
        "guard_evidence" => (
            SyntaxReviewSeverity::Low,
            "guard_evidence",
            "nearby guard evidence may bound a higher-risk seam",
        ),
        _ => (
            SyntaxReviewSeverity::Medium,
            "language_risk",
            "language-specific risk seam should be reviewed",
        ),
    }
}

fn looks_native_boundary_evidence(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("native")
        || lower.contains("binding")
        || contains_ffi_hint(&lower)
        || lower.contains("ctypes")
        || lower.contains("cffi")
        || lower.contains("dlopen")
        || lower.contains("cdll")
        || lower.contains("pydll")
}

fn contains_ffi_hint(value: &str) -> bool {
    value == "ffi"
        || value.starts_with("ffi")
        || value.contains(":ffi")
        || value.contains("/ffi")
        || value.contains("-ffi")
        || value.contains("_ffi")
        || value.contains(".ffi")
        || value.contains("ffi:")
        || value.contains("ffi/")
        || value.contains("ffi-")
        || value.contains("ffi_")
        || value.contains("ffi.")
}

#[cfg(test)]
mod tests {
    use super::looks_native_boundary_evidence;

    #[test]
    fn native_boundary_hint_avoids_interior_ffi_noise() {
        assert!(!looks_native_boundary_evidence("efficient_parser"));
        assert!(!looks_native_boundary_evidence("office_cache"));
        assert!(looks_native_boundary_evidence("bun:ffi"));
        assert!(looks_native_boundary_evidence("ffi_object"));
        assert!(looks_native_boundary_evidence("nativeBinding"));
        assert!(looks_native_boundary_evidence("ctypes.CDLL"));
    }
}
