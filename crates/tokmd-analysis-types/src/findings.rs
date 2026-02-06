//! Finding ID registry for cockpit outputs.
//!
//! Stable identifiers for tokmd findings, following `<tool>.<category>.<code>` pattern.

/// Risk-related findings
pub mod risk {
    /// High-churn file modified
    pub const HOTSPOT: &str = "tokmd.risk.hotspot";
    /// High-coupling file modified
    pub const COUPLING: &str = "tokmd.risk.coupling";
    /// Single-author file modified
    pub const BUS_FACTOR: &str = "tokmd.risk.bus_factor";
    /// Cyclomatic complexity above threshold
    pub const COMPLEXITY_HIGH: &str = "tokmd.risk.complexity_high";
    /// Cognitive complexity above threshold
    pub const COGNITIVE_HIGH: &str = "tokmd.risk.cognitive_high";
    /// Deep nesting detected
    pub const NESTING_DEEP: &str = "tokmd.risk.nesting_deep";
}

/// Contract-related findings
pub mod contract {
    /// Schema version changed
    pub const SCHEMA_CHANGED: &str = "tokmd.contract.schema_changed";
    /// Public API surface changed
    pub const API_CHANGED: &str = "tokmd.contract.api_changed";
    /// CLI interface changed
    pub const CLI_CHANGED: &str = "tokmd.contract.cli_changed";
}

/// Supply chain findings
pub mod supply {
    /// Dependency lockfile modified
    pub const LOCKFILE_CHANGED: &str = "tokmd.supply.lockfile_changed";
    /// New dependency added
    pub const NEW_DEPENDENCY: &str = "tokmd.supply.new_dependency";
    /// Vulnerable dependency detected
    pub const VULNERABILITY: &str = "tokmd.supply.vulnerability";
}

/// Gate-related findings
pub mod gate {
    /// Mutation testing threshold not met
    pub const MUTATION_FAILED: &str = "tokmd.gate.mutation_failed";
    /// Diff coverage threshold not met
    pub const COVERAGE_FAILED: &str = "tokmd.gate.coverage_failed";
    /// Complexity gate failed
    pub const COMPLEXITY_FAILED: &str = "tokmd.gate.complexity_failed";
}

/// Security-related findings
pub mod security {
    /// High-entropy file (potential secrets)
    pub const ENTROPY_HIGH: &str = "tokmd.security.entropy_high";
    /// License compatibility issue
    pub const LICENSE_CONFLICT: &str = "tokmd.security.license_conflict";
}

/// Architecture-related findings
pub mod architecture {
    /// Circular import detected
    pub const CIRCULAR_DEP: &str = "tokmd.architecture.circular_dep";
    /// Architecture boundary crossed
    pub const LAYER_VIOLATION: &str = "tokmd.architecture.layer_violation";
}
