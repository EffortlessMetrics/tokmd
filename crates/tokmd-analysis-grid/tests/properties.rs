//! Property-based tests for the analysis grid crate.

use proptest::prelude::*;
use tokmd_analysis_grid::{
    PRESET_GRID, PRESET_KINDS, PresetKind, preset_plan_for, preset_plan_for_name,
};

/// Strategy that produces a valid preset name string.
fn valid_preset_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(PRESET_KINDS.iter().map(|k| k.as_str()).collect::<Vec<_>>())
}

/// Strategy that produces an arbitrary string (mostly invalid preset names).
fn arbitrary_string() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9_ ]{0,30}").unwrap()
}

/// Strategy that produces a PresetKind by index.
fn preset_kind_strategy() -> impl Strategy<Value = PresetKind> {
    (0..PRESET_KINDS.len()).prop_map(|i| PRESET_KINDS[i])
}

proptest! {
    /// For any valid preset name, from_str always succeeds and roundtrips.
    #[test]
    fn from_str_roundtrip_always_succeeds(name in valid_preset_name()) {
        let kind = PresetKind::from_str(name).unwrap();
        prop_assert_eq!(kind.as_str(), name);
    }

    /// For any PresetKind, as_str is non-empty and lowercase ASCII.
    #[test]
    fn as_str_always_returns_lowercase_ascii(kind in preset_kind_strategy()) {
        let name = kind.as_str();
        prop_assert!(!name.is_empty());
        prop_assert!(name.chars().all(|c| c.is_ascii_lowercase()));
    }

    /// For any PresetKind, preset_plan_for matches the grid entry.
    #[test]
    fn preset_plan_for_matches_grid(kind in preset_kind_strategy()) {
        let plan = preset_plan_for(kind);
        let grid_plan = PRESET_GRID.iter().find(|r| r.preset == kind).unwrap().plan;
        prop_assert_eq!(plan, grid_plan);
    }

    /// For any valid preset name, preset_plan_for_name returns Some.
    #[test]
    fn preset_plan_for_name_returns_some_for_valid(name in valid_preset_name()) {
        prop_assert!(preset_plan_for_name(name).is_some());
    }

    /// For any arbitrary string, from_str either returns a valid preset or None —
    /// it never panics.
    #[test]
    fn from_str_never_panics(s in arbitrary_string()) {
        let _ = PresetKind::from_str(&s);
    }

    /// For any arbitrary string, preset_plan_for_name either returns Some or None —
    /// it never panics.
    #[test]
    fn preset_plan_for_name_never_panics(s in arbitrary_string()) {
        let _ = preset_plan_for_name(&s);
    }

    /// If from_str returns Some, the returned kind roundtrips through the grid.
    #[test]
    fn from_str_result_always_in_grid(s in arbitrary_string()) {
        if let Some(kind) = PresetKind::from_str(&s) {
            prop_assert!(PRESET_GRID.iter().any(|r| r.preset == kind));
        }
    }

    /// Grid lookup is deterministic: same kind always yields same plan.
    #[test]
    fn grid_lookup_is_deterministic(kind in preset_kind_strategy()) {
        let plan_a = preset_plan_for(kind);
        let plan_b = preset_plan_for(kind);
        prop_assert_eq!(plan_a, plan_b);
    }

    /// Round-trip serialization via as_str/from_str is stable for all presets.
    #[test]
    fn string_roundtrip_serialization(kind in preset_kind_strategy()) {
        let s = kind.as_str();
        let roundtripped = PresetKind::from_str(s).unwrap();
        prop_assert_eq!(roundtripped, kind);
        prop_assert_eq!(roundtripped.as_str(), s);
    }

    /// Round-trip through preset_plan_for_name matches preset_plan_for.
    #[test]
    fn plan_roundtrip_via_name(kind in preset_kind_strategy()) {
        let via_name = preset_plan_for_name(kind.as_str()).unwrap();
        let direct = preset_plan_for(kind);
        prop_assert_eq!(via_name, direct);
    }

    /// For any PresetKind, the Deep preset plan has at least as many enabled flags
    /// as any other preset (except fun which Deep disables).
    #[test]
    fn deep_is_superset_of_non_fun_flags(kind in preset_kind_strategy()) {
        let deep = preset_plan_for(PresetKind::Deep);
        let plan = preset_plan_for(kind);

        // Deep enables every non-fun flag that any other preset enables
        if plan.assets { prop_assert!(deep.assets); }
        if plan.deps { prop_assert!(deep.deps); }
        if plan.todo { prop_assert!(deep.todo); }
        if plan.dup { prop_assert!(deep.dup); }
        if plan.imports { prop_assert!(deep.imports); }
        if plan.git { prop_assert!(deep.git); }
        if plan.archetype { prop_assert!(deep.archetype); }
        if plan.topics { prop_assert!(deep.topics); }
        if plan.entropy { prop_assert!(deep.entropy); }
        if plan.license { prop_assert!(deep.license); }
        if plan.complexity { prop_assert!(deep.complexity); }
        if plan.api_surface { prop_assert!(deep.api_surface); }
    }
}
