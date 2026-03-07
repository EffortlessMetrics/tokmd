use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct EffortRequest {
    pub model: EffortModelKind,
    pub layer: EffortLayer,
    pub base_ref: Option<String>,
    pub head_ref: Option<String>,
    pub monte_carlo: bool,
    pub mc_iterations: u32,
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

#[derive(Debug, Clone)]
pub struct DeltaInput {
    pub base_ref: String,
    pub head_ref: String,
}
