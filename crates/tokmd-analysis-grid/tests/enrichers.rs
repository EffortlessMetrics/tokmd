//! Additional BDD-style tests for the analysis grid crate — preset plans
//! and feature-availability tracking.

use tokmd_analysis_grid::{
    DisabledFeature, PRESET_GRID, PresetKind, PresetPlan, preset_plan_for, preset_plan_for_name,
};

// ── Scenario: Preset plan field consistency ─────────────────────────────

#[test]
fn all_presets_with_assets_also_need_files() {
    for row in &PRESET_GRID {
        if row.plan.assets {
            assert!(
                row.plan.needs_files(),
                "{:?} has assets=true but needs_files()=false",
                row.preset
            );
        }
    }
}

#[test]
fn all_presets_with_imports_also_need_files() {
    for row in &PRESET_GRID {
        if row.plan.imports {
            assert!(
                row.plan.needs_files(),
                "{:?} has imports=true but needs_files()=false",
                row.preset
            );
        }
    }
}

#[test]
fn all_presets_with_entropy_also_need_files() {
    for row in &PRESET_GRID {
        if row.plan.entropy {
            assert!(
                row.plan.needs_files(),
                "{:?} has entropy=true but needs_files()=false",
                row.preset
            );
        }
    }
}

// ── Scenario: Preset plan mutual exclusions ─────────────────────────────

#[test]
fn receipt_is_the_only_preset_with_all_base_flags_off() {
    let receipt_plan = preset_plan_for(PresetKind::Receipt);
    // Verify Receipt is uniquely "empty" among all presets
    for row in &PRESET_GRID {
        if row.preset != PresetKind::Receipt {
            let p = &row.plan;
            let any_flag = p.assets
                || p.deps
                || p.todo
                || p.dup
                || p.imports
                || p.git
                || p.fun
                || p.archetype
                || p.topics
                || p.entropy
                || p.license
                || p.complexity
                || p.api_surface
                || p.effort;
            assert!(
                any_flag,
                "{:?} has all base flags off like Receipt",
                row.preset
            );
        }
    }
    // Receipt itself has all off
    assert!(!receipt_plan.assets);
    assert!(!receipt_plan.deps);
    assert!(!receipt_plan.todo);
    assert!(!receipt_plan.dup);
    assert!(!receipt_plan.imports);
    assert!(!receipt_plan.git);
    assert!(!receipt_plan.fun);
}

// ── Scenario: Plan lookup determinism ───────────────────────────────────

#[test]
fn plan_lookup_is_deterministic() {
    for row in &PRESET_GRID {
        let p1 = preset_plan_for(row.preset);
        let p2 = preset_plan_for(row.preset);
        assert_eq!(p1, p2, "plan lookup not deterministic for {:?}", row.preset);
    }
}

#[test]
fn plan_by_name_matches_plan_by_kind() {
    for row in &PRESET_GRID {
        let by_kind = preset_plan_for(row.preset);
        let by_name = preset_plan_for_name(row.preset.as_str()).unwrap();
        assert_eq!(
            by_kind, by_name,
            "plan mismatch between by-kind and by-name for {:?}",
            row.preset
        );
    }
}

// ── Scenario: DisabledFeature exhaustive coverage ──────────────────────

#[test]
fn disabled_feature_warning_strings_are_not_duplicated() {
    let all_features = [
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
    let msgs: Vec<&str> = all_features.iter().map(|f| f.warning()).collect();
    for (i, a) in msgs.iter().enumerate() {
        for (j, b) in msgs.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "duplicate warning between variant {i} and {j}");
            }
        }
    }
}

#[test]
fn disabled_feature_warnings_contain_skipping_or_disabled() {
    let all_features = [
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
    for feat in &all_features {
        let msg = feat.warning();
        assert!(
            msg.contains("skipping") || msg.contains("disabled"),
            "{:?} warning should contain 'skipping' or 'disabled': {msg}",
            feat
        );
    }
}

// ── Scenario: PresetPlan structural invariant ──────────────────────────

#[test]
fn preset_plan_is_copy_and_eq() {
    let plan = preset_plan_for(PresetKind::Deep);
    let copied: PresetPlan = plan;
    assert_eq!(plan, copied);
}
