//! Property-based tests for topic-cloud extraction.

use proptest::prelude::*;
use tokmd_analysis_topics::build_topic_clouds;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ── strategies ───────────────────────────────────────────────────────

fn arb_segment() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,11}")
        .unwrap()
        .prop_filter("non-empty", |s| !s.is_empty())
}

fn arb_path() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_segment(), 1..=5).prop_map(|segs| {
        let dir = segs[..segs.len().saturating_sub(1)].join("/");
        let file = segs.last().unwrap().clone();
        if dir.is_empty() {
            format!("{file}.rs")
        } else {
            format!("{dir}/{file}.rs")
        }
    })
}

fn arb_file_row() -> impl Strategy<Value = FileRow> {
    (arb_path(), 1..1000usize, 1..5000usize).prop_map(|(path, code, tokens)| {
        let module = path.split('/').next().unwrap_or("root").to_string();
        FileRow {
            path,
            module,
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code,
            comments: 0,
            blanks: 0,
            lines: code,
            bytes: code * 10,
            tokens,
        }
    })
}

fn arb_export() -> impl Strategy<Value = ExportData> {
    prop::collection::vec(arb_file_row(), 0..=30).prop_map(|rows| ExportData {
        rows,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    })
}

// ── properties ───────────────────────────────────────────────────────

proptest! {
    #[test]
    fn overall_len_at_most_top_k(export in arb_export()) {
        let clouds = build_topic_clouds(&export);
        prop_assert!(clouds.overall.len() <= 8,
            "overall length {} exceeds TOP_K=8", clouds.overall.len());
    }

    #[test]
    fn per_module_len_at_most_top_k(export in arb_export()) {
        let clouds = build_topic_clouds(&export);
        for (module, terms) in &clouds.per_module {
            prop_assert!(terms.len() <= 8,
                "module '{}' has {} terms, exceeds TOP_K=8", module, terms.len());
        }
    }

    #[test]
    fn all_scores_are_non_negative(export in arb_export()) {
        let clouds = build_topic_clouds(&export);
        for term in &clouds.overall {
            prop_assert!(term.score >= 0.0,
                "overall term '{}' has negative score {}", term.term, term.score);
        }
        for terms in clouds.per_module.values() {
            for term in terms {
                prop_assert!(term.score >= 0.0,
                    "term '{}' has negative score {}", term.term, term.score);
            }
        }
    }

    #[test]
    fn all_scores_are_finite(export in arb_export()) {
        let clouds = build_topic_clouds(&export);
        for term in &clouds.overall {
            prop_assert!(term.score.is_finite(),
                "overall term '{}' has non-finite score {}", term.term, term.score);
        }
        for terms in clouds.per_module.values() {
            for term in terms {
                prop_assert!(term.score.is_finite(),
                    "term '{}' has non-finite score {}", term.term, term.score);
            }
        }
    }

    #[test]
    fn all_terms_are_lowercase(export in arb_export()) {
        let clouds = build_topic_clouds(&export);
        for term in &clouds.overall {
            prop_assert_eq!(&term.term, &term.term.to_lowercase(),
                "term '{}' should be lowercase", term.term);
        }
    }

    #[test]
    fn overall_sorted_descending_by_score(export in arb_export()) {
        let clouds = build_topic_clouds(&export);
        for window in clouds.overall.windows(2) {
            prop_assert!(window[0].score >= window[1].score,
                "overall not sorted: {} ({}) < {} ({})",
                window[0].term, window[0].score,
                window[1].term, window[1].score);
        }
    }

    #[test]
    fn per_module_sorted_descending_by_score(export in arb_export()) {
        let clouds = build_topic_clouds(&export);
        for (module, terms) in &clouds.per_module {
            for window in terms.windows(2) {
                prop_assert!(window[0].score >= window[1].score,
                    "module '{}' not sorted: {} ({}) < {} ({})",
                    module,
                    window[0].term, window[0].score,
                    window[1].term, window[1].score);
            }
        }
    }

    #[test]
    fn empty_rows_produce_empty_clouds(
        roots in prop::collection::vec(arb_segment(), 0..3)
    ) {
        let export = ExportData {
            rows: vec![],
            module_roots: roots,
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let clouds = build_topic_clouds(&export);
        prop_assert!(clouds.overall.is_empty());
        prop_assert!(clouds.per_module.is_empty());
    }

    #[test]
    fn child_rows_are_ignored(
        rows in prop::collection::vec(arb_file_row(), 1..10)
    ) {
        let child_rows: Vec<FileRow> = rows.into_iter().map(|mut r| {
            r.kind = FileKind::Child;
            r
        }).collect();
        let export = ExportData {
            rows: child_rows,
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        let clouds = build_topic_clouds(&export);
        prop_assert!(clouds.overall.is_empty());
        prop_assert!(clouds.per_module.is_empty());
    }

    #[test]
    fn tf_is_positive_for_every_term(export in arb_export()) {
        let clouds = build_topic_clouds(&export);
        for term in &clouds.overall {
            prop_assert!(term.tf > 0, "tf should be > 0 for term '{}'", term.term);
        }
        for terms in clouds.per_module.values() {
            for term in terms {
                prop_assert!(term.tf > 0, "tf should be > 0 for term '{}'", term.term);
            }
        }
    }

    #[test]
    fn df_is_positive_and_bounded_by_parent_row_count(export in arb_export()) {
        let clouds = build_topic_clouds(&export);
        let parent_count = export.rows.iter()
            .filter(|r| r.kind == FileKind::Parent)
            .count() as u32;
        for term in &clouds.overall {
            prop_assert!(term.df > 0, "df should be > 0 for '{}'", term.term);
            prop_assert!(term.df <= parent_count,
                "df {} exceeds parent row count {} for '{}'",
                term.df, parent_count, term.term);
        }
    }

    #[test]
    fn deterministic_across_runs(export in arb_export()) {
        let a = build_topic_clouds(&export);
        let b = build_topic_clouds(&export);
        prop_assert_eq!(a.overall.len(), b.overall.len());
        for (ta, tb) in a.overall.iter().zip(b.overall.iter()) {
            prop_assert_eq!(&ta.term, &tb.term);
            prop_assert_eq!(ta.tf, tb.tf);
            prop_assert_eq!(ta.df, tb.df);
            prop_assert!((ta.score - tb.score).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn overall_terms_are_non_empty(export in arb_export()) {
        let clouds = build_topic_clouds(&export);
        for term in &clouds.overall {
            prop_assert!(!term.term.is_empty(), "empty term in overall");
        }
        for terms in clouds.per_module.values() {
            for term in terms {
                prop_assert!(!term.term.is_empty(), "empty term in per_module");
            }
        }
    }

    #[test]
    fn per_module_keys_correspond_to_input_modules(export in arb_export()) {
        let clouds = build_topic_clouds(&export);
        let input_modules: std::collections::BTreeSet<&str> = export.rows.iter()
            .filter(|r| r.kind == FileKind::Parent)
            .map(|r| r.module.as_str())
            .collect();
        for key in clouds.per_module.keys() {
            prop_assert!(
                input_modules.contains(key.as_str()),
                "per_module key '{}' not found in input modules {:?}",
                key,
                input_modules
            );
        }
    }

    #[test]
    fn no_stopword_extensions_in_terms(export in arb_export()) {
        let extensions = ["rs", "js", "ts", "py", "go", "java", "toml", "json", "yaml"];
        let clouds = build_topic_clouds(&export);
        for term in &clouds.overall {
            for ext in &extensions {
                prop_assert_ne!(
                    &term.term, ext,
                    "file extension '{}' should be filtered as stopword",
                    ext
                );
            }
        }
    }
}
