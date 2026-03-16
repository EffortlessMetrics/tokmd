use proptest::prelude::*;
use tokmd_analysis_grid::*;

proptest! {
    /// PresetKind::as_str → from_str round-trip is stable for all presets.
    #[test]
    fn preset_roundtrip_all(idx in 0usize..12) {
        let preset = PRESET_KINDS[idx];
        let name = preset.as_str();
        let parsed = PresetKind::from_str(name);
        prop_assert_eq!(parsed, Some(preset));
    }

    /// preset_plan_for never panics for valid presets.
    #[test]
    fn preset_plan_for_never_panics(idx in 0usize..12) {
        let preset = PRESET_KINDS[idx];
        let _plan = preset_plan_for(preset);
    }

    /// preset_plan_for is deterministic.
    #[test]
    fn preset_plan_for_is_deterministic(idx in 0usize..12) {
        let preset = PRESET_KINDS[idx];
        let p1 = preset_plan_for(preset);
        let p2 = preset_plan_for(preset);
        prop_assert_eq!(p1, p2);
    }

    /// preset_plan_for_name matches preset_plan_for for known names.
    #[test]
    fn preset_plan_for_name_matches(idx in 0usize..12) {
        let preset = PRESET_KINDS[idx];
        let by_kind = preset_plan_for(preset);
        let by_name = preset_plan_for_name(preset.as_str());
        prop_assert_eq!(by_name, Some(by_kind));
    }

    /// Unknown preset names return None.
    #[test]
    fn unknown_preset_name_returns_none(name in "zzz_[a-z]{3,15}") {
        let result = PresetKind::from_str(&name);
        prop_assert!(result.is_none(),
            "Unknown name '{}' should not parse as a preset", name);
    }

    /// preset_plan_for_name returns None for unknown names.
    #[test]
    fn preset_plan_for_unknown_name_returns_none(name in "zzz_[a-z]{3,15}") {
        let result = preset_plan_for_name(&name);
        prop_assert!(result.is_none());
    }

    /// The grid has exactly 12 entries (one per preset kind).
    #[test]
    fn grid_has_correct_size(_dummy in 0..1u8) {
        prop_assert_eq!(PRESET_GRID.len(), 12);
        prop_assert_eq!(PRESET_KINDS.len(), 12);
        prop_assert_eq!(PresetKind::all().len(), 12);
    }

    /// Every preset in the grid matches its index in PRESET_KINDS.
    #[test]
    fn grid_entry_matches_preset_kinds(idx in 0usize..12) {
        prop_assert_eq!(PRESET_GRID[idx].preset, PRESET_KINDS[idx]);
    }

    /// All DisabledFeature warnings are non-empty strings.
    #[test]
    fn disabled_feature_warnings_nonempty(
        idx in prop::sample::select(vec![
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
        ])
    ) {
        let warning = idx.warning();
        prop_assert!(!warning.is_empty());
        // All warnings should mention what's disabled.
        prop_assert!(warning.contains("disabled") || warning.contains("skipping"),
            "Warning should mention disabled/skipping: '{}'", warning);
    }

    /// PresetPlan for "deep" has the most features enabled.
    #[test]
    fn deep_preset_is_most_comprehensive(_dummy in 0..1u8) {
        let deep = preset_plan_for(PresetKind::Deep);
        prop_assert!(deep.assets);
        prop_assert!(deep.deps);
        prop_assert!(deep.todo);
        prop_assert!(deep.dup);
        prop_assert!(deep.imports);
        prop_assert!(deep.git);
        prop_assert!(deep.archetype);
        prop_assert!(deep.topics);
        prop_assert!(deep.entropy);
        prop_assert!(deep.license);
        prop_assert!(deep.complexity);
        prop_assert!(deep.api_surface);
        // fun is NOT in deep
        prop_assert!(!deep.fun);
    }

    /// PresetPlan for "receipt" enables core enrichers (dup, git, complexity, api_surface).
    #[test]
    fn receipt_preset_enables_core_enrichers(_dummy in 0..1u8) {
        let receipt = preset_plan_for(PresetKind::Receipt);
        prop_assert!(!receipt.assets);
        prop_assert!(!receipt.deps);
        prop_assert!(!receipt.todo);
        prop_assert!(receipt.dup);
        prop_assert!(!receipt.imports);
        prop_assert!(receipt.git);
        prop_assert!(!receipt.fun);
        prop_assert!(!receipt.archetype);
        prop_assert!(!receipt.topics);
        prop_assert!(!receipt.entropy);
        prop_assert!(!receipt.license);
        prop_assert!(receipt.complexity);
        prop_assert!(receipt.api_surface);
    }
}
