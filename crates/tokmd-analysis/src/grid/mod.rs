#![doc = "Feature matrix and warning catalog for tokmd-analysis preset execution."]

mod disabled_feature;
mod presets;

pub use disabled_feature::DisabledFeature;
#[cfg(test)]
pub use presets::PresetGridRow;
pub use presets::{
    PRESET_GRID, PRESET_KINDS, PresetKind, PresetPlan, preset_plan_for, preset_plan_for_name,
};

#[cfg(test)]
mod tests;
