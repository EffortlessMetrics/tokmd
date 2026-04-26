//! W73 deep tests for `analysis grid module` preset/feature matrix.
//!
//! Covers:
//! - All presets represented in PRESET_GRID
//! - PresetKind roundtrip stability
//! - PresetPlan needs_files correctness
//! - DisabledFeature warning catalog completeness
//! - Grid formatting and field correctness
//! - Preset name validation
//! - Deep preset superset property

use crate::grid::{
    DisabledFeature, PRESET_GRID, PRESET_KINDS, PresetKind, preset_plan_for, preset_plan_for_name,
};

// ═══════════════════════════════════════════════════════════════════════════
// 1. All presets represented in grid
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn grid_has_exactly_twelve_entries() {
    assert_eq!(PRESET_GRID.len(), 12);
}

#[test]
fn grid_covers_every_preset_kind() {
    for kind in PresetKind::all() {
        let found = PRESET_GRID.iter().any(|row| row.preset == *kind);
        assert!(found, "{:?} not found in PRESET_GRID", kind);
    }
}

#[test]
fn preset_kinds_array_has_twelve_entries() {
    assert_eq!(PRESET_KINDS.len(), 12);
}

#[test]
fn preset_kinds_no_duplicates() {
    for (i, a) in PRESET_KINDS.iter().enumerate() {
        for (j, b) in PRESET_KINDS.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "Duplicate preset at indices {} and {}", i, j);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. PresetKind roundtrip and naming
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn from_str_roundtrip_all_presets() {
    for kind in PresetKind::all() {
        let name = kind.as_str();
        let parsed = PresetKind::from_str(name);
        assert_eq!(parsed, Some(*kind), "Roundtrip failed for {:?}", kind);
    }
}

#[test]
fn from_str_rejects_unknown_names() {
    assert!(PresetKind::from_str("unknown").is_none());
    assert!(PresetKind::from_str("").is_none());
    assert!(PresetKind::from_str("RECEIPT").is_none());
    assert!(PresetKind::from_str("Deep").is_none());
    assert!(PresetKind::from_str("all").is_none());
    assert!(PresetKind::from_str("none").is_none());
}

#[test]
fn as_str_all_lowercase() {
    for kind in PresetKind::all() {
        let name = kind.as_str();
        assert!(
            name.chars().all(|c| c.is_ascii_lowercase()),
            "as_str for {:?} should be all lowercase: {}",
            kind,
            name
        );
    }
}

#[test]
fn as_str_non_empty() {
    for kind in PresetKind::all() {
        assert!(!kind.as_str().is_empty(), "{:?} has empty name", kind);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. preset_plan_for and preset_plan_for_name
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn preset_plan_for_name_matches_direct_lookup() {
    for kind in PresetKind::all() {
        let by_name = preset_plan_for_name(kind.as_str()).unwrap();
        let direct = preset_plan_for(*kind);
        assert_eq!(by_name, direct, "Plans differ for {:?}", kind);
    }
}

#[test]
fn preset_plan_for_name_returns_none_for_unknown() {
    assert!(preset_plan_for_name("nonexistent").is_none());
    assert!(preset_plan_for_name("").is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. PresetPlan needs_files correctness
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_plan_needs_files() {
    let plan = preset_plan_for(PresetKind::Receipt);
    assert!(
        plan.needs_files(),
        "Receipt should need files (dup, complexity, api_surface)"
    );
}

#[test]
fn supply_plan_needs_files() {
    let plan = preset_plan_for(PresetKind::Supply);
    assert!(
        plan.needs_files(),
        "Supply should need files (assets + deps)"
    );
}

#[test]
fn health_plan_needs_files() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(
        plan.needs_files(),
        "Health should need files (todo + complexity)"
    );
}

#[test]
fn deep_plan_needs_files() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.needs_files(), "Deep should need files");
}

#[test]
fn fun_plan_does_not_need_files() {
    let plan = preset_plan_for(PresetKind::Fun);
    assert!(!plan.needs_files(), "Fun should not need files");
}

#[test]
fn git_plan_does_not_need_files() {
    let plan = preset_plan_for(PresetKind::Git);
    assert!(!plan.needs_files(), "Git preset should not need files");
}

#[test]
fn topics_plan_does_not_need_files() {
    let plan = preset_plan_for(PresetKind::Topics);
    assert!(!plan.needs_files(), "Topics should not need files");
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. DisabledFeature warning catalog
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn all_disabled_features_have_nonempty_warnings() {
    let features = [
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
    for f in &features {
        let msg = f.warning();
        assert!(!msg.is_empty(), "{:?} has empty warning", f);
        assert!(
            msg.contains("disabled") || msg.contains("skipping"),
            "{:?} warning should mention 'disabled' or 'skipping': {}",
            f,
            msg
        );
    }
}

#[test]
fn disabled_feature_warnings_are_distinct() {
    let features = [
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
    let msgs: Vec<&str> = features.iter().map(|f| f.warning()).collect();
    for (i, a) in msgs.iter().enumerate() {
        for (j, b) in msgs.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "Duplicate warning at indices {} and {}", i, j);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Deep preset superset property
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn deep_plan_is_superset_of_every_non_fun_preset_base_fields() {
    let deep = preset_plan_for(PresetKind::Deep);
    for kind in PresetKind::all() {
        if *kind == PresetKind::Deep || *kind == PresetKind::Fun {
            continue;
        }
        let plan = preset_plan_for(*kind);
        // For each base field, if any preset enables it, Deep must also enable it
        if plan.assets {
            assert!(deep.assets, "Deep missing assets from {:?}", kind);
        }
        if plan.deps {
            assert!(deep.deps, "Deep missing deps from {:?}", kind);
        }
        if plan.todo {
            assert!(deep.todo, "Deep missing todo from {:?}", kind);
        }
        if plan.dup {
            assert!(deep.dup, "Deep missing dup from {:?}", kind);
        }
        if plan.imports {
            assert!(deep.imports, "Deep missing imports from {:?}", kind);
        }
        if plan.git {
            assert!(deep.git, "Deep missing git from {:?}", kind);
        }
        if plan.archetype {
            assert!(deep.archetype, "Deep missing archetype from {:?}", kind);
        }
        if plan.topics {
            assert!(deep.topics, "Deep missing topics from {:?}", kind);
        }
        if plan.entropy {
            assert!(deep.entropy, "Deep missing entropy from {:?}", kind);
        }
        if plan.license {
            assert!(deep.license, "Deep missing license from {:?}", kind);
        }
        if plan.complexity {
            assert!(deep.complexity, "Deep missing complexity from {:?}", kind);
        }
        if plan.api_surface {
            assert!(deep.api_surface, "Deep missing api_surface from {:?}", kind);
        }
    }
}

#[test]
fn deep_plan_does_not_enable_fun() {
    let deep = preset_plan_for(PresetKind::Deep);
    assert!(!deep.fun, "Deep should not enable fun");
}
