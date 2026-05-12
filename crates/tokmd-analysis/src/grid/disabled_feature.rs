//! Disabled-feature warning catalog for analysis presets.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisabledFeature {
    FileInventory,
    TodoScan,
    DuplicationScan,
    NearDuplicateScan,
    ImportScan,
    GitMetrics,
    EntropyProfiling,
    LicenseRadar,
    ComplexityAnalysis,
    ApiSurfaceAnalysis,
    Archetype,
    Topics,
    Fun,
}

impl DisabledFeature {
    pub const fn warning(self) -> &'static str {
        match self {
            Self::FileInventory => "walk feature disabled; skipping file inventory",
            Self::TodoScan => "content feature disabled; skipping TODO scan",
            Self::DuplicationScan => "content feature disabled; skipping duplication scan",
            Self::NearDuplicateScan => "content feature disabled; skipping near-dup scan",
            Self::ImportScan => "content feature disabled; skipping import scan",
            Self::GitMetrics => "git feature disabled; skipping git metrics",
            Self::EntropyProfiling => "content/walk feature disabled; skipping entropy profiling",
            Self::LicenseRadar => "content/walk feature disabled; skipping license radar",
            Self::ComplexityAnalysis => {
                "content/walk feature disabled; skipping complexity analysis"
            }
            Self::ApiSurfaceAnalysis => {
                "content/walk feature disabled; skipping API surface analysis"
            }
            Self::Archetype => {
                "archetype feature is disabled for analysis; set `archetype` feature to include archetype inference"
            }
            Self::Topics => {
                "topics feature is disabled for analysis; set `topics` feature to include topic clouds"
            }
            Self::Fun => {
                "fun feature is disabled for analysis; set `fun` feature to include eco-label output"
            }
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn disabled_feature_messages_are_declared() {
        assert!(!DisabledFeature::FileInventory.warning().is_empty());
        assert!(!DisabledFeature::TodoScan.warning().is_empty());
        assert!(!DisabledFeature::DuplicationScan.warning().is_empty());
        assert!(!DisabledFeature::NearDuplicateScan.warning().is_empty());
        assert!(!DisabledFeature::ImportScan.warning().is_empty());
        assert!(!DisabledFeature::GitMetrics.warning().is_empty());
        assert!(!DisabledFeature::EntropyProfiling.warning().is_empty());
        assert!(!DisabledFeature::LicenseRadar.warning().is_empty());
        assert!(!DisabledFeature::ComplexityAnalysis.warning().is_empty());
        assert!(!DisabledFeature::ApiSurfaceAnalysis.warning().is_empty());
        assert!(!DisabledFeature::Archetype.warning().is_empty());
        assert!(!DisabledFeature::Topics.warning().is_empty());
        assert!(!DisabledFeature::Fun.warning().is_empty());
    }
}
