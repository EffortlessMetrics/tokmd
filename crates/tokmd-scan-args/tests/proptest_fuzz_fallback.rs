use proptest::prelude::*;
use std::path::PathBuf;
use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

fn decode_mode(selector: u8) -> Option<RedactMode> {
    match selector % 4 {
        0 => None,
        1 => Some(RedactMode::None),
        2 => Some(RedactMode::Paths),
        _ => Some(RedactMode::All),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5000))]

    #[test]
    fn scan_args_invariants(
        redact_sel in 0u8..4,
        hidden in any::<bool>(),
        no_ignore in any::<bool>(),
        no_ignore_parent in any::<bool>(),
        no_ignore_dot in any::<bool>(),
        no_ignore_vcs in any::<bool>(),
        treat_doc_strings_as_comments in any::<bool>(),
        path_section in "\\PC*",
        excluded_section in "\\PC*"
    ) {
        let redact = decode_mode(redact_sel);

        let paths: Vec<PathBuf> = path_section
            .split('\u{1f}')
            .take(32)
            .map(PathBuf::from)
            .collect();

        let excluded: Vec<String> = excluded_section
            .split('\u{1f}')
            .take(32)
            .map(ToString::to_string)
            .collect();

        let scan_options = ScanOptions {
            excluded: excluded.clone(),
            hidden,
            no_ignore,
            no_ignore_parent,
            no_ignore_dot,
            no_ignore_vcs,
            treat_doc_strings_as_comments,
            ..Default::default()
        };

        let args = scan_args(&paths, &scan_options, redact);
        let args2 = scan_args(&paths, &scan_options, redact);

        assert_eq!(args.paths, args2.paths);
        assert_eq!(args.excluded, args2.excluded);
        assert_eq!(args.excluded_redacted, args2.excluded_redacted);
        assert_eq!(args.no_ignore_parent, args2.no_ignore_parent);
        assert_eq!(args.no_ignore_dot, args2.no_ignore_dot);
        assert_eq!(args.no_ignore_vcs, args2.no_ignore_vcs);

        assert_eq!(args.paths.len(), paths.len());

        assert!(args.paths.iter().all(|p| !p.contains('\\')));

        let should_redact = matches!(redact, Some(RedactMode::Paths | RedactMode::All));
        let expected_excluded_redacted = should_redact && !excluded.is_empty();
        assert_eq!(args.excluded_redacted, expected_excluded_redacted);

        if should_redact {
            assert!(
                args.paths
                    .iter()
                    .all(|p| !p.contains('/') && !p.contains('\\'))
            );

            assert_eq!(args.excluded.len(), excluded.len());
            assert!(args.excluded.iter().all(|value| {
                value.len() == 16
                    && value
                        .chars()
                        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            }));
        } else {
            let expected_paths: Vec<String> = paths.iter().map(|p| normalize_scan_input(p)).collect();
            assert_eq!(args.paths, expected_paths);
            assert_eq!(args.excluded, excluded);
        }

        if no_ignore {
            assert!(args.no_ignore_parent);
            assert!(args.no_ignore_dot);
            assert!(args.no_ignore_vcs);
        }
    }
}
