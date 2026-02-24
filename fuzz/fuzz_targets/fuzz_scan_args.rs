//! Fuzz target for ScanArgs construction and redaction mode behavior.
//!
//! Validates invariants for:
//! - Path normalization
//! - Redaction mode toggles
//! - Deterministic output
//! - Ignore-flag fan-out behavior

#![no_main]

use std::path::PathBuf;

use libfuzzer_sys::fuzz_target;
use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

const MAX_INPUT_SIZE: usize = 16 * 1024;
const MAX_LIST_ITEMS: usize = 32;
const SECTION_SPLIT: char = '\u{1e}';
const ITEM_SPLIT: char = '\u{1f}';

fn decode_mode(selector: u8) -> Option<RedactMode> {
    match selector % 4 {
        0 => None,
        1 => Some(RedactMode::None),
        2 => Some(RedactMode::Paths),
        _ => Some(RedactMode::All),
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 || data.len() > MAX_INPUT_SIZE {
        return;
    }

    let redact = decode_mode(data[0]);
    let hidden = data[1] & 1 == 1;
    let no_ignore = data[2] & 1 == 1;
    let no_ignore_parent = data[3] & 1 == 1;
    let no_ignore_dot = data[4] & 1 == 1;
    let no_ignore_vcs = data[5] & 1 == 1;
    let treat_doc_strings_as_comments = data[6] & 1 == 1;

    let Ok(payload) = std::str::from_utf8(&data[7..]) else {
        return;
    };

    let mut sections = payload.splitn(3, SECTION_SPLIT);
    let path_section = sections.next().unwrap_or_default();
    let excluded_section = sections.next().unwrap_or_default();

    let paths: Vec<PathBuf> = path_section
        .split(ITEM_SPLIT)
        .take(MAX_LIST_ITEMS)
        .map(PathBuf::from)
        .collect();

    let excluded: Vec<String> = excluded_section
        .split(ITEM_SPLIT)
        .take(MAX_LIST_ITEMS)
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

    // Determinism: same input must produce same output.
    assert_eq!(args.paths, args2.paths);
    assert_eq!(args.excluded, args2.excluded);
    assert_eq!(args.excluded_redacted, args2.excluded_redacted);
    assert_eq!(args.no_ignore_parent, args2.no_ignore_parent);
    assert_eq!(args.no_ignore_dot, args2.no_ignore_dot);
    assert_eq!(args.no_ignore_vcs, args2.no_ignore_vcs);

    // Paths are always one-to-one with input paths.
    assert_eq!(args.paths.len(), paths.len());

    // Normalization invariant: output paths never contain backslashes.
    assert!(args.paths.iter().all(|p| !p.contains('\\')));

    let should_redact = matches!(redact, Some(RedactMode::Paths | RedactMode::All));
    let expected_excluded_redacted = should_redact && !excluded.is_empty();
    assert_eq!(args.excluded_redacted, expected_excluded_redacted);

    if should_redact {
        // Redacted paths are hash-based and should not include separators.
        assert!(
            args.paths
                .iter()
                .all(|p| !p.contains('/') && !p.contains('\\'))
        );

        // Excluded patterns are short hashes (16 lowercase hex chars).
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

    // no_ignore should force sub-flags true.
    if no_ignore {
        assert!(args.no_ignore_parent);
        assert!(args.no_ignore_dot);
        assert!(args.no_ignore_vcs);
    }
});
