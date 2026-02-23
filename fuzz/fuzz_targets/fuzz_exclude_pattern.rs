#![no_main]

use std::path::PathBuf;

use libfuzzer_sys::fuzz_target;
use tokmd_exclude::{add_exclude_pattern, has_exclude_pattern, normalize_exclude_pattern};

const MAX_INPUT_SIZE: usize = 8 * 1024;
const SPLIT_BYTE: u8 = 0x1f;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > MAX_INPUT_SIZE {
        return;
    }

    let split = data
        .iter()
        .position(|b| *b == SPLIT_BYTE)
        .unwrap_or(data.len() / 2);

    let root_bytes = &data[..split];
    let path_bytes = if split < data.len() {
        &data[split + 1..]
    } else {
        &data[split..]
    };

    let root = String::from_utf8_lossy(root_bytes);
    let path = String::from_utf8_lossy(path_bytes);

    let root_path = PathBuf::from(root.as_ref());
    let target_path = PathBuf::from(path.as_ref());

    let normalized = normalize_exclude_pattern(&root_path, &target_path);
    let normalized_again = normalize_exclude_pattern(&root_path, &target_path);

    // Determinism and slash normalization are core invariants.
    assert_eq!(normalized, normalized_again);
    assert!(!normalized.contains('\\'));

    let mut seeded = vec![format!("./{}", normalized.replace('/', "\\"))];
    if normalized.is_empty() {
        assert!(has_exclude_pattern(&seeded, ""));
    } else {
        assert!(has_exclude_pattern(&seeded, &normalized));
    }
    assert!(!add_exclude_pattern(&mut seeded, normalized.clone()));

    let mut empty = Vec::new();
    let inserted_once = add_exclude_pattern(&mut empty, normalized.clone());
    let inserted_twice = add_exclude_pattern(&mut empty, format!("./{normalized}"));
    if normalized.is_empty() {
        assert!(!inserted_once);
        assert!(!inserted_twice);
        assert!(empty.is_empty());
    } else {
        assert!(inserted_once);
        assert!(!inserted_twice);
        assert_eq!(empty.len(), 1);
    }
});
