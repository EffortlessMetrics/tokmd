use proptest::prelude::*;
use tokmd_analysis_maintainability::compute_maintainability_index;

proptest! {
    #[test]
    fn score_is_non_negative(
        cc in 0.0f64..1000.0,
        loc in 0.01f64..1e6,
        vol in proptest::option::of(0.01f64..1e6),
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol) {
            prop_assert!(mi.score >= 0.0, "score must be >= 0, got {}", mi.score);
        }
    }

    #[test]
    fn score_is_at_most_171(
        cc in 0.0f64..1000.0,
        loc in 0.01f64..1e6,
        vol in proptest::option::of(0.01f64..1e6),
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol) {
            prop_assert!(mi.score <= 171.0, "score must be <= 171, got {}", mi.score);
        }
    }

    #[test]
    fn grade_is_valid(
        cc in 0.0f64..500.0,
        loc in 0.01f64..1e5,
        vol in proptest::option::of(0.01f64..1e5),
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol) {
            prop_assert!(
                mi.grade == "A" || mi.grade == "B" || mi.grade == "C",
                "unexpected grade: {}", mi.grade
            );
        }
    }

    #[test]
    fn grade_a_implies_score_at_least_85(
        cc in 0.0f64..500.0,
        loc in 0.01f64..1e5,
        vol in proptest::option::of(0.01f64..1e5),
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol)
            && mi.grade == "A"
        {
            prop_assert!(mi.score >= 85.0);
        }
    }

    #[test]
    fn grade_b_implies_score_between_65_and_85(
        cc in 0.0f64..500.0,
        loc in 0.01f64..1e5,
        vol in proptest::option::of(0.01f64..1e5),
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol)
            && mi.grade == "B"
        {
            prop_assert!(mi.score >= 65.0 && mi.score < 85.0);
        }
    }

    #[test]
    fn grade_c_implies_score_below_65(
        cc in 0.0f64..500.0,
        loc in 0.01f64..1e5,
        vol in proptest::option::of(0.01f64..1e5),
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, vol)
            && mi.grade == "C"
        {
            prop_assert!(mi.score < 65.0);
        }
    }

    #[test]
    fn zero_or_negative_loc_returns_none(
        cc in 0.0f64..100.0,
        loc in -1e6f64..=0.0,
    ) {
        prop_assert!(compute_maintainability_index(cc, loc, None).is_none());
    }

    #[test]
    fn halstead_volume_decreases_score(
        cc in 0.0f64..100.0,
        loc in 1.0f64..1000.0,
        vol in 1.0f64..1e5,
    ) {
        let simplified = compute_maintainability_index(cc, loc, None).expect("simplified");
        let full = compute_maintainability_index(cc, loc, Some(vol)).expect("full");
        prop_assert!(full.score <= simplified.score,
            "full ({}) should be <= simplified ({})", full.score, simplified.score);
    }

    #[test]
    fn higher_cc_decreases_score(
        cc1 in 0.0f64..500.0,
        cc2 in 0.0f64..500.0,
        loc in 1.0f64..1000.0,
    ) {
        let mi1 = compute_maintainability_index(cc1, loc, None).expect("mi1");
        let mi2 = compute_maintainability_index(cc2, loc, None).expect("mi2");
        if cc1 < cc2 {
            prop_assert!(mi1.score >= mi2.score);
        }
    }

    #[test]
    fn higher_loc_decreases_score(
        cc in 0.0f64..100.0,
        loc1 in 1.0f64..1000.0,
        loc2 in 1.0f64..1000.0,
    ) {
        let mi1 = compute_maintainability_index(cc, loc1, None).expect("mi1");
        let mi2 = compute_maintainability_index(cc, loc2, None).expect("mi2");
        // Due to rounding, only assert strict inequality when LOC are sufficiently different
        if loc1 < loc2 && (loc2 - loc1).abs() > 0.01 {
            prop_assert!(mi1.score >= mi2.score,
                "score({}) = {} should be >= score({}) = {}", loc1, mi1.score, loc2, mi2.score);
        }
    }

    #[test]
    fn avg_loc_is_rounded_to_2_decimals(
        cc in 0.0f64..100.0,
        loc in 0.01f64..1000.0,
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, None) {
            let scaled = mi.avg_loc * 100.0;
            let diff = (scaled - scaled.round()).abs();
            prop_assert!(diff < 1e-8, "avg_loc not rounded to 2 decimals: {}", mi.avg_loc);
        }
    }

    #[test]
    fn score_is_rounded_to_2_decimals(
        cc in 0.0f64..100.0,
        loc in 0.01f64..1000.0,
    ) {
        if let Some(mi) = compute_maintainability_index(cc, loc, None) {
            let scaled = mi.score * 100.0;
            let diff = (scaled - scaled.round()).abs();
            prop_assert!(diff < 1e-8, "score not rounded to 2 decimals: {}", mi.score);
        }
    }

    #[test]
    fn result_is_deterministic(
        cc in 0.0f64..500.0,
        loc in 0.01f64..1e5,
        vol in proptest::option::of(0.01f64..1e5),
    ) {
        let r1 = compute_maintainability_index(cc, loc, vol);
        let r2 = compute_maintainability_index(cc, loc, vol);
        match (r1, r2) {
            (Some(a), Some(b)) => {
                prop_assert_eq!(a.score, b.score);
                prop_assert_eq!(a.grade, b.grade);
            }
            (None, None) => {}
            _ => prop_assert!(false, "determinism violated"),
        }
    }

    #[test]
    fn zero_volume_falls_back_to_simplified(
        cc in 0.0f64..100.0,
        loc in 1.0f64..1000.0,
    ) {
        let with_zero = compute_maintainability_index(cc, loc, Some(0.0));
        let without = compute_maintainability_index(cc, loc, None);
        match (with_zero, without) {
            (Some(a), Some(b)) => {
                prop_assert_eq!(a.score, b.score);
                prop_assert_eq!(a.avg_halstead_volume, None);
            }
            _ => prop_assert!(false, "both should be Some for positive LOC"),
        }
    }

    #[test]
    fn negative_volume_falls_back_to_simplified(
        cc in 0.0f64..100.0,
        loc in 1.0f64..1000.0,
        vol in -1e6f64..0.0,
    ) {
        let with_neg = compute_maintainability_index(cc, loc, Some(vol));
        let without = compute_maintainability_index(cc, loc, None);
        match (with_neg, without) {
            (Some(a), Some(b)) => {
                prop_assert_eq!(a.score, b.score);
            }
            _ => prop_assert!(false, "both should be Some for positive LOC"),
        }
    }
}
