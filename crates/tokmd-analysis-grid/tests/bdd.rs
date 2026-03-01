//! BDD-style scenario tests for the analysis grid crate.

use tokmd_analysis_grid::{
    DisabledFeature, PRESET_GRID, PRESET_KINDS, PresetGridRow, PresetKind, preset_plan_for,
    preset_plan_for_name,
};

// ── Scenario: PRESET_KINDS array completeness ──────────────────────────

#[test]
fn preset_kinds_array_has_exactly_11_entries() {
    assert_eq!(PRESET_KINDS.len(), 11);
}

#[test]
fn preset_kinds_all_returns_same_as_const() {
    let all = PresetKind::all();
    assert_eq!(all.len(), PRESET_KINDS.len());
    for (a, b) in all.iter().zip(PRESET_KINDS.iter()) {
        assert_eq!(a, b);
    }
}

// ── Scenario: Every preset has a unique name ────────────────────────────

#[test]
fn all_preset_names_are_unique() {
    let names: Vec<&str> = PRESET_KINDS.iter().map(|p| p.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(names.len(), sorted.len(), "duplicate preset names detected");
}

// ── Scenario: from_str / as_str roundtrip for every preset ─────────────

#[test]
fn roundtrip_from_str_as_str_for_all_presets() {
    for kind in PresetKind::all() {
        let name = kind.as_str();
        let parsed = PresetKind::from_str(name);
        assert_eq!(parsed, Some(*kind), "roundtrip failed for {name}");
    }
}

#[test]
fn from_str_returns_none_for_unknown_names() {
    assert_eq!(PresetKind::from_str(""), None);
    assert_eq!(PresetKind::from_str("nonexistent"), None);
    assert_eq!(PresetKind::from_str("RECEIPT"), None); // case-sensitive
    assert_eq!(PresetKind::from_str("Deep"), None);
    assert_eq!(PresetKind::from_str(" receipt"), None);
    assert_eq!(PresetKind::from_str("receipt "), None);
}

// ── Scenario: PRESET_GRID coverage ─────────────────────────────────────

#[test]
fn preset_grid_has_exactly_11_rows() {
    assert_eq!(PRESET_GRID.len(), 11);
}

#[test]
fn every_preset_kind_appears_in_grid_exactly_once() {
    for kind in PresetKind::all() {
        let matches: Vec<&PresetGridRow> =
            PRESET_GRID.iter().filter(|r| r.preset == *kind).collect();
        assert_eq!(
            matches.len(),
            1,
            "preset {:?} appears {} times in PRESET_GRID",
            kind,
            matches.len()
        );
    }
}

#[test]
fn grid_order_matches_preset_kinds_order() {
    for (row, kind) in PRESET_GRID.iter().zip(PRESET_KINDS.iter()) {
        assert_eq!(
            row.preset, *kind,
            "PRESET_GRID order does not match PRESET_KINDS order"
        );
    }
}

// ── Scenario: preset_plan_for lookups ──────────────────────────────────

#[test]
fn preset_plan_for_returns_correct_plan_for_each_preset() {
    for row in &PRESET_GRID {
        let plan = preset_plan_for(row.preset);
        assert_eq!(plan, row.plan, "plan mismatch for {:?}", row.preset);
    }
}

#[test]
fn preset_plan_for_name_returns_some_for_valid_names() {
    for kind in PresetKind::all() {
        let plan = preset_plan_for_name(kind.as_str());
        assert!(plan.is_some(), "expected Some for {:?}", kind);
        assert_eq!(plan.unwrap(), preset_plan_for(*kind));
    }
}

#[test]
fn preset_plan_for_name_returns_none_for_invalid_names() {
    assert!(preset_plan_for_name("").is_none());
    assert!(preset_plan_for_name("bogus").is_none());
    assert!(preset_plan_for_name("DEEP").is_none());
}

// ── Scenario: Receipt preset enables nothing ───────────────────────────

#[test]
fn receipt_preset_enables_no_enrichers() {
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
fn receipt_preset_does_not_need_files() {
    assert!(!preset_plan_for(PresetKind::Receipt).needs_files());
}

// ── Scenario: Deep preset enables everything (except fun) ──────────────

#[test]
fn deep_preset_enables_all_non_fun_enrichers() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.assets);
    assert!(plan.deps);
    assert!(plan.todo);
    assert!(plan.dup);
    assert!(plan.imports);
    assert!(plan.git);
    assert!(!plan.fun, "deep should NOT enable fun");
    assert!(plan.archetype);
    assert!(plan.topics);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(plan.complexity);
    assert!(plan.api_surface);
}

#[test]
fn deep_preset_needs_files() {
    assert!(preset_plan_for(PresetKind::Deep).needs_files());
}

// ── Scenario: Fun preset enables only fun ──────────────────────────────

#[test]
fn fun_preset_enables_only_fun_flag() {
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

#[test]
fn fun_preset_does_not_need_files() {
    assert!(!preset_plan_for(PresetKind::Fun).needs_files());
}

// ── Scenario: Individual preset plans match documentation ──────────────

#[test]
fn health_preset_enables_todo_and_complexity() {
    let plan = preset_plan_for(PresetKind::Health);
    assert!(plan.todo);
    assert!(plan.complexity);
    assert!(!plan.git);
    assert!(!plan.assets);
    assert!(!plan.deps);
}

#[test]
fn risk_preset_enables_git_and_complexity() {
    let plan = preset_plan_for(PresetKind::Risk);
    assert!(plan.git);
    assert!(plan.complexity);
    assert!(!plan.todo);
    assert!(!plan.assets);
}

#[test]
fn supply_preset_enables_assets_and_deps() {
    let plan = preset_plan_for(PresetKind::Supply);
    assert!(plan.assets);
    assert!(plan.deps);
    assert!(!plan.git);
    assert!(!plan.todo);
    assert!(!plan.imports);
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
    assert!(!plan.git);
    assert!(!plan.assets);
    assert!(!plan.imports);
}

#[test]
fn security_preset_enables_entropy_and_license() {
    let plan = preset_plan_for(PresetKind::Security);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(!plan.git);
    assert!(!plan.todo);
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
    assert!(!plan.archetype);
}

// ── Scenario: needs_files correctness ──────────────────────────────────

#[test]
fn needs_files_is_true_when_any_file_dependent_flag_is_set() {
    // Presets that should need files (they have at least one file-dependent flag)
    let needs_files_presets = [
        PresetKind::Health,       // todo
        PresetKind::Supply,       // assets, deps
        PresetKind::Architecture, // imports, api_surface
        PresetKind::Security,     // entropy, license
        PresetKind::Deep,         // everything
    ];

    for preset in &needs_files_presets {
        assert!(
            preset_plan_for(*preset).needs_files(),
            "{:?} should need files",
            preset
        );
    }
}

#[test]
fn needs_files_is_false_when_no_file_dependent_flags_set() {
    let no_files_presets = [PresetKind::Receipt, PresetKind::Fun];
    for preset in &no_files_presets {
        assert!(
            !preset_plan_for(*preset).needs_files(),
            "{:?} should not need files",
            preset
        );
    }
}

// ── Scenario: Presets with git flag set ─────────────────────────────────

#[test]
fn presets_requiring_git_are_risk_identity_git_deep() {
    let git_presets: Vec<PresetKind> = PRESET_GRID
        .iter()
        .filter(|r| r.plan.git)
        .map(|r| r.preset)
        .collect();
    assert!(git_presets.contains(&PresetKind::Risk));
    assert!(git_presets.contains(&PresetKind::Identity));
    assert!(git_presets.contains(&PresetKind::Git));
    assert!(git_presets.contains(&PresetKind::Deep));
    assert_eq!(git_presets.len(), 4);
}

// ── Scenario: DisabledFeature exhaustive warnings ──────────────────────

#[test]
fn all_disabled_feature_warnings_are_non_empty() {
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
        assert!(!msg.is_empty(), "{:?} has empty warning", feat);
    }
}

#[test]
fn all_disabled_feature_warnings_mention_disabled_or_feature() {
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
            msg.contains("disabled") || msg.contains("feature"),
            "{:?} warning should mention 'disabled' or 'feature': {msg}",
            feat
        );
    }
}

#[test]
fn disabled_feature_warnings_are_all_unique() {
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
    let messages: Vec<&str> = features.iter().map(|f| f.warning()).collect();
    let mut sorted = messages.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        messages.len(),
        sorted.len(),
        "duplicate warning messages detected"
    );
}

// ── Scenario: Debug and equality traits ─────────────────────────────────

#[test]
fn preset_kind_debug_is_non_empty() {
    for kind in PresetKind::all() {
        let debug = format!("{:?}", kind);
        assert!(!debug.is_empty());
    }
}

#[test]
fn preset_kind_clone_and_copy() {
    let a = PresetKind::Receipt;
    let b = a;
    let c = a;
    assert_eq!(a, b);
    assert_eq!(a, c);
}

#[test]
fn preset_plan_clone_and_copy() {
    let plan = preset_plan_for(PresetKind::Deep);
    let cloned = plan;
    let copied = plan;
    assert_eq!(plan, cloned);
    assert_eq!(plan, copied);
}

#[test]
fn disabled_feature_clone_and_copy() {
    let a = DisabledFeature::GitMetrics;
    let b = a;
    let c = a;
    assert_eq!(a, b);
    assert_eq!(a, c);
}

#[test]
fn preset_grid_row_debug_is_non_empty() {
    for row in &PRESET_GRID {
        let debug = format!("{:?}", row);
        assert!(!debug.is_empty());
    }
}

// ── Scenario: Preset names are lowercase ASCII ─────────────────────────

#[test]
fn all_preset_names_are_lowercase_ascii() {
    for kind in PresetKind::all() {
        let name = kind.as_str();
        assert!(
            name.chars().all(|c| c.is_ascii_lowercase()),
            "preset name {name:?} contains non-lowercase-ascii chars"
        );
    }
}
