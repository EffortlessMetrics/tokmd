use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicClouds {
    pub per_module: BTreeMap<String, Vec<TopicTerm>>,
    pub overall: Vec<TopicTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicTerm {
    pub term: String,
    pub score: f64,
    pub tf: u32,
    pub df: u32,
}
