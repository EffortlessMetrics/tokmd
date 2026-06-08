use tokmd_format::redact_path;

#[test]
fn redaction_must_normalize_case_for_safe_extensions_hash() {
    let lower = redact_path("src/lib.rs");
    let upper = redact_path("src/lib.RS");
    let mixed = redact_path("src/lib.Rs");

    assert_eq!(lower, upper, "Upper-case extension produced a different hash/redaction: {} != {}", lower, upper);
    assert_eq!(lower, mixed, "Mixed-case extension produced a different hash/redaction: {} != {}", lower, mixed);

    assert!(lower.ends_with(".rs"));
    assert!(upper.ends_with(".rs"));
    assert!(mixed.ends_with(".rs"));
}
