//! Request and control types for effort estimation.
//!
//! These types form the boundary between CLI/config plumbing and the effort
//! engine. They are intentionally small and serializable in spirit:
//!
//! - `EffortModelKind` chooses the estimation strategy,
//! - `EffortLayer` controls how much of the estimate is intended for presentation,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effort_request_default_pins_engine_baseline() {
        let req = EffortRequest::default();
        assert_eq!(req.model, EffortModelKind::Cocomo81Basic);
        assert_eq!(req.layer, EffortLayer::Full);
        assert!(req.base_ref.is_none());
        assert!(req.head_ref.is_none());
        assert!(!req.monte_carlo);
        assert_eq!(req.mc_iterations, 10_000);
        assert!(req.mc_seed.is_none());
    }

    #[test]
    fn effort_model_kind_string_forms_are_stable() {
        for (kind, expected) in [
            (EffortModelKind::Cocomo81Basic, "cocomo81-basic"),
            (EffortModelKind::Cocomo2Early, "cocomo2-early"),
            (EffortModelKind::Ensemble, "ensemble"),
        ] {
            assert_eq!(kind.as_str(), expected);
            assert_eq!(kind.to_string(), expected);
        }
    }

    #[test]
    fn effort_layer_string_forms_are_stable() {
        for (layer, expected) in [
            (EffortLayer::Headline, "headline"),
            (EffortLayer::Why, "why"),
            (EffortLayer::Full, "full"),
        ] {
            assert_eq!(layer.as_str(), expected);
            assert_eq!(layer.to_string(), expected);
        }
    }
}
