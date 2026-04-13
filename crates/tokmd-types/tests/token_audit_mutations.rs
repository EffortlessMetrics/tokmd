use tokmd_types::{TokenAudit, TokenEstimationMeta, ToolInfo};

#[test]
fn test_token_audit_from_output_divisors() {
    let audit = TokenAudit::from_output_with_divisors(100, 80, 2.0, 1.0, 4.0);
    assert_eq!(audit.output_bytes, 100);
    assert_eq!(audit.overhead_bytes, 20);
    assert_eq!(audit.overhead_pct, 0.2);
    assert_eq!(audit.tokens_min, 25);
    assert_eq!(audit.tokens_est, 50);
    assert_eq!(audit.tokens_max, 100);

    let audit_ceil = TokenAudit::from_output_with_divisors(101, 80, 2.0, 1.0, 4.0);
    assert_eq!(audit_ceil.tokens_min, 26);
    assert_eq!(audit_ceil.tokens_est, 51);
    assert_eq!(audit_ceil.tokens_max, 101);

    let audit_zero = TokenAudit::from_output_with_divisors(0, 0, 2.0, 1.0, 4.0);
    assert_eq!(audit_zero.output_bytes, 0);
    assert_eq!(audit_zero.overhead_bytes, 0);
    assert_eq!(audit_zero.overhead_pct, 0.0);
    assert_eq!(audit_zero.tokens_min, 0);
    assert_eq!(audit_zero.tokens_est, 0);
    assert_eq!(audit_zero.tokens_max, 0);

    let audit_neg = TokenAudit::from_output_with_divisors(80, 100, 2.0, 1.0, 4.0);
    assert_eq!(audit_neg.overhead_bytes, 0);
}

#[test]
fn test_token_estimation_meta_from_bytes_with_bounds() {
    let est = TokenEstimationMeta::from_bytes_with_bounds(100, 2.0, 1.0, 4.0);
    assert_eq!(est.source_bytes, 100);
    assert_eq!(est.bytes_per_token_est, 2.0);
    assert_eq!(est.bytes_per_token_low, 1.0);
    assert_eq!(est.bytes_per_token_high, 4.0);
    assert_eq!(est.tokens_min, 25);
    assert_eq!(est.tokens_est, 50);
    assert_eq!(est.tokens_max, 100);

    let est_ceil = TokenEstimationMeta::from_bytes_with_bounds(101, 2.0, 1.0, 4.0);
    assert_eq!(est_ceil.tokens_min, 26);
    assert_eq!(est_ceil.tokens_est, 51);
    assert_eq!(est_ceil.tokens_max, 101);
}

#[test]
fn test_tool_info_current() {
    let tool = ToolInfo::current();
    assert_eq!(tool.name, "tokmd");
    assert!(!tool.version.is_empty());
}
