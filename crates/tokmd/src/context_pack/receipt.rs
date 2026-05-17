//! Shared context receipt construction helpers.

use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

use tokmd_types::{CONTEXT_SCHEMA_VERSION, ContextFileRow, ContextReceipt, TokenAudit, ToolInfo};

use crate::cli;

use super::SelectResult;

pub(crate) struct ContextReceiptParams<'a> {
    pub(crate) args: &'a cli::CliContextArgs,
    pub(crate) selected: &'a [ContextFileRow],
    pub(crate) budget: usize,
    pub(crate) used_tokens: usize,
    pub(crate) utilization: f64,
    pub(crate) select_result: &'a SelectResult,
    pub(crate) generated_at_ms: u128,
    pub(crate) bundle_audit: Option<TokenAudit>,
}

pub(crate) fn generated_at_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub(crate) fn lower_debug(value: &impl Debug) -> String {
    format!("{value:?}").to_lowercase()
}

pub(crate) fn total_file_bytes(selected: &[ContextFileRow]) -> usize {
    selected.iter().map(|file| file.bytes).sum()
}

pub(crate) fn rank_by_effective(select_result: &SelectResult) -> Option<String> {
    select_result
        .fallback_reason
        .as_ref()
        .map(|_| select_result.rank_by_effective.clone())
}

pub(crate) fn token_estimation(selected: &[ContextFileRow]) -> tokmd_types::TokenEstimationMeta {
    tokmd_types::TokenEstimationMeta::from_bytes(total_file_bytes(selected), 4.0)
}

pub(crate) fn build_context_receipt(params: ContextReceiptParams<'_>) -> ContextReceipt {
    ContextReceipt {
        schema_version: CONTEXT_SCHEMA_VERSION,
        generated_at_ms: params.generated_at_ms,
        tool: ToolInfo::current(),
        mode: "context".to_string(),
        budget_tokens: params.budget,
        used_tokens: params.used_tokens,
        utilization_pct: params.utilization,
        strategy: lower_debug(&params.args.strategy),
        rank_by: lower_debug(&params.args.rank_by),
        file_count: params.selected.len(),
        files: params.selected.to_vec(),
        rank_by_effective: rank_by_effective(params.select_result),
        fallback_reason: params.select_result.fallback_reason.clone(),
        excluded_by_policy: params.select_result.excluded_by_policy.clone(),
        token_estimation: Some(token_estimation(params.selected)),
        bundle_audit: params.bundle_audit,
    }
}
