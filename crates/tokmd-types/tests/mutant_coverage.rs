use tokmd_types::{ToolInfo, TokenEstimationMeta, TokenAudit, ContextFileRow, InclusionPolicy};

#[test]
fn test_tool_info_current_is_not_default() {
    let current = ToolInfo::current();
    let default = ToolInfo::default();
    assert_ne!(current.name, default.name);
    assert_ne!(current.version, default.version);
}

#[test]
fn test_token_estimation_meta_from_bytes_zero() {
    let meta = TokenEstimationMeta::from_bytes(0, 4.0);
    assert_eq!(meta.tokens_min, 0);
    assert_eq!(meta.tokens_est, 0);
    assert_eq!(meta.tokens_max, 0);
}

#[test]
fn test_token_estimation_meta_from_bytes_non_zero() {
    let meta = TokenEstimationMeta::from_bytes(100, 4.0);
    assert_ne!(meta.tokens_min, 0);
    assert_ne!(meta.tokens_est, 0);
    assert_ne!(meta.tokens_max, 0);
}

#[test]
fn test_token_estimation_meta_bounds_zero() {
    let meta = TokenEstimationMeta::from_bytes_with_bounds(0, 4.0, 3.0, 5.0);
    assert_eq!(meta.tokens_min, 0);
    assert_eq!(meta.tokens_est, 0);
    assert_eq!(meta.tokens_max, 0);
}

#[test]
fn test_token_estimation_meta_bounds_non_zero() {
    let meta = TokenEstimationMeta::from_bytes_with_bounds(100, 4.0, 3.0, 5.0);
    assert_ne!(meta.tokens_min, 0);
    assert_ne!(meta.tokens_est, 0);
    assert_ne!(meta.tokens_max, 0);

    // Check maths (explicit f64)
    assert_eq!(meta.tokens_min, (100.0_f64 / 5.0_f64).ceil() as usize);
    assert_eq!(meta.tokens_est, (100.0_f64 / 4.0_f64).ceil() as usize);
    assert_eq!(meta.tokens_max, (100.0_f64 / 3.0_f64).ceil() as usize);
}

#[test]
fn test_token_audit_from_output_zero() {
    let audit = TokenAudit::from_output(0, 0);
    assert_eq!(audit.tokens_min, 0);
    assert_eq!(audit.tokens_est, 0);
    assert_eq!(audit.tokens_max, 0);
}

#[test]
fn test_token_audit_from_output_non_zero() {
    let audit = TokenAudit::from_output(100, 50);
    assert_ne!(audit.tokens_min, 0);
    assert_ne!(audit.tokens_est, 0);
    assert_ne!(audit.tokens_max, 0);
}

#[test]
fn test_token_audit_from_output_with_divisors_zero() {
    let audit = TokenAudit::from_output_with_divisors(0, 0, 4.0, 3.0, 5.0);
    assert_eq!(audit.tokens_min, 0);
    assert_eq!(audit.tokens_est, 0);
    assert_eq!(audit.tokens_max, 0);
    assert_eq!(audit.overhead_pct, 0.0);
}

#[test]
fn test_token_audit_from_output_with_divisors_non_zero() {
    let audit = TokenAudit::from_output_with_divisors(100, 50, 4.0, 3.0, 5.0);
    assert_ne!(audit.tokens_min, 0);
    assert_ne!(audit.tokens_est, 0);
    assert_ne!(audit.tokens_max, 0);
    assert_ne!(audit.overhead_pct, 0.0);

    // Check maths
    assert_eq!(audit.tokens_min, (100.0_f64 / 5.0_f64).ceil() as usize);
    assert_eq!(audit.tokens_est, (100.0_f64 / 4.0_f64).ceil() as usize);
    assert_eq!(audit.tokens_max, (100.0_f64 / 3.0_f64).ceil() as usize);
    assert_eq!(audit.overhead_bytes, 50);
    assert_eq!(audit.overhead_pct, 50.0 / 100.0);
}

#[test]
fn test_is_default_policy_serialization() {
    let full = ContextFileRow {
        path: "a".into(),
        module: "a".into(),
        lang: "a".into(),
        bytes: 10,
        tokens: 10,
        code: 10,
        lines: 10,
        value: 10,
        rank_reason: "".into(),
        policy: InclusionPolicy::Full,
        effective_tokens: None,
        policy_reason: None,
        classifications: vec![],
    };
    let json = serde_json::to_string(&full).unwrap();
    assert!(!json.contains("\"policy\":"));

    let skip = ContextFileRow {
        path: "a".into(),
        module: "a".into(),
        lang: "a".into(),
        bytes: 10,
        tokens: 10,
        code: 10,
        lines: 10,
        value: 10,
        rank_reason: "".into(),
        policy: InclusionPolicy::Skip,
        effective_tokens: None,
        policy_reason: None,
        classifications: vec![],
    };
    let json_skip = serde_json::to_string(&skip).unwrap();
    assert!(json_skip.contains("\"policy\":\"skip\""));
}
