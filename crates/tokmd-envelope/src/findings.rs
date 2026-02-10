//! Finding ID registry for sensor outputs.
//!
//! Each module provides a `CHECK_ID` constant for the category and
//! individual code constants. Combined with `tool.name`, these form
//! the identity triple `(tool, check_id, code)` for buildfix routing.

/// Risk-related findings
pub mod risk {
    /// Check category identifier.
    pub const CHECK_ID: &str = "risk";
    /// High-churn file modified
    pub const HOTSPOT: &str = "hotspot";
    /// High-coupling file modified
    pub const COUPLING: &str = "coupling";
    /// Single-author file modified
    pub const BUS_FACTOR: &str = "bus_factor";
    /// Cyclomatic complexity above threshold
    pub const COMPLEXITY_HIGH: &str = "complexity_high";
    /// Cognitive complexity above threshold
    pub const COGNITIVE_HIGH: &str = "cognitive_high";
    /// Deep nesting detected
    pub const NESTING_DEEP: &str = "nesting_deep";
}

/// Contract-related findings
pub mod contract {
    /// Check category identifier.
    pub const CHECK_ID: &str = "contract";
    /// Schema version changed
    pub const SCHEMA_CHANGED: &str = "schema_changed";
    /// Public API surface changed
    pub const API_CHANGED: &str = "api_changed";
    /// CLI interface changed
    pub const CLI_CHANGED: &str = "cli_changed";
}

/// Supply chain findings
pub mod supply {
    /// Check category identifier.
    pub const CHECK_ID: &str = "supply";
    /// Dependency lockfile modified
    pub const LOCKFILE_CHANGED: &str = "lockfile_changed";
    /// New dependency added
    pub const NEW_DEPENDENCY: &str = "new_dependency";
    /// Vulnerable dependency detected
    pub const VULNERABILITY: &str = "vulnerability";
}

/// Gate-related findings
pub mod gate {
    /// Check category identifier.
    pub const CHECK_ID: &str = "gate";
    /// Mutation testing threshold not met
    pub const MUTATION_FAILED: &str = "mutation_failed";
    /// Diff coverage threshold not met
    pub const COVERAGE_FAILED: &str = "coverage_failed";
    /// Complexity gate failed
    pub const COMPLEXITY_FAILED: &str = "complexity_failed";
}

/// Security-related findings
pub mod security {
    /// Check category identifier.
    pub const CHECK_ID: &str = "security";
    /// High-entropy file (potential secrets)
    pub const ENTROPY_HIGH: &str = "entropy_high";
    /// License compatibility issue
    pub const LICENSE_CONFLICT: &str = "license_conflict";
}

/// Architecture-related findings
pub mod architecture {
    /// Check category identifier.
    pub const CHECK_ID: &str = "architecture";
    /// Circular import detected
    pub const CIRCULAR_DEP: &str = "circular_dep";
    /// Architecture boundary crossed
    pub const LAYER_VIOLATION: &str = "layer_violation";
}

/// Sensor-level findings (diff summary, etc.)
pub mod sensor {
    /// Check category identifier.
    pub const CHECK_ID: &str = "sensor";
    /// Diff summary finding
    pub const DIFF_SUMMARY: &str = "diff_summary";
}

/// Compose a fully-qualified finding ID from the triple `(tool, check_id, code)`.
pub fn finding_id(tool_name: &str, check_id: &str, code: &str) -> String {
    format!("{}.{}.{}", tool_name, check_id, code)
}
