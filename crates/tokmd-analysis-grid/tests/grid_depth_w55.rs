//! Comprehensive depth tests for tokmd-analysis-grid – wave 55.

use tokmd_analysis_grid::{
    DisabledFeature, PRESET_GRID, PRESET_KINDS, PresetKind, preset_plan_for, preset_plan_for_name,
};

// ── PresetKind basics ───────────────────────────────────────────────

#[test]
fn preset_kind_all_returns_eleven() {
    assert_eq!(PresetKind::all().len(), 12);
}

#[test]
fn preset_kinds_constant_matches_all() {
    assert_eq!(PRESET_KINDS.len(), PresetKind::all().len());
    for (a, b) in PRESET_KINDS.iter().zip(PresetKind::all().iter()) {
        assert_eq!(a, b);
    }
}

#[test]
fn as_str_roundtrip_all_presets() {
    for kind in PresetKind::all() {
        let s = kind.as_str();
        let parsed = PresetKind::from_str(s).unwrap();
        assert_eq!(parsed, *kind, "roundtrip failed for {s}");
    }
}

#[test]
fn from_str_unknown_returns_none() {
    assert!(PresetKind::from_str("unknown").is_none());
    assert!(PresetKind::from_str("").is_none());
    assert!(PresetKind::from_str("RECEIPT").is_none()); // case-sensitive
}

#[test]
fn as_str_values_are_lowercase() {
    for kind in PresetKind::all() {
        let s = kind.as_str();
        assert_eq!(s, s.to_lowercase(), "{s} is not lowercase");
    }
}

#[test]
fn preset_kind_debug_works() {
    let dbg = format!("{:?}", PresetKind::Receipt);
    assert!(dbg.contains("Receipt"));
}

#[test]
fn preset_kind_clone_eq() {
    let a = PresetKind::Deep;
    let b = a;
    assert_eq!(a, b);
}

// ── PRESET_GRID ─────────────────────────────────────────────────────

#[test]
fn grid_has_eleven_rows() {
    assert_eq!(PRESET_GRID.len(), 12);
}

#[test]
fn grid_covers_every_preset_kind() {
    for kind in PresetKind::all() {
        assert!(
            PRESET_GRID.iter().any(|row| row.preset == *kind),
            "grid missing preset {:?}",
            kind
        );
    }
}

#[test]
fn grid_rows_have_unique_presets() {
    let mut seen = std::collections::HashSet::new();
    for row in &PRESET_GRID {
        assert!(
            seen.insert(row.preset.as_str()),
            "duplicate preset {:?}",
            row.preset
        );
    }
}

#[test]
fn grid_row_debug_format() {
    let row = &PRESET_GRID[0];
    let dbg = format!("{:?}", row);
    assert!(dbg.contains("Receipt"));
}

// ── preset_plan_for / preset_plan_for_name ──────────────────────────

#[test]
fn plan_for_receipt_has_nothing_enabled() {
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
fn plan_for_health_enables_todo_and_complexity() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(plan.todo);
    assert!(plan.complexity);
    assert!(!plan.git);
    assert!(!plan.fun);
}

#[test]
fn plan_for_risk_enables_git_and_complexity() {
    let plan = preset_plan_for(PresetKind::Risk);
    assert!(plan.git);
    assert!(plan.complexity);
    assert!(!plan.todo);
}

#[test]
fn plan_for_supply_enables_assets_and_deps() {
    let plan = preset_plan_for(PresetKind::Supply);
    assert!(plan.assets);
    assert!(plan.deps);
    assert!(!plan.git);
}

#[test]
fn plan_for_architecture_enables_imports_and_api_surface() {
    let plan = preset_plan_for(PresetKind::Architecture);
    assert!(plan.imports);
    assert!(plan.api_surface);
    assert!(!plan.git);
}

#[test]
fn plan_for_topics_enables_topics_only() {
    let plan = preset_plan_for(PresetKind::Topics);
    assert!(plan.topics);
    assert!(!plan.git);
    assert!(!plan.todo);
    assert!(!plan.entropy);
}

#[test]
fn plan_for_security_enables_entropy_and_license() {
    let plan = preset_plan_for(PresetKind::Security);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(!plan.git);
}

#[test]
fn plan_for_identity_enables_git_and_archetype() {
    let plan = preset_plan_for(PresetKind::Identity);
    assert!(plan.git);
    assert!(plan.archetype);
}

#[test]
fn plan_for_git_enables_git() {
    let plan = preset_plan_for(PresetKind::Git);
    assert!(plan.git);
    assert!(!plan.todo);
    assert!(!plan.assets);
}

#[test]
fn plan_for_deep_enables_almost_everything() {
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
    assert!(!plan.fun, "deep should NOT include fun");
}

#[test]
fn plan_for_fun_enables_only_fun() {
    let plan = preset_plan_for(PresetKind::Fun);
    assert!(plan.fun);
    assert!(!plan.git);
    assert!(!plan.todo);
    assert!(!plan.assets);
}

#[test]
fn plan_for_name_valid() {
    assert!(preset_plan_for_name("receipt").is_some());
    assert!(preset_plan_for_name("deep").is_some());
}

#[test]
fn plan_for_name_invalid_returns_none() {
    assert!(preset_plan_for_name("bogus").is_none());
    assert!(preset_plan_for_name("").is_none());
}

// ── PresetPlan::needs_files ─────────────────────────────────────────

#[test]
fn receipt_needs_no_files() {
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

// ── DisabledFeature warnings ────────────────────────────────────────

#[test]
fn all_disabled_feature_warnings_non_empty() {
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
        assert!(!f.warning().is_empty(), "{f:?} has empty warning");
    }
}

#[test]
fn disabled_feature_warning_mentions_disabled() {
    assert!(DisabledFeature::GitMetrics.warning().contains("disabled"));
    assert!(DisabledFeature::TodoScan.warning().contains("disabled"));
}

#[test]
fn disabled_feature_debug_and_clone() {
    let f = DisabledFeature::FileInventory;
    let f2 = f;
    assert_eq!(f, f2);
    let dbg = format!("{:?}", f);
    assert!(dbg.contains("FileInventory"));
}
