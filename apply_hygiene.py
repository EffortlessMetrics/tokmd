"""Apply contract-hygiene improvements to Tier 0 crates using absolute paths."""
import os

BASE = r"C:\Code\Rust\tokmd"

def read(rel_path):
    full = os.path.join(BASE, rel_path)
    with open(full, 'r', encoding='utf-8') as f:
        return f.read()

def write(rel_path, content):
    full = os.path.join(BASE, rel_path)
    with open(full, 'w', encoding='utf-8', newline='\n') as f:
        f.write(content)

def replace_once(content, old, new, label=""):
    if old not in content:
        raise ValueError(f"NOT FOUND: {label!r}")
    count = content.count(old)
    if count != 1:
        raise ValueError(f"Expected 1 occurrence for {label!r}, found {count}")
    return content.replace(old, new)


# ============================================================================
# 1. tokmd-types/src/lib.rs
# ============================================================================
p = r"crates\tokmd-types\src\lib.rs"
c = read(p)

# --- PartialEq, Eq derives ---
c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ScanArgs {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct ScanArgs {',
    "ScanArgs")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct LangArgsMeta {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct LangArgsMeta {',
    "LangArgsMeta")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ModuleArgsMeta {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct ModuleArgsMeta {',
    "ModuleArgsMeta")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ExportArgsMeta {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct ExportArgsMeta {',
    "ExportArgsMeta")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct LangReport {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct LangReport {',
    "LangReport")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ModuleReport {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct ModuleReport {',
    "ModuleReport")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ExportData {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct ExportData {',
    "ExportData")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RunReceipt {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct RunReceipt {',
    "RunReceipt")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct PolicyExcludedFile {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct PolicyExcludedFile {',
    "PolicyExcludedFile")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct SmartExcludedFile {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct SmartExcludedFile {',
    "SmartExcludedFile")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ContextExcludedPath {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct ContextExcludedPath {',
    "ContextExcludedPath")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct HandoffExcludedPath {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct HandoffExcludedPath {',
    "HandoffExcludedPath")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct CapabilityStatus {\n    pub name: String,',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct CapabilityStatus {\n    pub name: String,',
    "CapabilityStatus (types)")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ArtifactEntry {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct ArtifactEntry {',
    "ArtifactEntry")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ArtifactHash {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct ArtifactHash {',
    "ArtifactHash")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize, Default)]\npub struct ToolInfo {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]\npub struct ToolInfo {',
    "ToolInfo")

# ScanStatus enum
c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\n#[serde(rename_all = "snake_case")]\npub enum ScanStatus {\n    Complete,\n    Partial,\n}',
    '#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(rename_all = "snake_case")]\npub enum ScanStatus {\n    Complete,\n    Partial,\n}\n\nimpl std::fmt::Display for ScanStatus {\n    fn fmt(&self, f: &mut std::fmt::Formatter<\'_>) -> std::fmt::Result {\n        match self {\n            ScanStatus::Complete => write!(f, "complete"),\n            ScanStatus::Partial => write!(f, "partial"),\n        }\n    }\n}',
    "ScanStatus")

# FileClassification Display
c = replace_once(c,
    '    /// *.js.map, *.css.map.\n    Sourcemap,\n}\n\n/// How a file is included',
    '    /// *.js.map, *.css.map.\n    Sourcemap,\n}\n\nimpl std::fmt::Display for FileClassification {\n    fn fmt(&self, f: &mut std::fmt::Formatter<\'_>) -> std::fmt::Result {\n        match self {\n            FileClassification::Generated => write!(f, "generated"),\n            FileClassification::Fixture => write!(f, "fixture"),\n            FileClassification::Vendored => write!(f, "vendored"),\n            FileClassification::Lockfile => write!(f, "lockfile"),\n            FileClassification::Minified => write!(f, "minified"),\n            FileClassification::DataBlob => write!(f, "data_blob"),\n            FileClassification::Sourcemap => write!(f, "sourcemap"),\n        }\n    }\n}\n\n/// How a file is included',
    "FileClassification Display")

# InclusionPolicy Display
c = replace_once(c,
    '    /// Excluded from payload entirely.\n    Skip,\n}\n\n/// Helper for serde skip_serializing_if',
    '    /// Excluded from payload entirely.\n    Skip,\n}\n\nimpl std::fmt::Display for InclusionPolicy {\n    fn fmt(&self, f: &mut std::fmt::Formatter<\'_>) -> std::fmt::Result {\n        match self {\n            InclusionPolicy::Full => write!(f, "full"),\n            InclusionPolicy::HeadTail => write!(f, "head_tail"),\n            InclusionPolicy::Summary => write!(f, "summary"),\n            InclusionPolicy::Skip => write!(f, "skip"),\n        }\n    }\n}\n\n/// Helper for serde skip_serializing_if',
    "InclusionPolicy Display")

# CapabilityState Display
c = replace_once(c,
    '    /// Capability is unavailable (e.g., not in a git repo).\n    Unavailable,\n}\n\n/// Entry describing an artifact',
    '    /// Capability is unavailable (e.g., not in a git repo).\n    Unavailable,\n}\n\nimpl std::fmt::Display for CapabilityState {\n    fn fmt(&self, f: &mut std::fmt::Formatter<\'_>) -> std::fmt::Result {\n        match self {\n            CapabilityState::Available => write!(f, "available"),\n            CapabilityState::Skipped => write!(f, "skipped"),\n            CapabilityState::Unavailable => write!(f, "unavailable"),\n        }\n    }\n}\n\n/// Entry describing an artifact',
    "CapabilityState Display")

# Strengthen scan_status_serde_roundtrip
c = replace_once(c,
    '    fn scan_status_serde_roundtrip() {\n        let json = serde_json::to_string(&ScanStatus::Complete).unwrap();\n        assert_eq!(json, "\\"complete\\"");\n        let back: ScanStatus = serde_json::from_str(&json).unwrap();\n        assert!(matches!(back, ScanStatus::Complete));\n    }',
    '    fn scan_status_serde_roundtrip() {\n        for variant in [ScanStatus::Complete, ScanStatus::Partial] {\n            let json = serde_json::to_string(&variant).unwrap();\n            let back: ScanStatus = serde_json::from_str(&json).unwrap();\n            assert_eq!(back, variant);\n        }\n    }',
    "strengthen scan_status_serde_roundtrip")

# Add new display tests - insert after capability_state_serde_roundtrip test
new_tests = """
    #[test]
    fn scan_status_display() {
        assert_eq!(ScanStatus::Complete.to_string(), "complete");
        assert_eq!(ScanStatus::Partial.to_string(), "partial");
    }

    #[test]
    fn file_classification_display() {
        assert_eq!(FileClassification::Generated.to_string(), "generated");
        assert_eq!(FileClassification::Fixture.to_string(), "fixture");
        assert_eq!(FileClassification::Vendored.to_string(), "vendored");
        assert_eq!(FileClassification::Lockfile.to_string(), "lockfile");
        assert_eq!(FileClassification::Minified.to_string(), "minified");
        assert_eq!(FileClassification::DataBlob.to_string(), "data_blob");
        assert_eq!(FileClassification::Sourcemap.to_string(), "sourcemap");
    }

    #[test]
    fn inclusion_policy_display() {
        assert_eq!(InclusionPolicy::Full.to_string(), "full");
        assert_eq!(InclusionPolicy::HeadTail.to_string(), "head_tail");
        assert_eq!(InclusionPolicy::Summary.to_string(), "summary");
        assert_eq!(InclusionPolicy::Skip.to_string(), "skip");
    }

    #[test]
    fn capability_state_display() {
        assert_eq!(CapabilityState::Available.to_string(), "available");
        assert_eq!(CapabilityState::Skipped.to_string(), "skipped");
        assert_eq!(CapabilityState::Unavailable.to_string(), "unavailable");
    }

"""

old_marker = '        }\n    }\n\n    #[test]\n    fn analysis_format_serde_roundtrip() {'
# This marker appears after the capability_state_serde_roundtrip test
c = replace_once(c, old_marker, '        }\n    }\n' + new_tests + '    #[test]\n    fn analysis_format_serde_roundtrip() {', "add display tests")

write(p, c)
print(f"[OK] {p}")


# ============================================================================
# 2. tokmd-types/src/cockpit.rs
# ============================================================================
p = r"crates\tokmd-types\src\cockpit.rs"
c = read(p)

# GateStatus Display
c = replace_once(c,
    '    Pending,\n}\n\n/// Source of evidence/gate results.',
    '    Pending,\n}\n\nimpl std::fmt::Display for GateStatus {\n    fn fmt(&self, f: &mut std::fmt::Formatter<\'_>) -> std::fmt::Result {\n        match self {\n            GateStatus::Pass => write!(f, "pass"),\n            GateStatus::Warn => write!(f, "warn"),\n            GateStatus::Fail => write!(f, "fail"),\n            GateStatus::Skipped => write!(f, "skipped"),\n            GateStatus::Pending => write!(f, "pending"),\n        }\n    }\n}\n\n/// Source of evidence/gate results.',
    "GateStatus Display")

# ComplexityIndicator Display - need unique context
c = replace_once(c,
    '    Critical,\n}\n\n/// Health warning for specific files.',
    '    Critical,\n}\n\nimpl std::fmt::Display for ComplexityIndicator {\n    fn fmt(&self, f: &mut std::fmt::Formatter<\'_>) -> std::fmt::Result {\n        match self {\n            ComplexityIndicator::Low => write!(f, "low"),\n            ComplexityIndicator::Medium => write!(f, "medium"),\n            ComplexityIndicator::High => write!(f, "high"),\n            ComplexityIndicator::Critical => write!(f, "critical"),\n        }\n    }\n}\n\n/// Health warning for specific files.',
    "ComplexityIndicator Display")

# TrendDirection Display
c = replace_once(c,
    '    Degrading,\n}\n\n#[cfg(test)]',
    '    Degrading,\n}\n\nimpl std::fmt::Display for TrendDirection {\n    fn fmt(&self, f: &mut std::fmt::Formatter<\'_>) -> std::fmt::Result {\n        match self {\n            TrendDirection::Improving => write!(f, "improving"),\n            TrendDirection::Stable => write!(f, "stable"),\n            TrendDirection::Degrading => write!(f, "degrading"),\n        }\n    }\n}\n\n#[cfg(test)]',
    "TrendDirection Display")

# Add cockpit tests
cockpit_tests = """
    #[test]
    fn gate_status_display() {
        assert_eq!(GateStatus::Pass.to_string(), "pass");
        assert_eq!(GateStatus::Warn.to_string(), "warn");
        assert_eq!(GateStatus::Fail.to_string(), "fail");
        assert_eq!(GateStatus::Skipped.to_string(), "skipped");
        assert_eq!(GateStatus::Pending.to_string(), "pending");
    }

    #[test]
    fn complexity_indicator_display() {
        assert_eq!(ComplexityIndicator::Low.to_string(), "low");
        assert_eq!(ComplexityIndicator::Medium.to_string(), "medium");
        assert_eq!(ComplexityIndicator::High.to_string(), "high");
        assert_eq!(ComplexityIndicator::Critical.to_string(), "critical");
    }

    #[test]
    fn trend_direction_display() {
        assert_eq!(TrendDirection::Improving.to_string(), "improving");
        assert_eq!(TrendDirection::Stable.to_string(), "stable");
        assert_eq!(TrendDirection::Degrading.to_string(), "degrading");
    }
"""

c = replace_once(c,
    '        assert_eq!(RiskLevel::Critical.to_string(), "critical");\n    }\n}',
    '        assert_eq!(RiskLevel::Critical.to_string(), "critical");\n    }\n' + cockpit_tests + '}',
    "cockpit tests")

write(p, c)
print(f"[OK] {p}")


# ============================================================================
# 3. tokmd-envelope/src/lib.rs
# ============================================================================
p = r"crates\tokmd-envelope\src\lib.rs"
c = read(p)

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct ToolMeta {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct ToolMeta {',
    "ToolMeta")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct FindingLocation {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct FindingLocation {',
    "FindingLocation")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Artifact {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct Artifact {',
    "Artifact")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct CapabilityStatus {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct CapabilityStatus {',
    "CapabilityStatus (envelope)")

# CapabilityState Display
c = replace_once(c,
    '    /// Capability was skipped (no relevant files, not applicable).\n    Skipped,\n}\n\nimpl CapabilityStatus {',
    '    /// Capability was skipped (no relevant files, not applicable).\n    Skipped,\n}\n\nimpl std::fmt::Display for CapabilityState {\n    fn fmt(&self, f: &mut std::fmt::Formatter<\'_>) -> std::fmt::Result {\n        match self {\n            CapabilityState::Available => write!(f, "available"),\n            CapabilityState::Unavailable => write!(f, "unavailable"),\n            CapabilityState::Skipped => write!(f, "skipped"),\n        }\n    }\n}\n\nimpl CapabilityStatus {',
    "envelope CapabilityState Display")

# Add envelope test
envelope_test = """
    #[test]
    fn envelope_capability_state_display() {
        assert_eq!(CapabilityState::Available.to_string(), "available");
        assert_eq!(CapabilityState::Unavailable.to_string(), "unavailable");
        assert_eq!(CapabilityState::Skipped.to_string(), "skipped");
    }
"""

c = replace_once(c,
    '        assert_eq!(badge.path, "out/badge.svg");\n    }\n}',
    '        assert_eq!(badge.path, "out/badge.svg");\n    }\n' + envelope_test + '}',
    "envelope tests")

write(p, c)
print(f"[OK] {p}")


# ============================================================================
# 4. tokmd-analysis-types/src/lib.rs
# ============================================================================
p = r"crates\tokmd-analysis-types\src\lib.rs"
c = read(p)

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct AnalysisSource {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct AnalysisSource {',
    "AnalysisSource")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct AnalysisArgsMeta {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct AnalysisArgsMeta {',
    "AnalysisArgsMeta")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct Archetype {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct Archetype {',
    "Archetype")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize, Default)]\npub struct CommitIntentCounts {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]\npub struct CommitIntentCounts {',
    "CommitIntentCounts")

# Add analysis tests
analysis_tests = """
    #[test]
    fn analysis_source_roundtrip() {
        let src = AnalysisSource {
            inputs: vec!["src".into()],
            export_path: Some("export.json".into()),
            base_receipt_path: None,
            export_schema_version: Some(2),
            export_generated_at_ms: Some(1000),
            base_signature: None,
            module_roots: vec![],
            module_depth: 1,
            children: "collapse".into(),
        };
        let json = serde_json::to_string(&src).unwrap();
        let back: AnalysisSource = serde_json::from_str(&json).unwrap();
        assert_eq!(src, back);
    }

    #[test]
    fn archetype_roundtrip() {
        let a = Archetype {
            kind: "web-app".into(),
            evidence: vec!["package.json".into(), "src/index.tsx".into()],
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: Archetype = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn commit_intent_counts_default() {
        let counts = CommitIntentCounts::default();
        assert_eq!(counts.total, 0);
        assert_eq!(counts.feat, 0);
        assert_eq!(counts.fix, 0);
        assert_eq!(counts, CommitIntentCounts::default());
    }
"""

c = replace_once(c,
    '        assert!(result.starts_with("2025-01-01"));\n    }\n}',
    '        assert!(result.starts_with("2025-01-01"));\n    }\n' + analysis_tests + '}',
    "analysis-types tests")

write(p, c)
print(f"[OK] {p}")


# ============================================================================
# 5. tokmd-substrate/src/lib.rs
# ============================================================================
p = r"crates\tokmd-substrate\src\lib.rs"
c = read(p)

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct RepoSubstrate {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct RepoSubstrate {',
    "RepoSubstrate")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct SubstrateFile {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct SubstrateFile {',
    "SubstrateFile")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct LangSummary {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct LangSummary {',
    "LangSummary")

c = replace_once(c,
    '#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct DiffRange {',
    '#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct DiffRange {',
    "DiffRange")

c = replace_once(c,
    '    fn serde_roundtrip() {\n        let sub = sample_substrate();\n        let json = serde_json::to_string(&sub).unwrap();\n        let back: RepoSubstrate = serde_json::from_str(&json).unwrap();\n        assert_eq!(back.files.len(), 2);\n        assert_eq!(back.total_code_lines, 150);\n        assert!(back.diff_range.is_some());\n    }',
    '    fn serde_roundtrip() {\n        let sub = sample_substrate();\n        let json = serde_json::to_string(&sub).unwrap();\n        let back: RepoSubstrate = serde_json::from_str(&json).unwrap();\n        assert_eq!(sub, back);\n    }',
    "strengthen substrate serde_roundtrip")

write(p, c)
print(f"[OK] {p}")

print("\nAll changes applied successfully!")
