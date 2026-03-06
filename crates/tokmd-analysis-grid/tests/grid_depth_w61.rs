//! W61 depth tests for tokmd-analysis-grid: BDD edge cases, determinism, proptest.

use tokmd_analysis_grid::{
    DisabledFeature, PRESET_GRID, PRESET_KINDS, PresetKind, preset_plan_for, preset_plan_for_name,
};

// ---------------------------------------------------------------------------
// BDD: PresetKind string conversion edge cases
// ---------------------------------------------------------------------------

#[test]
fn from_str_returns_none_for_empty_string() {
    assert_eq!(PresetKind::from_str(""), None);
}

#[test]
fn from_str_returns_none_for_uppercase_variant() {
    assert_eq!(PresetKind::from_str("RECEIPT"), None);
    assert_eq!(PresetKind::from_str("Deep"), None);
    assert_eq!(PresetKind::from_str("FUN"), None);
}

#[test]
fn from_str_returns_none_for_leading_trailing_whitespace() {
    assert_eq!(PresetKind::from_str(" receipt"), None);
    assert_eq!(PresetKind::from_str("receipt "), None);
    assert_eq!(PresetKind::from_str(" receipt "), None);
}

#[test]
fn from_str_returns_none_for_unicode_lookalike() {
    // Cyrillic 'е' looks like Latin 'e' but differs
    assert_eq!(PresetKind::from_str("r\u{0435}ceipt"), None);
}

#[test]
fn as_str_values_are_all_lowercase_ascii() {
    for kind in PresetKind::all() {
        let s = kind.as_str();
        assert!(
            s.bytes().all(|b| b.is_ascii_lowercase()),
            "as_str for {:?} has non-lowercase chars",
            kind
        );
    }
}

#[test]
fn as_str_values_contain_no_whitespace() {
    for kind in PresetKind::all() {
        let s = kind.as_str();
        assert!(
            !s.contains(char::is_whitespace),
            "as_str for {:?} contains whitespace",
            kind
        );
    }
}

// ---------------------------------------------------------------------------
// BDD: PresetKind::all() and PRESET_KINDS
// ---------------------------------------------------------------------------

#[test]
fn all_returns_exactly_eleven_presets() {
    assert_eq!(PresetKind::all().len(), 11);
}

#[test]
fn preset_kinds_array_matches_all() {
    let all = PresetKind::all();
    assert_eq!(PRESET_KINDS.len(), all.len());
    for (a, b) in PRESET_KINDS.iter().zip(all.iter()) {
        assert_eq!(a, b);
    }
}

#[test]
fn all_preset_names_are_unique() {
    let names: Vec<&str> = PresetKind::all().iter().map(|k| k.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(names.len(), sorted.len(), "Duplicate preset names detected");
}

// ---------------------------------------------------------------------------
// BDD: PRESET_GRID structure
// ---------------------------------------------------------------------------

#[test]
fn grid_length_matches_preset_count() {
    assert_eq!(PRESET_GRID.len(), PresetKind::all().len());
}

#[test]
fn grid_rows_ordered_same_as_all() {
    for (row, kind) in PRESET_GRID.iter().zip(PresetKind::all().iter()) {
        assert_eq!(
            row.preset, *kind,
            "Grid row order doesn't match PresetKind::all()"
        );
    }
}

#[test]
fn grid_has_no_duplicate_preset_kinds() {
    let mut seen = std::collections::HashSet::new();
    for row in &PRESET_GRID {
        assert!(
            seen.insert(row.preset.as_str()),
            "Duplicate preset in grid: {:?}",
            row.preset
        );
    }
}

// ---------------------------------------------------------------------------
// BDD: preset_plan_for correctness
// ---------------------------------------------------------------------------

#[test]
fn receipt_plan_has_all_flags_false() {
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
fn fun_plan_only_enables_fun() {
    let plan = preset_plan_for(PresetKind::Fun);
    assert!(!plan.assets);
    assert!(!plan.deps);
    assert!(!plan.todo);
    assert!(!plan.dup);
    assert!(!plan.imports);
    assert!(!plan.git);
    assert!(plan.fun, "Fun preset must enable fun flag");
    assert!(!plan.archetype);
    assert!(!plan.topics);
    assert!(!plan.entropy);
    assert!(!plan.license);
    assert!(!plan.complexity);
    assert!(!plan.api_surface);
}

#[test]
fn deep_enables_everything_except_fun() {
    let plan = preset_plan_for(PresetKind::Deep);
    assert!(plan.assets);
    assert!(plan.deps);
    assert!(plan.todo);
    assert!(plan.dup);
    assert!(plan.imports);
    assert!(plan.git);
    assert!(!plan.fun, "Deep must NOT enable fun");
    assert!(plan.archetype);
    assert!(plan.topics);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(plan.complexity);
    assert!(plan.api_surface);
}

#[test]
fn deep_is_superset_of_non_fun_presets() {
    let deep = preset_plan_for(PresetKind::Deep);
    for kind in PresetKind::all() {
        if *kind == PresetKind::Deep || *kind == PresetKind::Fun {
            continue;
        }
        let plan = preset_plan_for(*kind);
        if plan.assets {
            assert!(deep.assets, "{:?} assets not in deep", kind);
        }
        if plan.deps {
            assert!(deep.deps, "{:?} deps not in deep", kind);
        }
        if plan.todo {
            assert!(deep.todo, "{:?} todo not in deep", kind);
        }
        if plan.dup {
            assert!(deep.dup, "{:?} dup not in deep", kind);
        }
        if plan.imports {
            assert!(deep.imports, "{:?} imports not in deep", kind);
        }
        if plan.git {
            assert!(deep.git, "{:?} git not in deep", kind);
        }
        if plan.archetype {
            assert!(deep.archetype, "{:?} archetype not in deep", kind);
        }
        if plan.topics {
            assert!(deep.topics, "{:?} topics not in deep", kind);
        }
        if plan.entropy {
            assert!(deep.entropy, "{:?} entropy not in deep", kind);
        }
        if plan.license {
            assert!(deep.license, "{:?} license not in deep", kind);
        }
        if plan.complexity {
            assert!(deep.complexity, "{:?} complexity not in deep", kind);
        }
        if plan.api_surface {
            assert!(deep.api_surface, "{:?} api_surface not in deep", kind);
        }
    }
}

// ---------------------------------------------------------------------------
// BDD: preset_plan_for_name
// ---------------------------------------------------------------------------

#[test]
fn plan_for_name_returns_none_for_unknown() {
    assert!(preset_plan_for_name("nonexistent").is_none());
    assert!(preset_plan_for_name("DEEP").is_none());
    assert!(preset_plan_for_name("").is_none());
}

#[test]
fn plan_for_name_agrees_with_plan_for_kind() {
    for kind in PresetKind::all() {
        let by_kind = preset_plan_for(*kind);
        let by_name = preset_plan_for_name(kind.as_str()).expect("should resolve");
        assert_eq!(by_kind, by_name, "Mismatch for {:?}", kind);
    }
}

// ---------------------------------------------------------------------------
// BDD: needs_files()
// ---------------------------------------------------------------------------

#[test]
fn receipt_does_not_need_files() {
    assert!(!preset_plan_for(PresetKind::Receipt).needs_files());
}

#[test]
fn supply_needs_files_because_of_assets_and_deps() {
    assert!(preset_plan_for(PresetKind::Supply).needs_files());
}

#[test]
fn health_needs_files_because_of_todo_and_complexity() {
    assert!(preset_plan_for(PresetKind::Health).needs_files());
}

#[test]
fn topics_does_not_need_files() {
    // Topics only sets `topics: true`, which is not in needs_files() check
    assert!(!preset_plan_for(PresetKind::Topics).needs_files());
}

#[test]
fn fun_does_not_need_files() {
    assert!(!preset_plan_for(PresetKind::Fun).needs_files());
}

#[test]
fn deep_needs_files() {
    assert!(preset_plan_for(PresetKind::Deep).needs_files());
}

#[test]
fn security_needs_files_for_entropy_and_license() {
    let plan = preset_plan_for(PresetKind::Security);
    assert!(plan.entropy);
    assert!(plan.license);
    assert!(plan.needs_files());
}

// ---------------------------------------------------------------------------
// BDD: DisabledFeature warnings
// ---------------------------------------------------------------------------

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
        let w = f.warning();
        assert!(!w.is_empty(), "Warning for {:?} is empty", f);
        assert!(w.len() > 10, "Warning for {:?} is suspiciously short", f);
    }
}

#[test]
fn all_disabled_feature_warnings_are_unique() {
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
    let mut msgs: Vec<&str> = features.iter().map(|f| f.warning()).collect();
    let total = msgs.len();
    msgs.sort();
    msgs.dedup();
    assert_eq!(msgs.len(), total, "Some disabled-feature warnings collide");
}

#[test]
fn disabled_feature_warning_mentions_disabled() {
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
        let w = f.warning();
        assert!(
            w.contains("disabled") || w.contains("skipping"),
            "Warning for {:?} should mention 'disabled' or 'skipping': {}",
            f,
            w
        );
    }
}

// ---------------------------------------------------------------------------
// Determinism: repeated calls yield identical results
// ---------------------------------------------------------------------------

#[test]
fn preset_plan_for_is_deterministic() {
    for kind in PresetKind::all() {
        let a = preset_plan_for(*kind);
        let b = preset_plan_for(*kind);
        assert_eq!(a, b, "Non-deterministic plan for {:?}", kind);
    }
}

#[test]
fn preset_plan_for_name_is_deterministic() {
    for kind in PresetKind::all() {
        let a = preset_plan_for_name(kind.as_str());
        let b = preset_plan_for_name(kind.as_str());
        assert_eq!(a, b, "Non-deterministic name lookup for {:?}", kind);
    }
}

#[test]
fn grid_row_debug_is_deterministic() {
    for row in &PRESET_GRID {
        let a = format!("{:?}", row);
        let b = format!("{:?}", row);
        assert_eq!(a, b);
    }
}

// ---------------------------------------------------------------------------
// Proptest: property-based invariants
// ---------------------------------------------------------------------------

mod properties {
    use proptest::prelude::*;
    use tokmd_analysis_grid::{PresetKind, preset_plan_for, preset_plan_for_name};

    fn arb_preset_kind() -> impl Strategy<Value = PresetKind> {
        prop::sample::select(vec![
            PresetKind::Receipt,
            PresetKind::Health,
            PresetKind::Risk,
            PresetKind::Supply,
            PresetKind::Architecture,
            PresetKind::Topics,
            PresetKind::Security,
            PresetKind::Identity,
            PresetKind::Git,
            PresetKind::Deep,
            PresetKind::Fun,
        ])
    }

    proptest! {
        #[test]
        fn roundtrip_str_is_identity(kind in arb_preset_kind()) {
            let parsed = PresetKind::from_str(kind.as_str()).unwrap();
            prop_assert_eq!(parsed, kind);
        }

        #[test]
        fn plan_for_name_consistent_with_plan_for(kind in arb_preset_kind()) {
            let by_kind = preset_plan_for(kind);
            let by_name = preset_plan_for_name(kind.as_str()).unwrap();
            prop_assert_eq!(by_kind, by_name);
        }

        #[test]
        fn from_str_random_strings_do_not_panic(s in "\\PC{0,50}") {
            let _ = PresetKind::from_str(&s);
        }

        #[test]
        fn from_str_random_strings_never_return_receipt_unless_exact(s in "[a-zA-Z0-9_ ]{1,20}") {
            if let Some(kind) = PresetKind::from_str(&s) {
                prop_assert_eq!(kind.as_str(), s.as_str());
            }
        }

        #[test]
        fn needs_files_stable_across_calls(kind in arb_preset_kind()) {
            let plan = preset_plan_for(kind);
            prop_assert_eq!(plan.needs_files(), plan.needs_files());
        }
    }
}
