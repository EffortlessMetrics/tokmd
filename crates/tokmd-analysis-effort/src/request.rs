//! Request and control types for effort estimation.
//!
//! These types form the boundary between CLI/config plumbing and the effort
//! engine. They are intentionally small and serializable in spirit:
//!
//! - `EffortModelKind` chooses the estimation strategy,
//! - `EffortLayer` controls how much of the estimate is intended for presentation,
//! - `DeltaInput` describes a base/head comparison window,
//! - `EffortRequest` bundles the knobs needed by `build_effort_report`.
//!
//! The request surface is allowed to be ahead of the implementation surface.
//! In other words, callers may be able to request models or uncertainty modes
//! that are parsed and validated before every variant is fully implemented.

use std::fmt::{Display, Formatter};

/// Request object passed into the effort engine.
///
/// This is the computation-facing version of the CLI/config surface. It is
/// intentionally explicit so the builder can remain deterministic and avoid
/// reaching back into argument parsing layers.
///
/// Notes:
/// - `model` selects the requested estimate family,
/// - `layer` is presentation-oriented metadata,
/// - `base_ref` / `head_ref` enable delta output,
/// - Monte Carlo fields are carried here even if the current engine chooses to
///   reject or ignore them while only deterministic paths are implemented.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffortRequest {
    /// Estimation model requested by the caller.
    pub model: EffortModelKind,
    /// Requested presentation depth for effort output.
    pub layer: EffortLayer,
    /// Optional base reference for change-window estimation.
    pub base_ref: Option<String>,
    /// Optional head reference for change-window estimation.
    pub head_ref: Option<String>,
    /// Enable Monte Carlo uncertainty estimation.
    pub monte_carlo: bool,
    /// Monte Carlo sample count when uncertainty estimation is enabled.
    pub mc_iterations: usize,
    /// Optional deterministic seed for Monte Carlo.
    pub mc_seed: Option<u64>,
}

impl Default for EffortRequest {
    fn default() -> Self {
        Self {
            model: EffortModelKind::Cocomo81Basic,
            layer: EffortLayer::Full,
            base_ref: None,
            head_ref: None,
            monte_carlo: false,
            mc_iterations: 10_000,
            mc_seed: None,
        }
    }
}

/// Effort-estimation model requested by the caller.
///
/// `Cocomo81Basic` is the deterministic baseline.
/// Other variants may be accepted by the request layer before the underlying
/// engine fully implements them; those cases should fail clearly rather than
/// silently degrading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffortModelKind {
    Cocomo81Basic,
    Cocomo2Early,
    Ensemble,
}

impl EffortModelKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cocomo81Basic => "cocomo81-basic",
            Self::Cocomo2Early => "cocomo2-early",
            Self::Ensemble => "ensemble",
        }
    }
}

impl Display for EffortModelKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Requested presentation depth for effort output.
///
/// This is primarily a rendering hint:
/// - `Headline` focuses on summary numbers,
/// - `Why` adds explanatory context,
/// - `Full` includes assumptions and optional delta details.
///
/// The engine may still compute richer data than the selected layer displays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffortLayer {
    Headline,
    Why,
    Full,
}

impl EffortLayer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Headline => "headline",
            Self::Why => "why",
            Self::Full => "full",
        }
    }
}

impl Display for EffortLayer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Optional base/head comparison input for delta estimation.
///
/// When present, the effort engine may attach a delta section describing the
/// blast radius and estimated effort impact of the change window.
#[derive(Debug, Clone)]
pub struct DeltaInput {
    pub base_ref: String,
    pub head_ref: String,
}
