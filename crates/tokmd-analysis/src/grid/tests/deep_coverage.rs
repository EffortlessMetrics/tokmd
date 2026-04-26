//! Deep coverage tests for `analysis grid module`.
//!
//! Exercises preset recognition, feature matrix metadata, preset inclusion/
//! exclusion logic, determinism, and preset plan properties.

use crate::grid::{
    DisabledFeature, PRESET_GRID, PRESET_KINDS, PresetKind, preset_plan_for, preset_plan_for_name,
};

// ===========================================================================
// All preset names are recognized
// ===========================================================================

#[test]
fn all_preset_names_round_trip() {
    for kind in PresetKind::all() {
        let name = kind.as_str();
        let parsed = PresetKind::from_str(name);
        assert!(
            parsed.is_some(),
            "PresetKind::from_str({name:?}) returned None"
        );
        assert_eq!(parsed.unwrap(), *kind);
    }
}

#[test]
fn unknown_name_returns_none() {
    assert!(PresetKind::from_str("nonexistent").is_none());
    assert!(PresetKind::from_str("").is_none());
    assert!(PresetKind::from_str("RECEIPT").is_none()); // case sensitive
    assert!(PresetKind::from_str("Deep").is_none());
}

#[test]
fn preset_plan_for_name_returns_none_for_unknown() {
    assert!(preset_plan_for_name("nope").is_none());
    assert!(preset_plan_for_name("").is_none());
}

#[test]
fn preset_plan_for_name_matches_preset_plan_for() {
    for kind in PresetKind::all() {
        let by_kind = preset_plan_for(*kind);
        let by_name = preset_plan_for_name(kind.as_str()).unwrap();
        assert_eq!(by_kind, by_name);
    }
}

// ===========================================================================
// Feature matrix metadata
// ===========================================================================

#[test]
fn grid_length_matches_kinds_length() {
    assert_eq!(PRESET_GRID.len(), PRESET_KINDS.len());
    assert_eq!(PRESET_GRID.len(), 12);
}

#[test]
fn grid_covers_all_preset_kinds() {
    for kind in PresetKind::all() {
        let found = PRESET_GRID.iter().any(|row| row.preset == *kind);
        assert!(found, "PresetKind::{:?} not found in PRESET_GRID", kind);
    }
}

#[test]
fn grid_has_no_duplicate_presets() {
    let mut seen = std::collections::BTreeSet::new();
    for row in &PRESET_GRID {
        assert!(
            seen.insert(row.preset.as_str()),
            "Duplicate preset in grid: {:?}",
            row.preset
        );
    }
}

// ===========================================================================
// Preset inclusion/exclusion logic
// ===========================================================================

#[test]
fn receipt_preset_enables_core_enrichers() {
    let plan = preset_plan_for(PresetKind::Receipt);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(plan.dup);
    assert!(!plan.imports);
    assert!(plan.git);
    assert!(!plan.fun);
    assert!(!plan.archetype);
    assert!(!plan.topics);
    assert!(!plan.entropy);
    assert!(!plan.license);
    assert!(plan.complexity);
    assert!(plan.api_surface);
}

#[test]
fn supply_preset_enables_assets_and_deps() {
    let plan = preset_plan_for(PresetKind::Supply);
    assert!(plan.assets);
    assert!(plan.deps);
    assert!(!plan.todo);
    assert!(!plan.git);
    assert!(!plan.fun);
}

#[test]
fn health_preset_enables_todo_and_complexity() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(plan.todo);
    assert!(plan.complexity);
    assert!(!plan.assets);
    assert!(!plan.git);
}

#[test]
fn risk_preset_enables_git_and_complexity() {
    let plan = preset_plan_for(PresetKind::Risk);
    assert!(plan.git);
    assert!(plan.complexity);
    assert!(!plan.assets);
    assert!(!plan.todo);
}

#[test]
fn architecture_preset_enables_imports_and_api_surface() {
    let plan = preset_plan_for(PresetKind::Architecture);
    assert!(plan.imports);
    assert!(plan.api_surface);
    assert!(!plan.git);
    assert!(!plan.assets);
}

#[test]
fn topics_preset_enables_only_topics() {
    let plan = preset_plan_for(PresetKind::Topics);
    assert!(plan.topics);
    assert!(!plan.assets);
    assert!(!plan.git);
    assert!(!plan.todo);
}

#[test]
fn security_preset_enables_entropy_and_license() {
    let plan = preset_plan_for(PresetKind::Security);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(!plan.assets);
    assert!(!plan.git);
}

#[test]
fn identity_preset_enables_git_and_archetype() {
    let plan = preset_plan_for(PresetKind::Identity);
    assert!(plan.git);
    assert!(plan.archetype);
    assert!(!plan.assets);
    assert!(!plan.todo);
}

#[test]
fn git_preset_enables_git() {
    let plan = preset_plan_for(PresetKind::Git);
    assert!(plan.git);
    assert!(!plan.assets);
    assert!(!plan.todo);
    assert!(!plan.fun);
}

#[test]
fn fun_preset_enables_only_fun() {
    let plan = preset_plan_for(PresetKind::Fun);
    assert!(plan.fun);
    assert!(!plan.assets);
    assert!(!plan.git);
    assert!(!plan.todo);
    assert!(!plan.imports);
}

#[test]
fn deep_preset_enables_everything_except_fun() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.assets);
    assert!(plan.deps);
    assert!(plan.todo);
    assert!(plan.dup);
    assert!(plan.imports);
    assert!(plan.git);
    assert!(plan.archetype);
    assert!(plan.topics);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(plan.complexity);
    assert!(plan.api_surface);
    assert!(!plan.fun);
}

// ===========================================================================
// needs_files logic
// ===========================================================================

#[test]
fn receipt_needs_files_is_true() {
    let plan = preset_plan_for(PresetKind::Receipt);
    assert!(plan.needs_files());
}

#[test]
fn supply_needs_files_is_true() {
    let plan = preset_plan_for(PresetKind::Supply);
    assert!(plan.needs_files());
}

#[test]
fn health_needs_files_is_true() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(plan.needs_files());
}

#[test]
fn deep_needs_files_is_true() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.needs_files());
}

#[test]
fn fun_needs_files_is_false() {
    let plan = preset_plan_for(PresetKind::Fun);
    assert!(!plan.needs_files());
}

#[test]
fn git_needs_files_is_false() {
    let plan = preset_plan_for(PresetKind::Git);
    assert!(!plan.needs_files());
}

// ===========================================================================
// Grid is deterministic
// ===========================================================================

#[test]
fn grid_order_is_stable() {
    let names: Vec<&str> = PRESET_GRID.iter().map(|row| row.preset.as_str()).collect();
    let expected = vec![
        "receipt",
        "estimate",
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
    assert_eq!(names, expected);
}

#[test]
fn preset_plan_for_is_deterministic() {
    for kind in PresetKind::all() {
        let p1 = preset_plan_for(*kind);
        let p2 = preset_plan_for(*kind);
        assert_eq!(p1, p2);
    }
}

// ===========================================================================
// DisabledFeature warnings
// ===========================================================================

#[test]
fn all_disabled_features_have_nonempty_warning() {
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
        assert!(!msg.is_empty(), "{f:?} has empty warning");
        assert!(
            msg.contains("disabled") || msg.contains("skipping"),
            "{f:?} warning doesn't mention disabled/skipping: {msg}"
        );
    }
}

#[test]
fn disabled_feature_eq_and_clone() {
    let a = DisabledFeature::FileInventory;
    let b = DisabledFeature::FileInventory;
    assert_eq!(a, b);
    let c = DisabledFeature::TodoScan;
    assert_ne!(a, c);
}

#[test]
fn disabled_feature_debug_contains_variant() {
    let dbg = format!("{:?}", DisabledFeature::GitMetrics);
    assert!(dbg.contains("GitMetrics"));
}

// ===========================================================================
// PresetKind traits
// ===========================================================================

#[test]
fn preset_kind_eq_and_copy() {
    let a = PresetKind::Receipt;
    let b = a; // Copy
    assert_eq!(a, b);
}

#[test]
fn preset_kind_debug() {
    let dbg = format!("{:?}", PresetKind::Deep);
    assert!(dbg.contains("Deep"));
}
