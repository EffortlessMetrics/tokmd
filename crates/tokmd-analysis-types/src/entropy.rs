use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyReport {
    pub suspects: Vec<EntropyFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyFinding {
    pub path: String,
    pub module: String,
    pub entropy_bits_per_byte: f32,
    pub sample_bytes: u32,
    pub class: EntropyClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntropyClass {
    Low,
    Normal,
    Suspicious,
    High,
}
