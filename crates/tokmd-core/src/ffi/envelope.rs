//! Response-envelope conversion for the FFI JSON entrypoint.
//!
//! This module keeps the binding-facing JSON envelope in one place while
//! `ffi/mod.rs` owns the public `run_json` symbol.

use serde_json::Value;

use crate::error::{ResponseEnvelope, TokmdError};

pub(super) fn json_response(result: Result<Value, TokmdError>) -> String {
    match result {
        Ok(data) => ResponseEnvelope::success(data).to_json(),
        Err(err) => ResponseEnvelope::error(&err).to_json(),
    }
}
