//! Property-based tests for archetype detection.

use proptest::prelude::*;
use tokmd_analysis_archetype::detect_archetype;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Known archetype kinds returned by `detect_archetype`.
const KNOWN_KINDS: &[&str] = &[
    "Rust workspace",
    "Rust workspace (CLI)",
    "Next.js app",
    "Containerized service",
    "Infrastructure as code",
    "Python package",
    "Node package",
];

/// Generate a random file path segment.
fn path_segment() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,11}").unwrap()
}

/// Generate a plausible file path (1-3 segments with an extension).
fn file_path() -> impl Strategy<Value = String> {
    let extensions = prop_oneof![
        Just(".rs"),
        Just(".ts"),
        Just(".py"),
        Just(".js"),
        Just(".toml"),
        Just(".json"),
        Just(".tf"),
        Just(".yaml"),
    ];
    (prop::collection::vec(path_segment(), 1..=3), extensions)
        .prop_map(|(segs, ext)| format!("{}{}", segs.join("/"), ext))
}

/// Generate a set of file paths, optionally including known marker files.
fn file_paths() -> impl Strategy<Value = Vec<String>> {
    let markers = prop::collection::vec(
        prop_oneof![
            Just("Cargo.toml".to_string()),
            Just("package.json".to_string()),
            Just("Dockerfile".to_string()),
            Just("pyproject.toml".to_string()),
            Just("next.config.js".to_string()),
            Just("next.config.mjs".to_string()),
            Just("next.config.ts".to_string()),
            Just("main.tf".to_string()),
            Just("src/main.rs".to_string()),
            Just("crates/x/src/lib.rs".to_string()),
            Just("packages/y/src/lib.rs".to_string()),
            Just("k8s/deploy.yaml".to_string()),
            Just("kubernetes/pod.yaml".to_string()),
            Just("terraform/main.tf".to_string()),
        ],
        0..=5,
    );
    let random = prop::collection::vec(file_path(), 0..=8);
    (markers, random).prop_map(|(mut m, r)| {
        m.extend(r);
        m.sort();
        m.dedup();
        m
    })
}

fn export_from_paths(paths: Vec<String>) -> ExportData {
    let rows = paths
        .into_iter()
        .map(|p| FileRow {
            path: p,
            module: "(root)".to_string(),
            lang: "Unknown".to_string(),
            kind: FileKind::Parent,
            code: 0,
            comments: 0,
            blanks: 0,
            lines: 0,
            bytes: 0,
            tokens: 0,
        })
        .collect();
    ExportData {
        rows,
        module_roots: vec![],
        module_depth: 2,
        children: ChildIncludeMode::Separate,
    }
}

// ---------------------------------------------------------------------------
// Properties
// ---------------------------------------------------------------------------

proptest! {
    /// detect_archetype must never panic regardless of input.
    #[test]
    fn never_panics(paths in file_paths()) {
        let export = export_from_paths(paths);
        let _ = detect_archetype(&export);
    }

    /// When an archetype is detected, evidence must be non-empty.
    #[test]
    fn evidence_non_empty_when_detected(paths in file_paths()) {
        let export = export_from_paths(paths);
        if let Some(a) = detect_archetype(&export) {
            prop_assert!(!a.evidence.is_empty(), "evidence must not be empty for kind={}", a.kind);
        }
    }

    /// The returned kind must be one of the known archetype names.
    #[test]
    fn kind_is_known(paths in file_paths()) {
        let export = export_from_paths(paths);
        if let Some(a) = detect_archetype(&export) {
            prop_assert!(
                KNOWN_KINDS.contains(&a.kind.as_str()),
                "unexpected kind: {}", a.kind
            );
        }
    }

    /// detect_archetype is deterministic — same input always gives same output.
    #[test]
    fn deterministic(paths in file_paths()) {
        let export1 = export_from_paths(paths.clone());
        let export2 = export_from_paths(paths);
        let r1 = detect_archetype(&export1);
        let r2 = detect_archetype(&export2);
        match (&r1, &r2) {
            (None, None) => {}
            (Some(a1), Some(a2)) => {
                prop_assert_eq!(&a1.kind, &a2.kind);
                prop_assert_eq!(&a1.evidence, &a2.evidence);
            }
            _ => prop_assert!(false, "determinism violated: {:?} vs {:?}", r1, r2),
        }
    }

    /// Empty rows always produce None.
    #[test]
    fn empty_rows_always_none(_seed in 0u32..1000) {
        let export = ExportData {
            rows: vec![],
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        prop_assert!(detect_archetype(&export).is_none());
    }

    /// Child-only rows must never trigger archetype detection.
    #[test]
    fn child_only_rows_always_none(paths in file_paths()) {
        let rows: Vec<FileRow> = paths
            .into_iter()
            .map(|p| FileRow {
                path: p,
                module: "(root)".to_string(),
                lang: "Unknown".to_string(),
                kind: FileKind::Child,
                code: 0,
                comments: 0,
                blanks: 0,
                lines: 0,
                bytes: 0,
                tokens: 0,
            })
            .collect();
        let export = ExportData {
            rows,
            module_roots: vec![],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };
        prop_assert!(detect_archetype(&export).is_none(), "Child rows should never trigger detection");
    }

    /// Adding unrelated files to a recognized layout must not change the detected archetype kind.
    /// (monotonicity: recognition is stable when noise is added)
    #[test]
    fn adding_noise_preserves_archetype(
        base in prop::sample::subsequence(
            vec![
                "Cargo.toml".to_string(),
                "crates/x/src/lib.rs".to_string(),
            ],
            2..=2,
        ),
        noise in prop::collection::vec(file_path(), 0..=5),
    ) {
        let base_export = export_from_paths(base.clone());
        let base_result = detect_archetype(&base_export);

        let mut extended = base;
        extended.extend(noise);
        extended.sort();
        extended.dedup();
        let ext_export = export_from_paths(extended);
        let ext_result = detect_archetype(&ext_export);

        // The base is a Rust workspace; extended must still be detected
        if let Some(base_a) = &base_result {
            if let Some(ext_a) = &ext_result {
                // The kind should remain the same or be a more specific variant
                prop_assert!(
                    ext_a.kind.starts_with("Rust workspace"),
                    "noise changed archetype from {:?} to {:?}",
                    base_a.kind,
                    ext_a.kind
                );
            } else {
                prop_assert!(false, "adding noise removed archetype detection");
            }
        }
    }
}
