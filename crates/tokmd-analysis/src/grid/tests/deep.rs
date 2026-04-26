//! Deep tests for analysis grid module.
//!
//! Covers areas not fully exercised by existing tests:
//! - Preset name completeness against documented names
//! - Preset flag exclusivity (only specific presets enable specific flags)
//! - Deep preset superset invariant (including cfg-gated fields)
//! - Grid row uniqueness (no duplicate enricher configurations)
//! - PresetPlan needs_files exhaustive verification
//! - DisabledFeature Debug trait
//! - PresetKind ordering stability
//! - Cross-preset flag analysis (which presets share which flags)

use crate::grid::{
    DisabledFeature, PRESET_GRID, PRESET_KINDS, PresetKind, PresetPlan, preset_plan_for,
    preset_plan_for_name,
};

// =========================================================================
// 1. All presets are defined — preset names match documentation
// =========================================================================

mod preset_names {
    use super::*;

    /// Documented preset names from CLAUDE.md / docs.
    const DOCUMENTED_PRESETS: &[&str] = &[
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

    #[test]
    fn all_documented_presets_exist() {
        for name in DOCUMENTED_PRESETS {
            assert!(
                PresetKind::from_str(name).is_some(),
                "Documented preset '{}' not found in PresetKind::from_str",
                name
            );
        }
    }

    #[test]
    fn no_undocumented_presets_exist() {
        let kinds: Vec<&str> = PRESET_KINDS.iter().map(|k| k.as_str()).collect();
        for name in &kinds {
            assert!(
                DOCUMENTED_PRESETS.contains(name),
                "Preset '{}' exists in code but is not in the documented list",
                name
            );
        }
    }

    #[test]
    fn preset_count_matches_documentation() {
        assert_eq!(
            PRESET_KINDS.len(),
            DOCUMENTED_PRESETS.len(),
            "Number of presets in code ({}) doesn't match documentation ({})",
            PRESET_KINDS.len(),
            DOCUMENTED_PRESETS.len()
        );
    }

    #[test]
    fn as_str_values_are_all_lowercase_alphabetic() {
        for kind in PresetKind::all() {
            let name = kind.as_str();
            assert!(
                name.chars().all(|c| c.is_ascii_lowercase()),
                "Preset name '{}' contains non-lowercase-alpha characters",
                name
            );
            assert!(!name.is_empty(), "Preset name should not be empty");
        }
    }

    #[test]
    fn from_str_rejects_common_invalid_inputs() {
        let invalid = [
            "",
            " ",
            "RECEIPT",
            "Receipt",
            "DEEP",
            "Deep",
            "receipt ",
            " receipt",
            "receipt\n",
            "all",
            "none",
            "default",
            "full",
            "minimal",
            "custom",
        ];
        for input in &invalid {
            assert!(
                PresetKind::from_str(input).is_none(),
                "from_str({:?}) should return None",
                input
            );
        }
    }
}

// =========================================================================
// 2. Feature flags for each preset are consistent
// =========================================================================

mod preset_flags {
    use super::*;

    /// Count the number of base flags enabled in a PresetPlan.
    fn count_base_flags(plan: &PresetPlan) -> usize {
        let flags: Vec<bool> = vec![
            plan.assets,
            plan.deps,
            plan.todo,
            plan.dup,
            plan.imports,
            plan.git,
            plan.fun,
            plan.archetype,
            plan.topics,
            plan.entropy,
            plan.license,
            plan.complexity,
            plan.api_surface,
        ];
        flags.iter().filter(|&&f| f).count()
    }

    #[test]
    fn receipt_has_four_base_flags() {
        let plan = preset_plan_for(PresetKind::Receipt);
        assert_eq!(count_base_flags(&plan), 4);
        assert!(plan.dup);
        assert!(plan.git);
        assert!(plan.complexity);
        assert!(plan.api_surface);
    }

    #[test]
    fn estimate_has_four_base_flags() {
        let plan = preset_plan_for(PresetKind::Estimate);
        assert_eq!(count_base_flags(&plan), 4);
        assert!(plan.dup);
        assert!(plan.git);
        assert!(plan.complexity);
        assert!(plan.api_surface);
    }

    #[test]
    fn fun_has_exactly_one_base_flag() {
        let plan = preset_plan_for(PresetKind::Fun);
        assert_eq!(count_base_flags(&plan), 1);
        assert!(plan.fun);
    }

    #[test]
    fn health_has_exactly_two_base_flags() {
        let plan = preset_plan_for(PresetKind::Health);
        assert_eq!(count_base_flags(&plan), 2);
        assert!(plan.todo);
        assert!(plan.complexity);
    }

    #[test]
    fn risk_has_exactly_two_base_flags() {
        let plan = preset_plan_for(PresetKind::Risk);
        assert_eq!(count_base_flags(&plan), 2);
        assert!(plan.git);
        assert!(plan.complexity);
    }

    #[test]
    fn supply_has_exactly_two_base_flags() {
        let plan = preset_plan_for(PresetKind::Supply);
        assert_eq!(count_base_flags(&plan), 2);
        assert!(plan.assets);
        assert!(plan.deps);
    }

    #[test]
    fn architecture_has_exactly_two_base_flags() {
        let plan = preset_plan_for(PresetKind::Architecture);
        assert_eq!(count_base_flags(&plan), 2);
        assert!(plan.imports);
        assert!(plan.api_surface);
    }

    #[test]
    fn topics_has_exactly_one_base_flag() {
        let plan = preset_plan_for(PresetKind::Topics);
        assert_eq!(count_base_flags(&plan), 1);
        assert!(plan.topics);
    }

    #[test]
    fn security_has_exactly_two_base_flags() {
        let plan = preset_plan_for(PresetKind::Security);
        assert_eq!(count_base_flags(&plan), 2);
        assert!(plan.entropy);
        assert!(plan.license);
    }

    #[test]
    fn identity_has_exactly_two_base_flags() {
        let plan = preset_plan_for(PresetKind::Identity);
        assert_eq!(count_base_flags(&plan), 2);
        assert!(plan.git);
        assert!(plan.archetype);
    }

    #[test]
    fn git_has_exactly_one_base_flag() {
        let plan = preset_plan_for(PresetKind::Git);
        assert_eq!(count_base_flags(&plan), 1);
        assert!(plan.git);
    }

    #[test]
    fn deep_has_all_base_flags_except_fun() {
        let plan = preset_plan_for(PresetKind::Deep);
        // All 12 non-fun flags should be true
        assert_eq!(count_base_flags(&plan), 12);
        assert!(!plan.fun);
    }

    #[test]
    fn every_preset_has_at_least_one_flag() {
        for kind in PresetKind::all() {
            let plan = preset_plan_for(*kind);
            assert!(
                count_base_flags(&plan) > 0,
                "Preset {:?} has no flags enabled",
                kind
            );
        }
    }
}

// =========================================================================
// 3. Serialization roundtrip (PresetKind name roundtrip)
// =========================================================================

mod roundtrips {
    use super::*;

    #[test]
    fn from_str_as_str_roundtrip_all_presets() {
        for kind in PresetKind::all() {
            let name = kind.as_str();
            let parsed = PresetKind::from_str(name);
            assert_eq!(
                parsed,
                Some(*kind),
                "Roundtrip failed for {:?} -> {:?} -> {:?}",
                kind,
                name,
                parsed
            );
        }
    }

    #[test]
    fn preset_plan_for_name_matches_preset_plan_for_kind() {
        for kind in PresetKind::all() {
            let by_kind = preset_plan_for(*kind);
            let by_name = preset_plan_for_name(kind.as_str()).unwrap();
            assert_eq!(
                by_kind, by_name,
                "Plan mismatch between by-kind and by-name for {:?}",
                kind
            );
        }
    }
}

// =========================================================================
// 4. Grid determines which enrichers to run for a given preset
// =========================================================================

mod enricher_selection {
    use super::*;

    #[test]
    fn preset_grid_row_preset_matches_plan() {
        for row in &PRESET_GRID {
            let plan = preset_plan_for(row.preset);
            assert_eq!(
                plan, row.plan,
                "PRESET_GRID row for {:?} doesn't match preset_plan_for",
                row.preset
            );
        }
    }

    #[test]
    fn only_supply_and_deep_enable_assets() {
        let with_assets: Vec<PresetKind> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.assets)
            .map(|r| r.preset)
            .collect();
        assert!(with_assets.contains(&PresetKind::Supply));
        assert!(with_assets.contains(&PresetKind::Deep));
        assert_eq!(
            with_assets.len(),
            2,
            "Only supply and deep should have assets"
        );
    }

    #[test]
    fn only_supply_and_deep_enable_deps() {
        let with_deps: Vec<PresetKind> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.deps)
            .map(|r| r.preset)
            .collect();
        assert!(with_deps.contains(&PresetKind::Supply));
        assert!(with_deps.contains(&PresetKind::Deep));
        assert_eq!(with_deps.len(), 2);
    }

    #[test]
    fn only_health_and_deep_enable_todo() {
        let with_todo: Vec<PresetKind> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.todo)
            .map(|r| r.preset)
            .collect();
        assert!(with_todo.contains(&PresetKind::Health));
        assert!(with_todo.contains(&PresetKind::Deep));
        assert_eq!(with_todo.len(), 2);
    }

    #[test]
    fn receipt_estimate_and_deep_enable_dup() {
        let with_dup: Vec<PresetKind> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.dup)
            .map(|r| r.preset)
            .collect();
        assert!(with_dup.contains(&PresetKind::Receipt));
        assert!(with_dup.contains(&PresetKind::Estimate));
        assert!(with_dup.contains(&PresetKind::Deep));
        assert_eq!(with_dup.len(), 3);
    }

    #[test]
    fn only_architecture_and_deep_enable_imports() {
        let with_imports: Vec<PresetKind> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.imports)
            .map(|r| r.preset)
            .collect();
        assert!(with_imports.contains(&PresetKind::Architecture));
        assert!(with_imports.contains(&PresetKind::Deep));
        assert_eq!(with_imports.len(), 2);
    }

    #[test]
    fn receipt_estimate_risk_identity_git_deep_enable_git_flag() {
        let with_git: Vec<PresetKind> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.git)
            .map(|r| r.preset)
            .collect();
        assert!(with_git.contains(&PresetKind::Receipt));
        assert!(with_git.contains(&PresetKind::Estimate));
        assert!(with_git.contains(&PresetKind::Risk));
        assert!(with_git.contains(&PresetKind::Identity));
        assert!(with_git.contains(&PresetKind::Git));
        assert!(with_git.contains(&PresetKind::Deep));
        assert_eq!(with_git.len(), 6);
    }

    #[test]
    fn only_fun_enables_fun_flag() {
        let with_fun: Vec<PresetKind> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.fun)
            .map(|r| r.preset)
            .collect();
        assert_eq!(with_fun, vec![PresetKind::Fun]);
    }

    #[test]
    fn only_identity_and_deep_enable_archetype() {
        let with_archetype: Vec<PresetKind> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.archetype)
            .map(|r| r.preset)
            .collect();
        assert!(with_archetype.contains(&PresetKind::Identity));
        assert!(with_archetype.contains(&PresetKind::Deep));
        assert_eq!(with_archetype.len(), 2);
    }

    #[test]
    fn only_topics_and_deep_enable_topics_flag() {
        let with_topics: Vec<PresetKind> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.topics)
            .map(|r| r.preset)
            .collect();
        assert!(with_topics.contains(&PresetKind::Topics));
        assert!(with_topics.contains(&PresetKind::Deep));
        assert_eq!(with_topics.len(), 2);
    }

    #[test]
    fn only_security_and_deep_enable_entropy() {
        let with_entropy: Vec<PresetKind> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.entropy)
            .map(|r| r.preset)
            .collect();
        assert!(with_entropy.contains(&PresetKind::Security));
        assert!(with_entropy.contains(&PresetKind::Deep));
        assert_eq!(with_entropy.len(), 2);
    }

    #[test]
    fn only_security_and_deep_enable_license() {
        let with_license: Vec<PresetKind> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.license)
            .map(|r| r.preset)
            .collect();
        assert!(with_license.contains(&PresetKind::Security));
        assert!(with_license.contains(&PresetKind::Deep));
        assert_eq!(with_license.len(), 2);
    }

    #[test]
    fn receipt_estimate_health_risk_deep_enable_complexity() {
        let with_complexity: Vec<PresetKind> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.complexity)
            .map(|r| r.preset)
            .collect();
        assert!(with_complexity.contains(&PresetKind::Receipt));
        assert!(with_complexity.contains(&PresetKind::Estimate));
        assert!(with_complexity.contains(&PresetKind::Health));
        assert!(with_complexity.contains(&PresetKind::Risk));
        assert!(with_complexity.contains(&PresetKind::Deep));
        assert_eq!(with_complexity.len(), 5);
    }

    #[test]
    fn receipt_estimate_architecture_deep_enable_api_surface() {
        let with_api: Vec<PresetKind> = PRESET_GRID
            .iter()
            .filter(|r| r.plan.api_surface)
            .map(|r| r.preset)
            .collect();
        assert!(with_api.contains(&PresetKind::Receipt));
        assert!(with_api.contains(&PresetKind::Estimate));
        assert!(with_api.contains(&PresetKind::Architecture));
        assert!(with_api.contains(&PresetKind::Deep));
        assert_eq!(with_api.len(), 4);
    }
}

// =========================================================================
// 5. Deep preset includes all features (superset)
// =========================================================================

mod deep_superset {
    use super::*;

    #[test]
    fn deep_is_superset_of_all_non_fun_presets() {
        let deep = preset_plan_for(PresetKind::Deep);
        for kind in PresetKind::all() {
            if *kind == PresetKind::Fun || *kind == PresetKind::Deep {
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

    #[test]
    fn deep_does_not_enable_fun() {
        assert!(!preset_plan_for(PresetKind::Deep).fun);
    }

    #[test]
    fn deep_enables_all_twelve_non_fun_flags() {
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
    }
}

// =========================================================================
// 6. Receipt preset is minimal
// =========================================================================

mod receipt_enrichers {
    use super::*;

    #[test]
    fn receipt_enables_core_enrichers() {
        let plan = preset_plan_for(PresetKind::Receipt);
        assert!(plan.dup);
        assert!(plan.git);
        assert!(plan.complexity);
        assert!(plan.api_surface);
        // But not these:
        assert!(!plan.assets);
        assert!(!plan.deps);
        assert!(!plan.todo);
        assert!(!plan.imports);
        assert!(!plan.fun);
        assert!(!plan.archetype);
        assert!(!plan.topics);
        assert!(!plan.entropy);
        assert!(!plan.license);
    }

    #[test]
    fn receipt_needs_files() {
        assert!(preset_plan_for(PresetKind::Receipt).needs_files());
    }
}

// =========================================================================
// 7. No duplicate enricher entries in grid
// =========================================================================

mod no_duplicates {
    use super::*;

    #[test]
    fn no_duplicate_preset_kinds_in_grid() {
        let mut seen: Vec<PresetKind> = Vec::new();
        for row in &PRESET_GRID {
            assert!(
                !seen.contains(&row.preset),
                "Duplicate preset {:?} in PRESET_GRID",
                row.preset
            );
            seen.push(row.preset);
        }
    }

    #[test]
    fn no_duplicate_preset_kinds_in_kinds_array() {
        let mut seen: Vec<PresetKind> = Vec::new();
        for kind in &PRESET_KINDS {
            assert!(
                !seen.contains(kind),
                "Duplicate preset {:?} in PRESET_KINDS",
                kind
            );
            seen.push(*kind);
        }
    }

    #[test]
    fn grid_and_kinds_same_length() {
        assert_eq!(PRESET_GRID.len(), PRESET_KINDS.len());
    }

    #[test]
    fn grid_order_matches_kinds_order() {
        for (row, kind) in PRESET_GRID.iter().zip(PRESET_KINDS.iter()) {
            assert_eq!(
                row.preset, *kind,
                "Grid order doesn't match PRESET_KINDS order"
            );
        }
    }
}

// =========================================================================
// 8. needs_files correctness
// =========================================================================

mod needs_files_deep {
    use super::*;

    #[test]
    fn needs_files_matches_any_file_dependent_flag() {
        for row in &PRESET_GRID {
            let plan = &row.plan;
            let expected = plan.assets
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
                expected,
                "needs_files mismatch for {:?}: got {}, expected {}",
                row.preset,
                plan.needs_files(),
                expected
            );
        }
    }

    #[test]
    fn git_only_presets_do_not_need_files() {
        // Git and Identity presets that only enable git/archetype shouldn't need files
        // (archetype is not in needs_files, git is not in needs_files)
        let git_plan = preset_plan_for(PresetKind::Git);
        assert!(!git_plan.needs_files(), "git preset should not need files");
    }

    #[test]
    fn fun_does_not_need_files() {
        assert!(!preset_plan_for(PresetKind::Fun).needs_files());
    }

    #[test]
    fn topics_does_not_need_files() {
        // topics flag alone is not in the needs_files check
        let plan = preset_plan_for(PresetKind::Topics);
        assert!(!plan.needs_files());
    }

    #[test]
    fn identity_does_not_need_files() {
        // identity = git + archetype, neither is in needs_files
        let plan = preset_plan_for(PresetKind::Identity);
        assert!(!plan.needs_files());
    }
}

// =========================================================================
// 9. DisabledFeature exhaustive coverage
// =========================================================================

mod disabled_features_deep {
    use super::*;

    const ALL_FEATURES: [DisabledFeature; 13] = [
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

    #[test]
    fn all_warnings_are_non_empty_and_ascii() {
        for feat in &ALL_FEATURES {
            let msg = feat.warning();
            assert!(!msg.is_empty(), "{:?} has empty warning", feat);
            assert!(msg.is_ascii(), "{:?} warning is not ASCII: {}", feat, msg);
        }
    }

    #[test]
    fn all_warnings_are_unique() {
        let messages: Vec<&str> = ALL_FEATURES.iter().map(|f| f.warning()).collect();
        for (i, a) in messages.iter().enumerate() {
            for (j, b) in messages.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "Duplicate warning between variants {} and {}", i, j);
                }
            }
        }
    }

    #[test]
    fn debug_output_is_non_empty_for_all_variants() {
        for feat in &ALL_FEATURES {
            let debug = format!("{:?}", feat);
            assert!(!debug.is_empty(), "{:?} has empty Debug output", feat);
        }
    }

    #[test]
    fn clone_and_copy_work_for_disabled_feature() {
        for feat in &ALL_FEATURES {
            let cloned = *feat;
            assert_eq!(*feat, cloned);
        }
    }

    #[test]
    fn disabled_feature_equality_is_reflexive() {
        for feat in &ALL_FEATURES {
            assert_eq!(*feat, *feat);
        }
    }

    #[test]
    fn disabled_feature_inequality_across_variants() {
        for (i, a) in ALL_FEATURES.iter().enumerate() {
            for (j, b) in ALL_FEATURES.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }
}

// =========================================================================
// 10. Cross-preset flag analysis
// =========================================================================

mod cross_preset {
    use super::*;

    #[test]
    fn no_preset_enables_both_fun_and_any_other_flag() {
        for row in &PRESET_GRID {
            if row.plan.fun {
                assert!(
                    !row.plan.assets
                        && !row.plan.deps
                        && !row.plan.todo
                        && !row.plan.dup
                        && !row.plan.imports
                        && !row.plan.git
                        && !row.plan.archetype
                        && !row.plan.topics
                        && !row.plan.entropy
                        && !row.plan.license
                        && !row.plan.complexity
                        && !row.plan.api_surface,
                    "{:?} has fun=true along with other flags",
                    row.preset
                );
            }
        }
    }

    #[test]
    fn every_base_flag_is_enabled_by_at_least_one_preset() {
        let all_plans: Vec<&PresetPlan> = PRESET_GRID.iter().map(|r| &r.plan).collect();

        assert!(
            all_plans.iter().any(|p| p.assets),
            "no preset enables assets"
        );
        assert!(all_plans.iter().any(|p| p.deps), "no preset enables deps");
        assert!(all_plans.iter().any(|p| p.todo), "no preset enables todo");
        assert!(all_plans.iter().any(|p| p.dup), "no preset enables dup");
        assert!(
            all_plans.iter().any(|p| p.imports),
            "no preset enables imports"
        );
        assert!(all_plans.iter().any(|p| p.git), "no preset enables git");
        assert!(all_plans.iter().any(|p| p.fun), "no preset enables fun");
        assert!(
            all_plans.iter().any(|p| p.archetype),
            "no preset enables archetype"
        );
        assert!(
            all_plans.iter().any(|p| p.topics),
            "no preset enables topics"
        );
        assert!(
            all_plans.iter().any(|p| p.entropy),
            "no preset enables entropy"
        );
        assert!(
            all_plans.iter().any(|p| p.license),
            "no preset enables license"
        );
        assert!(
            all_plans.iter().any(|p| p.complexity),
            "no preset enables complexity"
        );
        assert!(
            all_plans.iter().any(|p| p.api_surface),
            "no preset enables api_surface"
        );
    }

    #[test]
    fn presets_with_complexity_also_need_files() {
        for row in &PRESET_GRID {
            if row.plan.complexity {
                assert!(
                    row.plan.needs_files(),
                    "{:?} has complexity=true but needs_files()=false",
                    row.preset
                );
            }
        }
    }

    #[test]
    fn presets_with_license_also_need_files() {
        for row in &PRESET_GRID {
            if row.plan.license {
                assert!(
                    row.plan.needs_files(),
                    "{:?} has license=true but needs_files()=false",
                    row.preset
                );
            }
        }
    }
}

// =========================================================================
// 11. PresetKind trait implementations
// =========================================================================

mod traits {
    use super::*;

    #[test]
    fn preset_kind_debug_output_matches_variant_name() {
        let cases = [
            (PresetKind::Receipt, "Receipt"),
            (PresetKind::Estimate, "Estimate"),
            (PresetKind::Health, "Health"),
            (PresetKind::Risk, "Risk"),
            (PresetKind::Supply, "Supply"),
            (PresetKind::Architecture, "Architecture"),
            (PresetKind::Topics, "Topics"),
            (PresetKind::Security, "Security"),
            (PresetKind::Identity, "Identity"),
            (PresetKind::Git, "Git"),
            (PresetKind::Deep, "Deep"),
            (PresetKind::Fun, "Fun"),
        ];
        for (kind, expected) in &cases {
            assert_eq!(format!("{:?}", kind), *expected);
        }
    }

    #[test]
    fn preset_plan_debug_is_deterministic() {
        let plan = preset_plan_for(PresetKind::Deep);
        let debug1 = format!("{:?}", plan);
        let debug2 = format!("{:?}", plan);
        assert_eq!(debug1, debug2);
    }

    #[test]
    fn preset_grid_row_debug_contains_preset_name() {
        for row in &PRESET_GRID {
            let debug = format!("{:?}", row);
            let kind_debug = format!("{:?}", row.preset);
            assert!(
                debug.contains(&kind_debug),
                "Debug of grid row should contain preset name {:?}, got: {}",
                row.preset,
                debug
            );
        }
    }

    #[test]
    fn preset_kind_copy_semantics() {
        let a = PresetKind::Deep;
        let b = a; // Copy
        assert_eq!(a, b);
        // `a` is still usable (not moved)
        assert_eq!(a.as_str(), "deep");
    }

    #[test]
    fn preset_plan_copy_semantics() {
        let plan = preset_plan_for(PresetKind::Risk);
        let plan2 = plan; // Copy
        assert_eq!(plan, plan2);
        // `plan` is still usable
        assert!(plan.git);
    }
}

// =========================================================================
// 12. Lookup stability
// =========================================================================

mod stability {
    use super::*;

    #[test]
    fn preset_plan_for_is_deterministic() {
        for kind in PresetKind::all() {
            let p1 = preset_plan_for(*kind);
            let p2 = preset_plan_for(*kind);
            let p3 = preset_plan_for(*kind);
            assert_eq!(p1, p2);
            assert_eq!(p2, p3);
        }
    }

    #[test]
    fn preset_plan_for_name_is_deterministic() {
        for kind in PresetKind::all() {
            let p1 = preset_plan_for_name(kind.as_str());
            let p2 = preset_plan_for_name(kind.as_str());
            assert_eq!(p1, p2);
        }
    }

    #[test]
    fn preset_kinds_const_is_stable() {
        let all = PresetKind::all();
        assert_eq!(all.len(), PRESET_KINDS.len());
        for (a, b) in all.iter().zip(PRESET_KINDS.iter()) {
            assert_eq!(a, b);
        }
    }
}
