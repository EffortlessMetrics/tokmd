//! Unit tests for `tokmd-analysis-grid` preset resolution, enricher inclusion,
//! and feature matrix metadata.

use tokmd_analysis_grid::{
    DisabledFeature, PRESET_GRID, PRESET_KINDS, PresetKind, preset_plan_for, preset_plan_for_name,
};

// ── Preset resolution ───────────────────────────────────────────────────────

#[test]
fn preset_plan_for_each_kind_matches_grid_row() {
    for row in &PRESET_GRID {
        let resolved = preset_plan_for(row.preset);
        assert_eq!(
            resolved, row.plan,
            "preset_plan_for({:?}) diverges from PRESET_GRID entry",
            row.preset
        );
    }
}

#[test]
fn preset_plan_for_name_resolves_all_known_names() {
    let names = [
        "receipt",
        "health",
        "risk",
        "supply",
        "architecture",
        "topics",
        "security",
        "identity",
        "git",
        "deep",
        "fun",
    ];
    for name in &names {
        let plan = preset_plan_for_name(name);
        assert!(plan.is_some(), "expected Some for name '{}'", name);
    }
}

#[test]
fn preset_plan_for_name_rejects_invalid_inputs() {
    for input in &["", " ", "RECEIPT", "Health", "unknown", "deep ", " deep"] {
        assert!(
            preset_plan_for_name(input).is_none(),
            "expected None for '{}'",
            input
        );
    }
}

// ── Enricher inclusion per preset ───────────────────────────────────────────

#[test]
fn health_enables_todo_complexity_only() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(plan.todo);
    assert!(plan.complexity);
    // Must not enable unrelated enrichers
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.dup);
    assert!(!plan.imports);
    assert!(!plan.git);
    assert!(!plan.fun);
    assert!(!plan.archetype);
    assert!(!plan.topics);
    assert!(!plan.entropy);
    assert!(!plan.license);
    assert!(!plan.api_surface);
}

#[test]
fn risk_enables_git_and_complexity_only() {
    let plan = preset_plan_for(PresetKind::Risk);
    assert!(plan.git);
    assert!(plan.complexity);
    assert!(!plan.todo);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.dup);
    assert!(!plan.imports);
    assert!(!plan.fun);
    assert!(!plan.archetype);
    assert!(!plan.topics);
    assert!(!plan.entropy);
    assert!(!plan.license);
    assert!(!plan.api_surface);
}

#[test]
fn supply_enables_assets_deps_only() {
    let plan = preset_plan_for(PresetKind::Supply);
    assert!(plan.assets);
    assert!(plan.deps);
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
fn architecture_enables_imports_api_surface_only() {
    let plan = preset_plan_for(PresetKind::Architecture);
    assert!(plan.imports);
    assert!(plan.api_surface);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(!plan.dup);
    assert!(!plan.git);
    assert!(!plan.fun);
    assert!(!plan.archetype);
    assert!(!plan.topics);
    assert!(!plan.entropy);
    assert!(!plan.license);
    assert!(!plan.complexity);
}

#[test]
fn security_enables_entropy_license_only() {
    let plan = preset_plan_for(PresetKind::Security);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(!plan.dup);
    assert!(!plan.imports);
    assert!(!plan.git);
    assert!(!plan.fun);
    assert!(!plan.archetype);
    assert!(!plan.topics);
    assert!(!plan.complexity);
    assert!(!plan.api_surface);
}

#[test]
fn identity_enables_git_archetype_only() {
    let plan = preset_plan_for(PresetKind::Identity);
    assert!(plan.git);
    assert!(plan.archetype);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(!plan.dup);
    assert!(!plan.imports);
    assert!(!plan.fun);
    assert!(!plan.topics);
    assert!(!plan.entropy);
    assert!(!plan.license);
    assert!(!plan.complexity);
    assert!(!plan.api_surface);
}

// ── Feature matrix metadata ─────────────────────────────────────────────────

#[test]
fn grid_length_equals_preset_kinds_length() {
    assert_eq!(PRESET_GRID.len(), PRESET_KINDS.len());
}

#[test]
fn needs_files_true_iff_any_file_flag_set() {
    for row in &PRESET_GRID {
        let plan = &row.plan;
        let any_file_flag = plan.assets
            || plan.deps
            || plan.todo
            || plan.dup
            || plan.imports
            || plan.entropy
            || plan.license
            || plan.complexity
            || plan.api_surface;
        assert_eq!(
            plan.needs_files(),
            any_file_flag,
            "needs_files mismatch for {:?}",
            row.preset
        );
    }
}

#[test]
fn only_fun_preset_sets_fun_flag() {
    for row in &PRESET_GRID {
        if row.preset == PresetKind::Fun {
            assert!(row.plan.fun);
        } else {
            assert!(!row.plan.fun, "{:?} unexpectedly has fun=true", row.preset);
        }
    }
}

#[test]
fn deep_is_superset_of_every_non_fun_preset() {
    let deep = preset_plan_for(PresetKind::Deep);
    for kind in PresetKind::all() {
        if *kind == PresetKind::Fun {
            continue;
        }
        let plan = preset_plan_for(*kind);
        if plan.assets {
            assert!(deep.assets, "deep missing assets from {:?}", kind);
        }
        if plan.deps {
            assert!(deep.deps, "deep missing deps from {:?}", kind);
        }
        if plan.todo {
            assert!(deep.todo, "deep missing todo from {:?}", kind);
        }
        if plan.dup {
            assert!(deep.dup, "deep missing dup from {:?}", kind);
        }
        if plan.imports {
            assert!(deep.imports, "deep missing imports from {:?}", kind);
        }
        if plan.git {
            assert!(deep.git, "deep missing git from {:?}", kind);
        }
        if plan.archetype {
            assert!(deep.archetype, "deep missing archetype from {:?}", kind);
        }
        if plan.topics {
            assert!(deep.topics, "deep missing topics from {:?}", kind);
        }
        if plan.entropy {
            assert!(deep.entropy, "deep missing entropy from {:?}", kind);
        }
        if plan.license {
            assert!(deep.license, "deep missing license from {:?}", kind);
        }
        if plan.complexity {
            assert!(deep.complexity, "deep missing complexity from {:?}", kind);
        }
        if plan.api_surface {
            assert!(deep.api_surface, "deep missing api_surface from {:?}", kind);
        }
    }
}

// ── DisabledFeature warnings ────────────────────────────────────────────────

#[test]
fn disabled_feature_warning_mentions_skipping_or_feature() {
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
    for feat in &features {
        let msg = feat.warning();
        assert!(
            msg.contains("skipping") || msg.contains("feature"),
            "{:?} warning should mention 'skipping' or 'feature', got: {}",
            feat,
            msg
        );
    }
}

#[test]
fn disabled_feature_count_matches_expected() {
    // There are exactly 13 DisabledFeature variants
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

// ── PresetKind traits ───────────────────────────────────────────────────────

#[test]
fn preset_kind_equality_is_reflexive() {
    for kind in PresetKind::all() {
        assert_eq!(*kind, *kind);
    }
}

#[test]
fn preset_kind_inequality_across_variants() {
    let kinds: Vec<PresetKind> = PresetKind::all().to_vec();
    for (i, a) in kinds.iter().enumerate() {
        for (j, b) in kinds.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "{:?} should not equal {:?}", a, b);
            }
        }
    }
}

// ── PresetPlan equality ─────────────────────────────────────────────────────

#[test]
fn distinct_presets_may_have_distinct_plans() {
    let receipt = preset_plan_for(PresetKind::Receipt);
    let deep = preset_plan_for(PresetKind::Deep);
    assert_ne!(receipt, deep);
}
