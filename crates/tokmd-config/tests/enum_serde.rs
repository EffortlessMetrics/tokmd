//! Tests for enum serde serialization specifics beyond round-trips.
//!
//! Focuses on exact serialized string values, case sensitivity,
//! and kebab-case enforcement for enums not covered by properties.rs.

use tokmd_config::{
    CockpitFormat, ColorMode, ContextOutput, ContextStrategy, DiffFormat, DiffRangeMode,
    GateFormat, HandoffPreset, NearDupScope, SensorFormat, ValueMetric,
};

// =========================================================================
// Exact serialized string values
// =========================================================================

#[test]
fn diff_format_serialized_values() {
    assert_eq!(serde_json::to_string(&DiffFormat::Md).unwrap(), "\"md\"");
    assert_eq!(
        serde_json::to_string(&DiffFormat::Json).unwrap(),
        "\"json\""
    );
}

#[test]
fn color_mode_serialized_values() {
    assert_eq!(serde_json::to_string(&ColorMode::Auto).unwrap(), "\"auto\"");
    assert_eq!(
        serde_json::to_string(&ColorMode::Always).unwrap(),
        "\"always\""
    );
    assert_eq!(
        serde_json::to_string(&ColorMode::Never).unwrap(),
        "\"never\""
    );
}

#[test]
fn cockpit_format_serialized_values() {
    assert_eq!(
        serde_json::to_string(&CockpitFormat::Json).unwrap(),
        "\"json\""
    );
    assert_eq!(serde_json::to_string(&CockpitFormat::Md).unwrap(), "\"md\"");
    assert_eq!(
        serde_json::to_string(&CockpitFormat::Sections).unwrap(),
        "\"sections\""
    );
}

#[test]
fn gate_format_serialized_values() {
    assert_eq!(
        serde_json::to_string(&GateFormat::Text).unwrap(),
        "\"text\""
    );
    assert_eq!(
        serde_json::to_string(&GateFormat::Json).unwrap(),
        "\"json\""
    );
}

#[test]
fn context_strategy_serialized_values() {
    assert_eq!(
        serde_json::to_string(&ContextStrategy::Greedy).unwrap(),
        "\"greedy\""
    );
    assert_eq!(
        serde_json::to_string(&ContextStrategy::Spread).unwrap(),
        "\"spread\""
    );
}

#[test]
fn value_metric_serialized_values() {
    assert_eq!(
        serde_json::to_string(&ValueMetric::Code).unwrap(),
        "\"code\""
    );
    assert_eq!(
        serde_json::to_string(&ValueMetric::Tokens).unwrap(),
        "\"tokens\""
    );
    assert_eq!(
        serde_json::to_string(&ValueMetric::Churn).unwrap(),
        "\"churn\""
    );
    assert_eq!(
        serde_json::to_string(&ValueMetric::Hotspot).unwrap(),
        "\"hotspot\""
    );
}

#[test]
fn context_output_serialized_values() {
    assert_eq!(
        serde_json::to_string(&ContextOutput::List).unwrap(),
        "\"list\""
    );
    assert_eq!(
        serde_json::to_string(&ContextOutput::Bundle).unwrap(),
        "\"bundle\""
    );
    assert_eq!(
        serde_json::to_string(&ContextOutput::Json).unwrap(),
        "\"json\""
    );
}

#[test]
fn near_dup_scope_serialized_values() {
    assert_eq!(
        serde_json::to_string(&NearDupScope::Module).unwrap(),
        "\"module\""
    );
    assert_eq!(
        serde_json::to_string(&NearDupScope::Lang).unwrap(),
        "\"lang\""
    );
    assert_eq!(
        serde_json::to_string(&NearDupScope::Global).unwrap(),
        "\"global\""
    );
}

#[test]
fn diff_range_mode_serialized_values() {
    assert_eq!(
        serde_json::to_string(&DiffRangeMode::TwoDot).unwrap(),
        "\"two-dot\""
    );
    assert_eq!(
        serde_json::to_string(&DiffRangeMode::ThreeDot).unwrap(),
        "\"three-dot\""
    );
}

#[test]
fn handoff_preset_serialized_values() {
    assert_eq!(
        serde_json::to_string(&HandoffPreset::Minimal).unwrap(),
        "\"minimal\""
    );
    assert_eq!(
        serde_json::to_string(&HandoffPreset::Standard).unwrap(),
        "\"standard\""
    );
    assert_eq!(
        serde_json::to_string(&HandoffPreset::Risk).unwrap(),
        "\"risk\""
    );
    assert_eq!(
        serde_json::to_string(&HandoffPreset::Deep).unwrap(),
        "\"deep\""
    );
}

#[test]
fn sensor_format_serialized_values() {
    assert_eq!(
        serde_json::to_string(&SensorFormat::Json).unwrap(),
        "\"json\""
    );
    assert_eq!(serde_json::to_string(&SensorFormat::Md).unwrap(), "\"md\"");
}

// =========================================================================
// Case sensitivity: uppercase variants should NOT parse
// =========================================================================

#[test]
fn uppercase_diff_format_fails() {
    let result: Result<DiffFormat, _> = serde_json::from_str("\"MD\"");
    assert!(result.is_err());
    let result: Result<DiffFormat, _> = serde_json::from_str("\"Json\"");
    assert!(result.is_err());
}

#[test]
fn uppercase_color_mode_fails() {
    let result: Result<ColorMode, _> = serde_json::from_str("\"AUTO\"");
    assert!(result.is_err());
    let result: Result<ColorMode, _> = serde_json::from_str("\"Always\"");
    assert!(result.is_err());
}

#[test]
fn uppercase_handoff_preset_fails() {
    let result: Result<HandoffPreset, _> = serde_json::from_str("\"Risk\"");
    assert!(result.is_err());
    let result: Result<HandoffPreset, _> = serde_json::from_str("\"DEEP\"");
    assert!(result.is_err());
}

#[test]
fn uppercase_context_strategy_fails() {
    let result: Result<ContextStrategy, _> = serde_json::from_str("\"Greedy\"");
    assert!(result.is_err());
}

#[test]
fn uppercase_near_dup_scope_fails() {
    let result: Result<NearDupScope, _> = serde_json::from_str("\"Global\"");
    assert!(result.is_err());
}

// =========================================================================
// Round-trip for enums not covered in properties.rs
// =========================================================================

macro_rules! roundtrip {
    ($name:ident, $type:ty, $variants:expr) => {
        #[test]
        fn $name() {
            for variant in $variants {
                let json = serde_json::to_string(&variant).expect("serialize");
                let back: $type = serde_json::from_str(&json).expect("deserialize");
                assert_eq!(variant, back, "Round-trip failed for {:?}", variant);
            }
        }
    };
}

roundtrip!(
    diff_format_roundtrip,
    DiffFormat,
    [DiffFormat::Md, DiffFormat::Json]
);

roundtrip!(
    color_mode_roundtrip,
    ColorMode,
    [ColorMode::Auto, ColorMode::Always, ColorMode::Never]
);

roundtrip!(
    cockpit_format_roundtrip,
    CockpitFormat,
    [
        CockpitFormat::Json,
        CockpitFormat::Md,
        CockpitFormat::Sections
    ]
);

roundtrip!(
    gate_format_roundtrip,
    GateFormat,
    [GateFormat::Text, GateFormat::Json]
);

roundtrip!(
    context_strategy_roundtrip,
    ContextStrategy,
    [ContextStrategy::Greedy, ContextStrategy::Spread]
);

roundtrip!(
    value_metric_roundtrip,
    ValueMetric,
    [
        ValueMetric::Code,
        ValueMetric::Tokens,
        ValueMetric::Churn,
        ValueMetric::Hotspot
    ]
);

roundtrip!(
    context_output_roundtrip,
    ContextOutput,
    [
        ContextOutput::List,
        ContextOutput::Bundle,
        ContextOutput::Json
    ]
);

roundtrip!(
    near_dup_scope_roundtrip,
    NearDupScope,
    [
        NearDupScope::Module,
        NearDupScope::Lang,
        NearDupScope::Global
    ]
);

roundtrip!(
    diff_range_mode_roundtrip,
    DiffRangeMode,
    [DiffRangeMode::TwoDot, DiffRangeMode::ThreeDot]
);

roundtrip!(
    handoff_preset_roundtrip,
    HandoffPreset,
    [
        HandoffPreset::Minimal,
        HandoffPreset::Standard,
        HandoffPreset::Risk,
        HandoffPreset::Deep
    ]
);

roundtrip!(
    sensor_format_roundtrip,
    SensorFormat,
    [SensorFormat::Json, SensorFormat::Md]
);

// =========================================================================
// Unknown string values rejected
// =========================================================================

#[test]
fn unknown_diff_format_fails() {
    let result: Result<DiffFormat, _> = serde_json::from_str("\"yaml\"");
    assert!(result.is_err());
}

#[test]
fn unknown_handoff_preset_fails() {
    let result: Result<HandoffPreset, _> = serde_json::from_str("\"ultra\"");
    assert!(result.is_err());
}

#[test]
fn unknown_context_output_fails() {
    let result: Result<ContextOutput, _> = serde_json::from_str("\"stream\"");
    assert!(result.is_err());
}

#[test]
fn unknown_sensor_format_fails() {
    let result: Result<SensorFormat, _> = serde_json::from_str("\"xml\"");
    assert!(result.is_err());
}
