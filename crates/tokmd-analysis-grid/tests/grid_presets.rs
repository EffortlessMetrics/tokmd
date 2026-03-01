//! Deeper tests for preset grid definitions, feature flag membership,
//! and exhaustive coverage of grid rows including feature-gated fields.

use tokmd_analysis_grid::{
    DisabledFeature, PRESET_GRID, PresetKind, preset_plan_for, preset_plan_for_name,
};

// ── Feature-gated: halstead ─────────────────────────────────────────

#[cfg(all(feature = "halstead", feature = "content", feature = "walk"))]
mod halstead_features {
    use super::*;

    #[test]
    fn health_preset_enables_halstead() {
        assert!(preset_plan_for(PresetKind::Health).halstead);
    }

    #[test]
    fn risk_preset_enables_halstead() {
        assert!(preset_plan_for(PresetKind::Risk).halstead);
    }

    #[test]
    fn deep_preset_enables_halstead() {
        assert!(preset_plan_for(PresetKind::Deep).halstead);
    }

    #[test]
    fn receipt_fun_supply_do_not_enable_halstead() {
        assert!(!preset_plan_for(PresetKind::Receipt).halstead);
        assert!(!preset_plan_for(PresetKind::Fun).halstead);
        assert!(!preset_plan_for(PresetKind::Supply).halstead);
        assert!(!preset_plan_for(PresetKind::Architecture).halstead);
        assert!(!preset_plan_for(PresetKind::Topics).halstead);
        assert!(!preset_plan_for(PresetKind::Security).halstead);
        assert!(!preset_plan_for(PresetKind::Identity).halstead);
        assert!(!preset_plan_for(PresetKind::Git).halstead);
    }

    #[test]
    fn exactly_three_presets_enable_halstead() {
        let count = PRESET_GRID.iter().filter(|r| r.plan.halstead).count();
        assert_eq!(count, 3, "only Health, Risk, Deep should enable halstead");
    }

    #[test]
    fn halstead_true_alone_triggers_needs_files() {
        // Verify via Deep which has halstead=true alongside other file flags
        let plan = preset_plan_for(PresetKind::Deep);
        assert!(plan.halstead);
        assert!(plan.needs_files());
    }
}

// ── Feature-gated: git (churn, fingerprint) ─────────────────────────

#[cfg(feature = "git")]
mod git_features {
    use super::*;

    #[test]
    fn git_preset_enables_churn_but_not_fingerprint() {
        let plan = preset_plan_for(PresetKind::Git);
        assert!(plan.churn);
        assert!(!plan.fingerprint);
    }

    #[test]
    fn deep_preset_enables_both_churn_and_fingerprint() {
        let plan = preset_plan_for(PresetKind::Deep);
        assert!(plan.churn);
        assert!(plan.fingerprint);
    }

    #[test]
    fn identity_preset_enables_fingerprint_but_not_churn() {
        let plan = preset_plan_for(PresetKind::Identity);
        assert!(plan.fingerprint);
        assert!(!plan.churn);
    }

    #[test]
    fn receipt_does_not_enable_churn_or_fingerprint() {
        let plan = preset_plan_for(PresetKind::Receipt);
        assert!(!plan.churn);
        assert!(!plan.fingerprint);
    }

    #[test]
    fn exactly_two_presets_enable_churn() {
        let churn: Vec<_> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.churn)
            .map(|r| r.preset)
            .collect();
        assert_eq!(churn.len(), 2);
        assert!(churn.contains(&PresetKind::Git));
        assert!(churn.contains(&PresetKind::Deep));
    }

    #[test]
    fn exactly_two_presets_enable_fingerprint() {
        let fp: Vec<_> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.fingerprint)
            .map(|r| r.preset)
            .collect();
        assert_eq!(fp.len(), 2);
        assert!(fp.contains(&PresetKind::Identity));
        assert!(fp.contains(&PresetKind::Deep));
    }
}

// ── Cross-cutting flag membership queries ───────────────────────────

#[test]
fn presets_enabling_complexity_are_health_risk_deep() {
    let presets: Vec<_> = PRESET_GRID
        .iter()
        .filter(|r| r.plan.complexity)
        .map(|r| r.preset)
        .collect();
    assert_eq!(presets.len(), 3);
    assert!(presets.contains(&PresetKind::Health));
    assert!(presets.contains(&PresetKind::Risk));
    assert!(presets.contains(&PresetKind::Deep));
}

#[test]
fn presets_enabling_imports_are_architecture_deep() {
    let presets: Vec<_> = PRESET_GRID
        .iter()
        .filter(|r| r.plan.imports)
        .map(|r| r.preset)
        .collect();
    assert_eq!(presets.len(), 2);
    assert!(presets.contains(&PresetKind::Architecture));
    assert!(presets.contains(&PresetKind::Deep));
}

#[test]
fn presets_enabling_entropy_are_security_deep() {
    let presets: Vec<_> = PRESET_GRID
        .iter()
        .filter(|r| r.plan.entropy)
        .map(|r| r.preset)
        .collect();
    assert_eq!(presets.len(), 2);
    assert!(presets.contains(&PresetKind::Security));
    assert!(presets.contains(&PresetKind::Deep));
}

#[test]
fn presets_enabling_license_are_security_deep() {
    let presets: Vec<_> = PRESET_GRID
        .iter()
        .filter(|r| r.plan.license)
        .map(|r| r.preset)
        .collect();
    assert_eq!(presets.len(), 2);
    assert!(presets.contains(&PresetKind::Security));
    assert!(presets.contains(&PresetKind::Deep));
}

#[test]
fn presets_enabling_archetype_are_identity_deep() {
    let presets: Vec<_> = PRESET_GRID
        .iter()
        .filter(|r| r.plan.archetype)
        .map(|r| r.preset)
        .collect();
    assert_eq!(presets.len(), 2);
    assert!(presets.contains(&PresetKind::Identity));
    assert!(presets.contains(&PresetKind::Deep));
}

#[test]
fn presets_enabling_topics_are_topics_deep() {
    let presets: Vec<_> = PRESET_GRID
        .iter()
        .filter(|r| r.plan.topics)
        .map(|r| r.preset)
        .collect();
    assert_eq!(presets.len(), 2);
    assert!(presets.contains(&PresetKind::Topics));
    assert!(presets.contains(&PresetKind::Deep));
}

#[test]
fn only_fun_preset_enables_fun_flag() {
    let presets: Vec<_> = PRESET_GRID
        .iter()
        .filter(|r| r.plan.fun)
        .map(|r| r.preset)
        .collect();
    assert_eq!(presets, vec![PresetKind::Fun]);
}

#[test]
fn only_deep_preset_enables_dup() {
    let presets: Vec<_> = PRESET_GRID
        .iter()
        .filter(|r| r.plan.dup)
        .map(|r| r.preset)
        .collect();
    assert_eq!(presets, vec![PresetKind::Deep]);
}

#[test]
fn presets_enabling_assets_are_supply_deep() {
    let presets: Vec<_> = PRESET_GRID
        .iter()
        .filter(|r| r.plan.assets)
        .map(|r| r.preset)
        .collect();
    assert_eq!(presets.len(), 2);
    assert!(presets.contains(&PresetKind::Supply));
    assert!(presets.contains(&PresetKind::Deep));
}

#[test]
fn presets_enabling_deps_are_supply_deep() {
    let presets: Vec<_> = PRESET_GRID
        .iter()
        .filter(|r| r.plan.deps)
        .map(|r| r.preset)
        .collect();
    assert_eq!(presets.len(), 2);
    assert!(presets.contains(&PresetKind::Supply));
    assert!(presets.contains(&PresetKind::Deep));
}

#[test]
fn presets_enabling_todo_are_health_deep() {
    let presets: Vec<_> = PRESET_GRID
        .iter()
        .filter(|r| r.plan.todo)
        .map(|r| r.preset)
        .collect();
    assert_eq!(presets.len(), 2);
    assert!(presets.contains(&PresetKind::Health));
    assert!(presets.contains(&PresetKind::Deep));
}

#[test]
fn presets_enabling_api_surface_are_architecture_deep() {
    let presets: Vec<_> = PRESET_GRID
        .iter()
        .filter(|r| r.plan.api_surface)
        .map(|r| r.preset)
        .collect();
    assert_eq!(presets.len(), 2);
    assert!(presets.contains(&PresetKind::Architecture));
    assert!(presets.contains(&PresetKind::Deep));
}

// ── Deep-is-superset invariant ──────────────────────────────────────

#[test]
fn deep_is_superset_of_every_non_fun_base_flag() {
    let deep = preset_plan_for(PresetKind::Deep);
    for row in &PRESET_GRID {
        if row.preset == PresetKind::Fun {
            continue;
        }
        let p = row.plan;
        if p.assets {
            assert!(deep.assets, "{:?} assets not covered by Deep", row.preset);
        }
        if p.deps {
            assert!(deep.deps, "{:?} deps not covered by Deep", row.preset);
        }
        if p.todo {
            assert!(deep.todo, "{:?} todo not covered by Deep", row.preset);
        }
        if p.dup {
            assert!(deep.dup, "{:?} dup not covered by Deep", row.preset);
        }
        if p.imports {
            assert!(deep.imports, "{:?} imports not covered", row.preset);
        }
        if p.git {
            assert!(deep.git, "{:?} git not covered by Deep", row.preset);
        }
        if p.archetype {
            assert!(deep.archetype, "{:?} archetype not covered", row.preset);
        }
        if p.topics {
            assert!(deep.topics, "{:?} topics not covered", row.preset);
        }
        if p.entropy {
            assert!(deep.entropy, "{:?} entropy not covered", row.preset);
        }
        if p.license {
            assert!(deep.license, "{:?} license not covered", row.preset);
        }
        if p.complexity {
            assert!(deep.complexity, "{:?} complexity not covered", row.preset);
        }
        if p.api_surface {
            assert!(deep.api_surface, "{:?} api_surface not covered", row.preset);
        }
    }
}

#[cfg(feature = "git")]
#[test]
fn deep_is_superset_of_git_gated_flags() {
    let deep = preset_plan_for(PresetKind::Deep);
    for row in &PRESET_GRID {
        if row.preset == PresetKind::Fun {
            continue;
        }
        if row.plan.churn {
            assert!(deep.churn, "{:?} churn not covered by Deep", row.preset);
        }
        if row.plan.fingerprint {
            assert!(
                deep.fingerprint,
                "{:?} fingerprint not covered by Deep",
                row.preset
            );
        }
    }
}

// ── needs_files boundary tests (gaps in existing coverage) ──────────

#[test]
fn risk_preset_needs_files_because_complexity() {
    // Risk has complexity=true which triggers needs_files
    assert!(preset_plan_for(PresetKind::Risk).needs_files());
}

#[test]
fn git_preset_does_not_need_files() {
    // git flag alone doesn't trigger needs_files
    assert!(!preset_plan_for(PresetKind::Git).needs_files());
}

#[test]
fn topics_preset_does_not_need_files() {
    assert!(!preset_plan_for(PresetKind::Topics).needs_files());
}

#[test]
fn identity_preset_does_not_need_files() {
    // git + archetype don't trigger needs_files
    assert!(!preset_plan_for(PresetKind::Identity).needs_files());
}

#[test]
fn every_preset_needs_files_classification() {
    let needs = |k| preset_plan_for(k).needs_files();
    assert!(!needs(PresetKind::Receipt));
    assert!(needs(PresetKind::Health));
    assert!(needs(PresetKind::Risk));
    assert!(needs(PresetKind::Supply));
    assert!(needs(PresetKind::Architecture));
    assert!(!needs(PresetKind::Topics));
    assert!(needs(PresetKind::Security));
    assert!(!needs(PresetKind::Identity));
    assert!(!needs(PresetKind::Git));
    assert!(needs(PresetKind::Deep));
    assert!(!needs(PresetKind::Fun));
}

// ── Disabled feature warning keyword hints ──────────────────────────

#[test]
fn disabled_feature_warnings_contain_relevant_keywords() {
    assert!(DisabledFeature::FileInventory.warning().contains("walk"));
    assert!(DisabledFeature::TodoScan.warning().contains("content"));
    assert!(
        DisabledFeature::DuplicationScan
            .warning()
            .contains("content")
    );
    assert!(
        DisabledFeature::NearDuplicateScan
            .warning()
            .contains("near-dup")
    );
    assert!(DisabledFeature::ImportScan.warning().contains("import"));
    assert!(DisabledFeature::GitMetrics.warning().contains("git"));
    assert!(
        DisabledFeature::EntropyProfiling
            .warning()
            .contains("entropy")
    );
    assert!(DisabledFeature::LicenseRadar.warning().contains("license"));
    assert!(
        DisabledFeature::ComplexityAnalysis
            .warning()
            .contains("complexity")
    );
    assert!(
        DisabledFeature::ApiSurfaceAnalysis
            .warning()
            .contains("API")
    );
    assert!(DisabledFeature::Archetype.warning().contains("archetype"));
    assert!(DisabledFeature::Topics.warning().contains("topics"));
    assert!(DisabledFeature::Fun.warning().contains("eco-label"));
}

#[test]
fn disabled_feature_count_is_thirteen() {
    let all = [
        DisabledFeature::FileInventory,
        DisabledFeature::TodoScan,
        DisabledFeature::DuplicationScan,
        DisabledFeature::NearDuplicateScan,
        DisabledFeature::ImportScan,
        DisabledFeature::GitMetrics,
        DisabledFeature::EntropyProfiling,
        DisabledFeature::LicenseRadar,
        DisabledFeature::ComplexityAnalysis,
        DisabledFeature::ApiSurfaceAnalysis,
        DisabledFeature::Archetype,
        DisabledFeature::Topics,
        DisabledFeature::Fun,
    ];
    assert_eq!(all.len(), 13);
}

// ── Preset name invariants ──────────────────────────────────────────

#[test]
fn preset_names_are_valid_ascii_identifiers() {
    for kind in PresetKind::all() {
        let name = kind.as_str();
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()),
            "preset name {name:?} has invalid chars"
        );
        assert!(
            !name.is_empty(),
            "preset name must not be empty for {:?}",
            kind
        );
    }
}

#[test]
fn from_str_rejects_mixed_case_variants() {
    let mixed = [
        "Receipt",
        "HEALTH",
        "rIsK",
        "Supply",
        "ARCHITECTURE",
        "Topics",
        "SECURITY",
        "Identity",
        "GIT",
        "Deep",
        "FUN",
    ];
    for name in &mixed {
        assert!(
            PresetKind::from_str(name).is_none(),
            "mixed case {name:?} should be rejected"
        );
    }
}

// ── plan_for_name round-trip through every name ─────────────────────

#[test]
fn plan_for_name_matches_plan_for_kind_for_all_presets() {
    for kind in PresetKind::all() {
        let via_name = preset_plan_for_name(kind.as_str()).unwrap();
        let via_kind = preset_plan_for(*kind);
        assert_eq!(via_name, via_kind, "mismatch for {:?}", kind);
    }
}
