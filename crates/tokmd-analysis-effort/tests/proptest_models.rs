use proptest::prelude::*;
use tokmd_analysis_effort::cocomo2::{cocomo2_baseline, cocomo2_effort_pm};
use tokmd_analysis_effort::cocomo81::{cocomo81_baseline, cocomo81_effort_pm};
use tokmd_analysis_types::{EffortConfidence, EffortConfidenceLevel, EffortResults};
use tokmd_analysis_effort::uncertainty::apply_uncertainty;

proptest! {
    #[test]
    fn cocomo81_non_negative_kloc(kloc in 0.0_f64..10_000.0) {
        let (effort, schedule, staff, _) = cocomo81_effort_pm(kloc);
        if kloc == 0.0 {
            prop_assert_eq!(effort, 0.0);
            prop_assert_eq!(schedule, 0.0);
            prop_assert_eq!(staff, 0.0);
        } else {
            prop_assert!(effort > 0.0);
            prop_assert!(schedule > 0.0);
            prop_assert!(staff > 0.0);

            // Staff is effort / schedule
            let expected_staff = effort / schedule;
            prop_assert!((staff - expected_staff).abs() < 1e-6);
        }
    }

    #[test]
    fn cocomo81_negative_kloc_is_zero(kloc in -10_000.0_f64..0.0) {
        let (effort, schedule, staff, _) = cocomo81_effort_pm(kloc);
        prop_assert_eq!(effort, 0.0);
        prop_assert_eq!(schedule, 0.0);
        prop_assert_eq!(staff, 0.0);
    }

    #[test]
    fn cocomo2_non_negative_kloc(kloc in 0.0_f64..10_000.0) {
        let (effort, schedule, staff, _) = cocomo2_effort_pm(kloc);
        if kloc == 0.0 {
            prop_assert_eq!(effort, 0.0);
            prop_assert_eq!(schedule, 0.0);
            prop_assert_eq!(staff, 0.0);
        } else {
            prop_assert!(effort > 0.0);
            prop_assert!(schedule > 0.0);
            prop_assert!(staff > 0.0);

            let expected_staff = effort / schedule;
            prop_assert!((staff - expected_staff).abs() < 1e-6);
        }
    }

    #[test]
    fn cocomo2_negative_kloc_is_zero(kloc in -10_000.0_f64..0.0) {
        let (effort, schedule, staff, _) = cocomo2_effort_pm(kloc);
        prop_assert_eq!(effort, 0.0);
        prop_assert_eq!(schedule, 0.0);
        prop_assert_eq!(staff, 0.0);
    }

    #[test]
    fn baseline_results_ordering(kloc in 0.1_f64..10_000.0) {
        let res81 = cocomo81_baseline(kloc);
        prop_assert!(res81.effort_pm_low <= res81.effort_pm_p50);
        prop_assert!(res81.effort_pm_p50 <= res81.effort_pm_p80);
        prop_assert!(res81.schedule_months_low <= res81.schedule_months_p50);
        prop_assert!(res81.schedule_months_p50 <= res81.schedule_months_p80);

        let res2 = cocomo2_baseline(kloc);
        prop_assert!(res2.effort_pm_low <= res2.effort_pm_p50);
        prop_assert!(res2.effort_pm_p50 <= res2.effort_pm_p80);
        prop_assert!(res2.schedule_months_low <= res2.schedule_months_p50);
        prop_assert!(res2.schedule_months_p50 <= res2.schedule_months_p80);
    }

    #[test]
    fn uncertainty_maintains_invariants(
        effort in 1.0_f64..10_000.0,
        schedule in 1.0_f64..100.0,
        basis_conf in 0.0_f64..1.0,
        data_cov in 0.0_f64..1.0,
    ) {
        let base = EffortResults {
            effort_pm_p50: effort,
            schedule_months_p50: schedule,
            staff_p50: effort / schedule,
            effort_pm_low: effort,
            effort_pm_p80: effort,
            schedule_months_low: schedule,
            schedule_months_p80: schedule,
            staff_low: effort / schedule,
            staff_p80: effort / schedule,
        };

        for level in [EffortConfidenceLevel::Low, EffortConfidenceLevel::Medium, EffortConfidenceLevel::High] {
            let conf = EffortConfidence {
                level,
                reasons: vec![],
                data_coverage_pct: Some(data_cov),
            };

            let res = apply_uncertainty(base.clone(), &conf, basis_conf, &[]);

            prop_assert!(res.effort_pm_low <= res.effort_pm_p50);
            prop_assert!(res.effort_pm_p50 <= res.effort_pm_p80);
            prop_assert!(res.schedule_months_low <= res.schedule_months_p50);
            prop_assert!(res.schedule_months_p50 <= res.schedule_months_p80);

            if res.schedule_months_p80 > 0.0 {
                prop_assert!((res.staff_low - (res.effort_pm_low / res.schedule_months_p80)).abs() < 1e-6);
            }
            if res.schedule_months_low > 0.0 {
                prop_assert!((res.staff_p80 - (res.effort_pm_p80 / res.schedule_months_low)).abs() < 1e-6);
            }
        }
    }
}
