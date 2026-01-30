//! # tokmd-gate
//!
//! **Tier 2 (Policy Evaluation)**
//!
//! Policy evaluation engine for CI gating based on analysis receipts.
//!
//! ## What belongs here
//! * Policy rule types and parsing
//! * JSON Pointer resolution
//! * Rule evaluation logic
//!
//! ## Example
//! ```ignore
//! use tokmd_gate::{PolicyConfig, evaluate_policy};
//!
//! let receipt = serde_json::from_str(json)?;
//! let policy = PolicyConfig::from_file("policy.toml")?;
//! let result = evaluate_policy(&receipt, &policy)?;
//! ```

mod evaluate;
mod pointer;
mod types;

pub use evaluate::evaluate_policy;
pub use pointer::resolve_pointer;
pub use types::{
    GateError, GateResult, PolicyConfig, PolicyRule, RuleLevel, RuleOperator, RuleResult,
};
