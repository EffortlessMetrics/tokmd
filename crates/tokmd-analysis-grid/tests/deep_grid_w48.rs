//! Deep analysis-grid tests (wave 48).
//!
//! Covers:
//! - Preset/feature matrix metadata
//! - Grid cell evaluation
//! - Feature availability reporting
//! - DisabledFeature warning messages

use tokmd_analysis_grid::{
    DisabledFeature, PRESET_GRID, PRESET_KINDS, PresetKind, preset_plan_for, preset_plan_for_name,
};

// ═══════════════════════════════════════════════════════════════════════════
// 1. Preset/feature matrix metadata
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn preset_grid_has_11_entries() {
    assert_eq!(PRESET_GRID.len(), 11);
}

#[test]
fn preset_kinds_has_11_entries() {
    assert_eq!(PRESET_KINDS.len(), 11);
}

#[test]
fn preset_kind_all_returns_same_as_const() {
    let all = PresetKind::all();
    assert_eq!(all.len(), PRESET_KINDS.len());
    for (a, b) in all.iter().zip(PRESET_KINDS.iter()) {
        assert_eq!(a, b);
    }
}

#[test]
fn every_preset_kind_in_grid() {
    for kind in PresetKind::all() {
        let found = PRESET_GRID.iter().any(|row| row.preset == *kind);
        assert!(found, "{:?} not found in PRESET_GRID", kind);
    }
}

#[test]
fn no_duplicate_presets_in_grid() {
    let mut seen = Vec::new();
    for row in &PRESET_GRID {
        assert!(
            !seen.contains(&row.preset),
            "Duplicate preset {:?} in PRESET_GRID",
            row.preset
        );
        seen.push(row.preset);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Grid cell evaluation — preset_plan_for and preset_plan_for_name
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn preset_plan_for_name_returns_none_for_unknown() {
    assert!(preset_plan_for_name("nonexistent").is_none());
    assert!(preset_plan_for_name("").is_none());
    assert!(preset_plan_for_name("RECEIPT").is_none());
}

#[test]
fn preset_plan_for_name_matches_kind_lookup() {
    for kind in PresetKind::all() {
        let by_name = preset_plan_for_name(kind.as_str()).unwrap();
        let by_kind = preset_plan_for(*kind);
        assert_eq!(by_name, by_kind, "Mismatch for {:?}", kind);
    }
}

#[test]
fn preset_kind_from_str_roundtrip() {
    for kind in PresetKind::all() {
        let s = kind.as_str();
        let parsed = PresetKind::from_str(s).unwrap();
        assert_eq!(parsed, *kind);
    }
}

#[test]
fn receipt_plan_enables_nothing() {
    let plan = preset_plan_for(PresetKind::Receipt);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(!plan.dup);
    assert!(!plan.imports);
    assert!(!plan.git);
    assert!(!plan.fun);
    assert!(!plan.archetype);
    assert!(!plan.topics);
    assert!(!plan.entropy);
    assert!(!plan.license);
    assert!(!plan.complexity);
    assert!(!plan.api_surface);
}

#[test]
fn deep_plan_enables_all_base_flags_except_fun() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.assets);
    assert!(plan.deps);
    assert!(plan.todo);
    assert!(plan.dup);
    assert!(plan.imports);
    assert!(plan.git);
    assert!(!plan.fun);
    assert!(plan.archetype);
    assert!(plan.topics);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(plan.complexity);
    assert!(plan.api_surface);
}

#[test]
fn fun_plan_enables_only_fun() {
    let plan = preset_plan_for(PresetKind::Fun);
    assert!(plan.fun);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(!plan.git);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. PresetPlan::needs_files
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn receipt_does_not_need_files() {
    assert!(!preset_plan_for(PresetKind::Receipt).needs_files());
}

#[test]
fn supply_needs_files() {
    assert!(preset_plan_for(PresetKind::Supply).needs_files());
}

#[test]
fn deep_needs_files() {
    assert!(preset_plan_for(PresetKind::Deep).needs_files());
}

#[test]
fn fun_does_not_need_files() {
    assert!(!preset_plan_for(PresetKind::Fun).needs_files());
}

#[test]
fn topics_does_not_need_files() {
    assert!(!preset_plan_for(PresetKind::Topics).needs_files());
}

#[test]
fn git_does_not_need_files() {
    assert!(!preset_plan_for(PresetKind::Git).needs_files());
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Feature availability reporting — DisabledFeature warnings
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
    for feature in &features {
        assert!(
            !feature.warning().is_empty(),
            "{:?} has empty warning",
            feature
        );
    }
}

#[test]
fn disabled_feature_warnings_contain_feature_keyword() {
    // Each warning should mention the feature or its gate
    assert!(DisabledFeature::GitMetrics.warning().contains("git"));
    assert!(DisabledFeature::TodoScan.warning().contains("content"));
    assert!(
        DisabledFeature::EntropyProfiling
            .warning()
            .contains("content")
    );
    assert!(DisabledFeature::LicenseRadar.warning().contains("content"));
    assert!(DisabledFeature::Fun.warning().contains("fun"));
}

#[test]
fn disabled_feature_warnings_contain_skipping_or_disabled() {
    let features = [
        DisabledFeature::FileInventory,
        DisabledFeature::TodoScan,
        DisabledFeature::DuplicationScan,
        DisabledFeature::ImportScan,
        DisabledFeature::GitMetrics,
    ];
    for feature in &features {
        let w = feature.warning();
        assert!(
            w.contains("skipping") || w.contains("disabled"),
            "{:?} warning doesn't contain 'skipping' or 'disabled': {}",
            feature,
            w
        );
    }
}

#[test]
fn disabled_feature_debug_format_is_readable() {
    let debug = format!("{:?}", DisabledFeature::GitMetrics);
    assert_eq!(debug, "GitMetrics");
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Deep preset superset invariant
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn deep_is_superset_of_every_non_fun_preset_base_flags() {
    let deep = preset_plan_for(PresetKind::Deep);
    for kind in PresetKind::all() {
        if *kind == PresetKind::Fun || *kind == PresetKind::Deep {
            continue;
        }
        let plan = preset_plan_for(*kind);
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
