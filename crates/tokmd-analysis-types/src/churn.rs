use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveChurnReport {
    pub per_module: BTreeMap<String, ChurnTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnTrend {
    pub slope: f64,
    pub r2: f64,
    pub recent_change: i64,
    pub classification: TrendClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendClass {
    Rising,
    Flat,
    Falling,
}
