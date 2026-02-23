#![no_main]

use libfuzzer_sys::fuzz_target;
use tokmd_context_policy::{
    DEFAULT_DENSE_THRESHOLD, DEFAULT_MAX_FILE_PCT, assign_policy, classify_file, compute_file_cap,
    is_spine_file, smart_exclude_reason,
};
use tokmd_types::InclusionPolicy;

const MAX_INPUT_SIZE: usize = 8 * 1024;
const SPLIT_BYTE: u8 = 0x1f;

fn parse_usize(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        return 0;
    }

    bytes.iter().take(8).fold(0usize, |acc, b| {
        acc.wrapping_mul(257).wrapping_add(*b as usize)
    })
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > MAX_INPUT_SIZE {
        return;
    }

    let mut parts = data.splitn(4, |b| *b == SPLIT_BYTE);
    let path = String::from_utf8_lossy(parts.next().unwrap_or_default());
    let tokens = parse_usize(parts.next().unwrap_or_default());
    let lines = parse_usize(parts.next().unwrap_or_default());
    let budget = parse_usize(parts.next().unwrap_or_default());

    let _ = is_spine_file(path.as_ref());

    if let Some(reason) = smart_exclude_reason(path.as_ref()) {
        assert!(matches!(reason, "lockfile" | "minified" | "sourcemap"));
    }

    let classes = classify_file(path.as_ref(), tokens, lines, DEFAULT_DENSE_THRESHOLD);
    let mut sorted = classes.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(classes, sorted);

    let cap_default = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, None);
    let cap_hard = compute_file_cap(budget, DEFAULT_MAX_FILE_PCT, Some(4_000));
    assert!(cap_hard <= 4_000 || cap_hard == usize::MAX);

    let (policy_default, reason_default) = assign_policy(tokens, cap_default, &classes);
    match policy_default {
        InclusionPolicy::Full => {
            assert!(tokens <= cap_default);
            assert!(reason_default.is_none());
        }
        InclusionPolicy::HeadTail | InclusionPolicy::Skip => {
            if cap_default != usize::MAX {
                assert!(tokens > cap_default);
                assert!(reason_default.is_some());
            }
        }
        InclusionPolicy::Summary => {}
    }
});
