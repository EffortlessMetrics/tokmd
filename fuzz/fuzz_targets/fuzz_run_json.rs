//! Fuzz target for the FFI `run_json` entrypoint.
//!
//! Uses a structured input model to improve mode and payload coverage for the
//! single-entrypoint FFI function while preserving adversarial noise inputs.

#![no_main]

use libfuzzer_sys::{arbitrary, arbitrary::Arbitrary, fuzz_target};
use serde_json::Value;
use tokmd_core::ffi::run_json;

/// Cap generated payload size to keep iterations fast.
const MAX_ARGS_SIZE: usize = 64 * 1024; // 64 KB

#[derive(Arbitrary, Debug)]
enum ModeHint {
    Lang,
    Module,
    Export,
    Analyze,
    Diff,
    Version,
    RandomAscii,
    Empty,
}

#[derive(Arbitrary, Debug)]
struct Input {
    mode: ModeHint,
    suffix: String,
    args_template: ArgsTemplate,
    noise: Vec<u8>,
}

#[derive(Arbitrary, Debug)]
enum ArgsTemplate {
    EmptyObject,
    EmptyArray,
    Null,
    MinimalPath,
    MinimalDiff,
    InvalidJsonLike,
    NoiseOnly,
}

fn mode_string(mode: ModeHint, suffix: String) -> String {
    let base = match mode {
        ModeHint::Lang => "lang",
        ModeHint::Module => "module",
        ModeHint::Export => "export",
        ModeHint::Analyze => "analyze",
        ModeHint::Diff => "diff",
        ModeHint::Version => "version",
        ModeHint::RandomAscii => "tokmd",
        ModeHint::Empty => "",
    };

    if suffix.is_empty() {
        base.to_owned()
    } else {
        format!("{base}{suffix}")
    }
}

fn args_json(template: ArgsTemplate, noise: Vec<u8>) -> String {
    match template {
        ArgsTemplate::EmptyObject => "{}".to_owned(),
        ArgsTemplate::EmptyArray => "[]".to_owned(),
        ArgsTemplate::Null => "null".to_owned(),
        ArgsTemplate::MinimalPath => {
            r#"{"path":".","format":"json","children":"collapse"}"#.to_owned()
        }
        ArgsTemplate::MinimalDiff => {
            r#"{"left":"./a.json","right":"./b.json"}"#.to_owned()
        }
        ArgsTemplate::InvalidJsonLike => "{\"path\":\".\",\"format\":".to_owned(),
        ArgsTemplate::NoiseOnly => {
            let mut payload = noise;
            payload.truncate(MAX_ARGS_SIZE);
            String::from_utf8_lossy(&payload).to_string()
        }
    }
}

fuzz_target!(|input: Input| {
    let mode = mode_string(input.mode, input.suffix);
    let args_json = args_json(input.args_template, input.noise);

    // Call the FFI entrypoint — must never panic.
    let result = run_json(&mode, &args_json);

    // Invariant: result is always valid JSON.
    let envelope: Value =
        serde_json::from_str(&result).expect("run_json must always return valid JSON");

    // Invariant: envelope always contains an "ok" boolean field.
    assert!(
        envelope.get("ok").and_then(Value::as_bool).is_some(),
        "envelope must have boolean 'ok' field, got: {}",
        result
    );
});
