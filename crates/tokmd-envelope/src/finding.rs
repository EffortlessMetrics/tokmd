//! Portable finding DTOs embedded in sensor reports.

use serde::{Deserialize, Serialize};

/// A finding reported by the sensor.
///
/// Findings use a `(check_id, code)` tuple for identity. Combined with
/// `tool.name` this forms the triple `(tool, check_id, code)` used for
/// buildfix routing and cockpit policy (e.g., `("tokmd", "risk", "hotspot")`).
///
/// # Examples
///
/// ```
/// use tokmd_envelope::{Finding, FindingSeverity, FindingLocation};
///
/// let finding = Finding::new(
///     "risk", "hotspot",
///     FindingSeverity::Warn,
///     "High-churn file",
///     "src/lib.rs modified 42 times in 30 days",
/// ).with_location(FindingLocation::path_line("src/lib.rs", 1));
///
/// assert_eq!(finding.check_id, "risk");
/// assert_eq!(finding.code, "hotspot");
/// assert!(finding.location.is_some());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Check category (e.g., "risk", "contract", "gate").
    pub check_id: String,
    /// Finding code within the category (e.g., "hotspot", "coupling").
    pub code: String,
    /// Severity level.
    pub severity: FindingSeverity,
    /// Short title for the finding.
    pub title: String,
    /// Detailed message describing the finding.
    pub message: String,
    /// Source location (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<FindingLocation>,
    /// Additional evidence data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<serde_json::Value>,
    /// Documentation URL for this finding type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
    /// Stable identity fingerprint for deduplication and buildfix routing.
    /// BLAKE3 hash of (tool_name, check_id, code, location.path).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
}

/// Severity level for findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    /// Blocks merge (hard gate failure).
    Error,
    /// Review recommended.
    Warn,
    /// Informational, no action required.
    Info,
}

/// Source location for a finding.
///
/// # Examples
///
/// ```
/// use tokmd_envelope::FindingLocation;
///
/// // Path only
/// let loc = FindingLocation::path("src/main.rs");
/// assert_eq!(loc.path, "src/main.rs");
/// assert!(loc.line.is_none());
///
/// // Path + line
/// let loc = FindingLocation::path_line("src/lib.rs", 42);
/// assert_eq!(loc.line, Some(42));
///
/// // Path + line + column
/// let loc = FindingLocation::path_line_column("src/lib.rs", 42, 10);
/// assert_eq!(loc.column, Some(10));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingLocation {
    /// File path (normalized to forward slashes).
    pub path: String,
    /// Line number (1-indexed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Column number (1-indexed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

impl Finding {
    /// Create a new finding with required fields.
    pub fn new(
        check_id: impl Into<String>,
        code: impl Into<String>,
        severity: FindingSeverity,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            check_id: check_id.into(),
            code: code.into(),
            severity,
            title: title.into(),
            message: message.into(),
            location: None,
            evidence: None,
            docs_url: None,
            fingerprint: None,
        }
    }

    /// Add a location to the finding.
    pub fn with_location(mut self, location: FindingLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add evidence to the finding.
    pub fn with_evidence(mut self, evidence: serde_json::Value) -> Self {
        self.evidence = Some(evidence);
        self
    }

    /// Add a documentation URL to the finding.
    pub fn with_docs_url(mut self, url: impl Into<String>) -> Self {
        self.docs_url = Some(url.into());
        self
    }

    /// Compute a stable fingerprint from `(tool_name, check_id, code, path)`.
    ///
    /// Returns first 16 bytes (32 hex chars) of a BLAKE3 hash for compactness.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd_envelope::{Finding, FindingSeverity, FindingLocation};
    ///
    /// let f = Finding::new("risk", "hotspot", FindingSeverity::Warn, "Churn", "high")
    ///     .with_location(FindingLocation::path("src/lib.rs"));
    /// let fp = f.compute_fingerprint("tokmd");
    /// assert_eq!(fp.len(), 32);
    ///
    /// // Same inputs produce same fingerprint
    /// let f2 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "Churn", "high")
    ///     .with_location(FindingLocation::path("src/lib.rs"));
    /// assert_eq!(f2.compute_fingerprint("tokmd"), fp);
    /// ```
    pub fn compute_fingerprint(&self, tool_name: &str) -> String {
        let path = self
            .location
            .as_ref()
            .map(|l| l.path.as_str())
            .unwrap_or("");
        let identity = format!("{}\0{}\0{}\0{}", tool_name, self.check_id, self.code, path);
        let hash = blake3::hash(identity.as_bytes());
        let hex = hash.to_hex();
        hex[..32].to_string()
    }

    /// Auto-compute and set fingerprint. Builder pattern.
    pub fn with_fingerprint(mut self, tool_name: &str) -> Self {
        self.fingerprint = Some(self.compute_fingerprint(tool_name));
        self
    }
}

impl FindingLocation {
    /// Create a new location with just a path.
    pub fn path(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            line: None,
            column: None,
        }
    }

    /// Create a new location with path and line.
    pub fn path_line(path: impl Into<String>, line: u32) -> Self {
        Self {
            path: path.into(),
            line: Some(line),
            column: None,
        }
    }

    /// Create a new location with path, line, and column.
    pub fn path_line_column(path: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            path: path.into(),
            line: Some(line),
            column: Some(column),
        }
    }
}

impl std::fmt::Display for FindingSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindingSeverity::Error => write!(f, "error"),
            FindingSeverity::Warn => write!(f, "warn"),
            FindingSeverity::Info => write!(f, "info"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Finding, FindingLocation, FindingSeverity};
    use crate::findings;

    #[test]
    fn finding_severity_display_matches_serde() {
        for (variant, expected) in [
            (FindingSeverity::Error, "error"),
            (FindingSeverity::Warn, "warn"),
            (FindingSeverity::Info, "info"),
        ] {
            assert_eq!(variant.to_string(), expected);
            let json = serde_json::to_value(variant).unwrap();
            assert_eq!(json.as_str().unwrap(), expected);
        }
    }

    #[test]
    fn finding_builders_and_fingerprint() {
        let location = FindingLocation::path_line_column("src/lib.rs", 10, 2);
        let finding = Finding::new(
            findings::risk::CHECK_ID,
            findings::risk::COUPLING,
            FindingSeverity::Info,
            "Coupled module",
            "Modules share excessive dependencies",
        )
        .with_location(location.clone())
        .with_evidence(serde_json::json!({ "coupling": 0.87 }))
        .with_docs_url("https://example.com/docs/coupling");

        let expected_identity = format!(
            "{}\0{}\0{}\0{}",
            "tokmd",
            findings::risk::CHECK_ID,
            findings::risk::COUPLING,
            location.path
        );
        let expected_hash = blake3::hash(expected_identity.as_bytes()).to_hex();
        let expected_fingerprint = expected_hash[..32].to_string();

        assert_eq!(finding.compute_fingerprint("tokmd"), expected_fingerprint);

        let with_fp = finding.clone().with_fingerprint("tokmd");
        assert_eq!(
            with_fp.fingerprint.as_deref(),
            Some(expected_fingerprint.as_str())
        );

        let no_location = Finding::new(
            findings::risk::CHECK_ID,
            findings::risk::HOTSPOT,
            FindingSeverity::Warn,
            "Hotspot",
            "Churn is elevated",
        );
        assert_ne!(
            no_location.compute_fingerprint("tokmd"),
            finding.compute_fingerprint("tokmd")
        );
    }

    #[test]
    fn finding_location_constructors() {
        let path_only = FindingLocation::path("src/main.rs");
        assert_eq!(path_only.path, "src/main.rs");
        assert_eq!(path_only.line, None);
        assert_eq!(path_only.column, None);

        let path_line = FindingLocation::path_line("src/main.rs", 42);
        assert_eq!(path_line.path, "src/main.rs");
        assert_eq!(path_line.line, Some(42));
        assert_eq!(path_line.column, None);

        let path_line_column = FindingLocation::path_line_column("src/main.rs", 7, 3);
        assert_eq!(path_line_column.path, "src/main.rs");
        assert_eq!(path_line_column.line, Some(7));
        assert_eq!(path_line_column.column, Some(3));
    }

    #[test]
    fn finding_omits_optional_fields_when_none() {
        let finding = Finding::new(
            findings::risk::CHECK_ID,
            findings::risk::HOTSPOT,
            FindingSeverity::Warn,
            "Hotspot",
            "Churn is elevated",
        );
        let json = serde_json::to_value(&finding).unwrap();

        assert_eq!(json["check_id"], findings::risk::CHECK_ID);
        assert_eq!(json["code"], findings::risk::HOTSPOT);
        assert_eq!(json["severity"], "warn");
        assert!(json.get("location").is_none());
        assert!(json.get("evidence").is_none());
        assert!(json.get("docs_url").is_none());
        assert!(json.get("fingerprint").is_none());
    }
}
