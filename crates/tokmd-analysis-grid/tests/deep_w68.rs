//! W68 deep tests for `tokmd-analysis-grid`.
//!
//! Covers preset plan correctness, needs_files invariants, feature-flag
//! mapping, deep-superset guarantee, preset uniqueness, disabled-feature
//! messages, and deterministic ordering.

use tokmd_analysis_grid::{
    DisabledFeature, PRESET_GRID, PRESET_KINDS, PresetKind, preset_plan_for, preset_plan_for_name,
};

// ═══════════════════════════════════════════════════════════════════
// § 1. Receipt preset enables no enrichers
// ═══════════════════════════════════════════════════════════════════

#[test]
fn receipt_enables_nothing() {
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

// ═══════════════════════════════════════════════════════════════════
// § 2. Receipt preset needs_files is false
// ═══════════════════════════════════════════════════════════════════

#[test]
fn receipt_needs_no_files() {
    let plan = preset_plan_for(PresetKind::Receipt);
    assert!(!plan.needs_files());
}

// ═══════════════════════════════════════════════════════════════════
// § 3. Fun preset only enables fun
// ═══════════════════════════════════════════════════════════════════

#[test]
fn fun_only_enables_fun() {
    let plan = preset_plan_for(PresetKind::Fun);
    assert!(plan.fun);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(!plan.dup);
    assert!(!plan.imports);
    assert!(!plan.git);
    assert!(!plan.archetype);
    assert!(!plan.topics);
    assert!(!plan.entropy);
    assert!(!plan.license);
    assert!(!plan.complexity);
    assert!(!plan.api_surface);
}

// ═══════════════════════════════════════════════════════════════════
// § 4. Supply preset enables assets + deps
// ═══════════════════════════════════════════════════════════════════

#[test]
fn supply_enables_assets_and_deps() {
    let plan = preset_plan_for(PresetKind::Supply);
    assert!(plan.assets);
    assert!(plan.deps);
    assert!(!plan.todo);
    assert!(!plan.git);
    assert!(!plan.fun);
}

// ═══════════════════════════════════════════════════════════════════
// § 5. Architecture preset enables imports + api_surface
// ═══════════════════════════════════════════════════════════════════

#[test]
fn architecture_enables_imports_and_api_surface() {
    let plan = preset_plan_for(PresetKind::Architecture);
    assert!(plan.imports);
    assert!(plan.api_surface);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.git);
}

// ═══════════════════════════════════════════════════════════════════
// § 6. Health preset enables todo + complexity
// ═══════════════════════════════════════════════════════════════════

#[test]
fn health_enables_todo_and_complexity() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(plan.todo);
    assert!(plan.complexity);
    assert!(!plan.git);
    assert!(!plan.assets);
}

// ═══════════════════════════════════════════════════════════════════
// § 7. Risk preset enables git + complexity
// ═══════════════════════════════════════════════════════════════════

#[test]
fn risk_enables_git_and_complexity() {
    let plan = preset_plan_for(PresetKind::Risk);
    assert!(plan.git);
    assert!(plan.complexity);
    assert!(!plan.assets);
    assert!(!plan.fun);
}

// ═══════════════════════════════════════════════════════════════════
// § 8. Security preset enables entropy + license
// ═══════════════════════════════════════════════════════════════════

#[test]
fn security_enables_entropy_and_license() {
    let plan = preset_plan_for(PresetKind::Security);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(!plan.git);
    assert!(!plan.todo);
}

// ═══════════════════════════════════════════════════════════════════
// § 9. Deep preset is superset of all base-flag enrichers
// ═══════════════════════════════════════════════════════════════════

#[test]
fn deep_is_superset_of_all_base_flags() {
    let deep = preset_plan_for(PresetKind::Deep);
    assert!(deep.assets);
    assert!(deep.deps);
    assert!(deep.todo);
    assert!(deep.dup);
    assert!(deep.imports);
    assert!(deep.git);
    assert!(deep.archetype);
    assert!(deep.topics);
    assert!(deep.entropy);
    assert!(deep.license);
    assert!(deep.complexity);
    assert!(deep.api_surface);
    // fun is intentionally excluded from deep
    assert!(!deep.fun);
}

// ═══════════════════════════════════════════════════════════════════
// § 10. Deep preset needs_files is true
// ═══════════════════════════════════════════════════════════════════

#[test]
fn deep_needs_files() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.needs_files());
}

// ═══════════════════════════════════════════════════════════════════
// § 11. needs_files true for any file-dependent enricher
// ═══════════════════════════════════════════════════════════════════

#[test]
fn needs_files_true_for_file_dependent_presets() {
    let file_presets = [
        PresetKind::Health,
        PresetKind::Supply,
        PresetKind::Architecture,
        PresetKind::Security,
        PresetKind::Deep,
    ];
    for kind in &file_presets {
        let plan = preset_plan_for(*kind);
        assert!(plan.needs_files(), "preset {:?} should need files", kind);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 12. PRESET_GRID has exactly 12 entries
// ═══════════════════════════════════════════════════════════════════

#[test]
fn grid_has_eleven_entries() {
    assert_eq!(PRESET_GRID.len(), 12);
    assert_eq!(PRESET_KINDS.len(), 12);
    assert_eq!(PresetKind::all().len(), 12);
}

// ═══════════════════════════════════════════════════════════════════
// § 13. Every preset in PRESET_KINDS has a grid row
// ═══════════════════════════════════════════════════════════════════

#[test]
fn every_kind_has_grid_row() {
    for kind in PresetKind::all() {
        let found = PRESET_GRID.iter().any(|row| row.preset == *kind);
        assert!(found, "PresetKind::{:?} missing from PRESET_GRID", kind);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 14. DisabledFeature warnings are non-empty and unique
// ═══════════════════════════════════════════════════════════════════

#[test]
fn disabled_feature_warnings_non_empty_and_unique() {
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
    let mut seen = std::collections::BTreeSet::new();
    for feat in &features {
        let msg = feat.warning();
        assert!(!msg.is_empty(), "{:?} has empty warning", feat);
        assert!(seen.insert(msg), "duplicate warning message: {}", msg);
    }
}

// ═══════════════════════════════════════════════════════════════════
// § 15. preset_plan_for_name returns None for invalid names
// ═══════════════════════════════════════════════════════════════════

#[test]
fn preset_plan_for_name_rejects_invalid() {
    assert!(preset_plan_for_name("").is_none());
    assert!(preset_plan_for_name("DEEP").is_none());
    assert!(preset_plan_for_name("unknown").is_none());
    assert!(preset_plan_for_name("receipt ").is_none());
    assert!(preset_plan_for_name(" receipt").is_none());
}
