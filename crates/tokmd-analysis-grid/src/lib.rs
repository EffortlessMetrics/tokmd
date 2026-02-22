#![doc = "Feature matrix and warning catalog for tokmd-analysis preset execution."]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetKind {
    Receipt,
    Health,
    Risk,
    Supply,
    Architecture,
    Topics,
    Security,
    Identity,
    Git,
    Deep,
    Fun,
}

impl PresetKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Receipt => "receipt",
            Self::Health => "health",
            Self::Risk => "risk",
            Self::Supply => "supply",
            Self::Architecture => "architecture",
            Self::Topics => "topics",
            Self::Security => "security",
            Self::Identity => "identity",
            Self::Git => "git",
            Self::Deep => "deep",
            Self::Fun => "fun",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "receipt" => Some(Self::Receipt),
            "health" => Some(Self::Health),
            "risk" => Some(Self::Risk),
            "supply" => Some(Self::Supply),
            "architecture" => Some(Self::Architecture),
            "topics" => Some(Self::Topics),
            "security" => Some(Self::Security),
            "identity" => Some(Self::Identity),
            "git" => Some(Self::Git),
            "deep" => Some(Self::Deep),
            "fun" => Some(Self::Fun),
            _ => None,
        }
    }
}

pub const PRESET_KINDS: [PresetKind; 11] = [
    PresetKind::Receipt,
    PresetKind::Health,
    PresetKind::Risk,
    PresetKind::Supply,
    PresetKind::Architecture,
    PresetKind::Topics,
    PresetKind::Security,
    PresetKind::Identity,
    PresetKind::Git,
    PresetKind::Deep,
    PresetKind::Fun,
];

impl PresetKind {
    pub const fn all() -> &'static [PresetKind; 11] {
        &PRESET_KINDS
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresetPlan {
    pub assets: bool,
    pub deps: bool,
    pub todo: bool,
    pub dup: bool,
    pub imports: bool,
    pub git: bool,
    pub fun: bool,
    pub archetype: bool,
    pub topics: bool,
    pub entropy: bool,
    pub license: bool,
    pub complexity: bool,
    pub api_surface: bool,
    #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
    pub halstead: bool,
    #[cfg(feature = "git")]
    pub churn: bool,
    #[cfg(feature = "git")]
    pub fingerprint: bool,
}

impl PresetPlan {
    #[cfg_attr(
        not(all(feature = "halstead", feature = "content", feature = "walk")),
        allow(unused_mut)
    )]
    pub fn needs_files(&self) -> bool {
        let mut needs = self.assets
            || self.deps
            || self.todo
            || self.dup
            || self.imports
            || self.entropy
            || self.license
            || self.complexity
            || self.api_surface;
        #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
        {
            needs = needs || self.halstead;
        }
        needs
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PresetGridRow {
    pub preset: PresetKind,
    pub plan: PresetPlan,
}

pub const PRESET_GRID: [PresetGridRow; 11] = [
    PresetGridRow {
        preset: PresetKind::Receipt,
        plan: PresetPlan {
            assets: false,
            deps: false,
            todo: false,
            dup: false,
            imports: false,
            git: false,
            fun: false,
            archetype: false,
            topics: false,
            entropy: false,
            license: false,
            complexity: false,
            api_surface: false,
            #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
            halstead: false,
            #[cfg(feature = "git")]
            churn: false,
            #[cfg(feature = "git")]
            fingerprint: false,
        },
    },
    PresetGridRow {
        preset: PresetKind::Health,
        plan: PresetPlan {
            assets: false,
            deps: false,
            todo: true,
            dup: false,
            imports: false,
            git: false,
            fun: false,
            archetype: false,
            topics: false,
            entropy: false,
            license: false,
            complexity: true,
            api_surface: false,
            #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
            halstead: true,
            #[cfg(feature = "git")]
            churn: false,
            #[cfg(feature = "git")]
            fingerprint: false,
        },
    },
    PresetGridRow {
        preset: PresetKind::Risk,
        plan: PresetPlan {
            assets: false,
            deps: false,
            todo: false,
            dup: false,
            imports: false,
            git: true,
            fun: false,
            archetype: false,
            topics: false,
            entropy: false,
            license: false,
            complexity: true,
            api_surface: false,
            #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
            halstead: true,
            #[cfg(feature = "git")]
            churn: false,
            #[cfg(feature = "git")]
            fingerprint: false,
        },
    },
    PresetGridRow {
        preset: PresetKind::Supply,
        plan: PresetPlan {
            assets: true,
            deps: true,
            todo: false,
            dup: false,
            imports: false,
            git: false,
            fun: false,
            archetype: false,
            topics: false,
            entropy: false,
            license: false,
            complexity: false,
            api_surface: false,
            #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
            halstead: false,
            #[cfg(feature = "git")]
            churn: false,
            #[cfg(feature = "git")]
            fingerprint: false,
        },
    },
    PresetGridRow {
        preset: PresetKind::Architecture,
        plan: PresetPlan {
            assets: false,
            deps: false,
            todo: false,
            dup: false,
            imports: true,
            git: false,
            fun: false,
            archetype: false,
            topics: false,
            entropy: false,
            license: false,
            complexity: false,
            api_surface: true,
            #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
            halstead: false,
            #[cfg(feature = "git")]
            churn: false,
            #[cfg(feature = "git")]
            fingerprint: false,
        },
    },
    PresetGridRow {
        preset: PresetKind::Topics,
        plan: PresetPlan {
            assets: false,
            deps: false,
            todo: false,
            dup: false,
            imports: false,
            git: false,
            fun: false,
            archetype: false,
            topics: true,
            entropy: false,
            license: false,
            complexity: false,
            api_surface: false,
            #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
            halstead: false,
            #[cfg(feature = "git")]
            churn: false,
            #[cfg(feature = "git")]
            fingerprint: false,
        },
    },
    PresetGridRow {
        preset: PresetKind::Security,
        plan: PresetPlan {
            assets: false,
            deps: false,
            todo: false,
            dup: false,
            imports: false,
            git: false,
            fun: false,
            archetype: false,
            topics: false,
            entropy: true,
            license: true,
            complexity: false,
            api_surface: false,
            #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
            halstead: false,
            #[cfg(feature = "git")]
            churn: false,
            #[cfg(feature = "git")]
            fingerprint: false,
        },
    },
    PresetGridRow {
        preset: PresetKind::Identity,
        plan: PresetPlan {
            assets: false,
            deps: false,
            todo: false,
            dup: false,
            imports: false,
            git: true,
            fun: false,
            archetype: true,
            topics: false,
            entropy: false,
            license: false,
            complexity: false,
            api_surface: false,
            #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
            halstead: false,
            #[cfg(feature = "git")]
            churn: false,
            #[cfg(feature = "git")]
            fingerprint: true,
        },
    },
    PresetGridRow {
        preset: PresetKind::Git,
        plan: PresetPlan {
            assets: false,
            deps: false,
            todo: false,
            dup: false,
            imports: false,
            git: true,
            fun: false,
            archetype: false,
            topics: false,
            entropy: false,
            license: false,
            complexity: false,
            api_surface: false,
            #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
            halstead: false,
            #[cfg(feature = "git")]
            churn: true,
            #[cfg(feature = "git")]
            fingerprint: false,
        },
    },
    PresetGridRow {
        preset: PresetKind::Deep,
        plan: PresetPlan {
            assets: true,
            deps: true,
            todo: true,
            dup: true,
            imports: true,
            git: true,
            fun: false,
            archetype: true,
            topics: true,
            entropy: true,
            license: true,
            complexity: true,
            api_surface: true,
            #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
            halstead: true,
            #[cfg(feature = "git")]
            churn: true,
            #[cfg(feature = "git")]
            fingerprint: true,
        },
    },
    PresetGridRow {
        preset: PresetKind::Fun,
        plan: PresetPlan {
            assets: false,
            deps: false,
            todo: false,
            dup: false,
            imports: false,
            git: false,
            fun: true,
            archetype: false,
            topics: false,
            entropy: false,
            license: false,
            complexity: false,
            api_surface: false,
            #[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
            halstead: false,
            #[cfg(feature = "git")]
            churn: false,
            #[cfg(feature = "git")]
            fingerprint: false,
        },
    },
];

pub fn preset_plan_for(preset: PresetKind) -> PresetPlan {
    let mut i = 0;
    while i < PRESET_GRID.len() {
        if PRESET_GRID[i].preset == preset {
            return PRESET_GRID[i].plan;
        }
        i += 1;
    }
    unreachable!();
}

pub fn preset_plan_for_name(name: &str) -> Option<PresetPlan> {
    PresetKind::from_str(name).map(preset_plan_for)
}

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
mod tests {
    use super::*;

    #[test]
    fn preset_table_covers_all_presets() {
        for preset in PresetKind::all() {
            assert!(PRESET_GRID.iter().any(|row| row.preset == *preset));
        }
        assert_eq!(PRESET_GRID.len(), PresetKind::all().len());
    }

    #[test]
    fn preset_name_roundtrip_is_stable() {
        for preset in PresetKind::all() {
            let parsed = PresetKind::from_str(preset.as_str()).expect("preset should parse");
            assert_eq!(parsed, *preset);
            assert_eq!(
                preset_plan_for_name(preset.as_str()),
                Some(preset_plan_for(*preset))
            );
        }
    }

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
