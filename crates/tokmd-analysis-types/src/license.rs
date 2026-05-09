use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseReport {
    pub findings: Vec<LicenseFinding>,
    pub effective: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseFinding {
    pub spdx: String,
    pub confidence: f32,
    pub source_path: String,
    pub source_kind: LicenseSourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseSourceKind {
    Metadata,
    Text,
}
