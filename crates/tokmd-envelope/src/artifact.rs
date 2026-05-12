//! Artifact references attached to sensor reports.

use serde::{Deserialize, Serialize};

/// Artifact reference in the sensor report.
///
/// # Examples
///
/// ```
/// use tokmd_envelope::Artifact;
///
/// let art = Artifact::receipt("output/receipt.json")
///     .with_id("analysis")
///     .with_mime("application/json");
/// assert_eq!(art.artifact_type, "receipt");
/// assert_eq!(art.id.as_deref(), Some("analysis"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact identifier (e.g., "analysis", "handoff").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Artifact type (e.g., "comment", "receipt", "badge").
    #[serde(rename = "type")]
    pub artifact_type: String,
    /// Path to the artifact file.
    pub path: String,
    /// MIME type (e.g., "application/json").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
}

impl Artifact {
    /// Create a new artifact reference.
    pub fn new(artifact_type: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: None,
            artifact_type: artifact_type.into(),
            path: path.into(),
            mime: None,
        }
    }

    /// Create a comment artifact.
    pub fn comment(path: impl Into<String>) -> Self {
        Self::new("comment", path)
    }

    /// Create a receipt artifact.
    pub fn receipt(path: impl Into<String>) -> Self {
        Self::new("receipt", path)
    }

    /// Create a badge artifact.
    pub fn badge(path: impl Into<String>) -> Self {
        Self::new("badge", path)
    }

    /// Set the artifact ID. Builder pattern.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the MIME type. Builder pattern.
    pub fn with_mime(mut self, mime: impl Into<String>) -> Self {
        self.mime = Some(mime.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::Artifact;

    #[test]
    fn artifact_builders_cover_variants() {
        let custom = Artifact::new("custom", "out/custom.json");
        assert_eq!(custom.artifact_type, "custom");
        assert_eq!(custom.path, "out/custom.json");

        let comment = Artifact::comment("out/comment.md");
        assert_eq!(comment.artifact_type, "comment");
        assert_eq!(comment.path, "out/comment.md");

        let receipt = Artifact::receipt("out/receipt.json")
            .with_id("receipt")
            .with_mime("application/json");
        assert_eq!(receipt.artifact_type, "receipt");
        assert_eq!(receipt.path, "out/receipt.json");
        assert_eq!(receipt.id.as_deref(), Some("receipt"));
        assert_eq!(receipt.mime.as_deref(), Some("application/json"));

        let badge = Artifact::badge("out/badge.svg");
        assert_eq!(badge.artifact_type, "badge");
        assert_eq!(badge.path, "out/badge.svg");
    }
}
