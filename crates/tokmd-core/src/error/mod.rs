//! Structured error types for binding-friendly API.
//!
//! These error types are designed to be easily converted to JSON
//! for FFI boundaries while providing rich error information.

mod classify;
mod code;
mod envelope;
mod legacy;
mod tokmd;

pub use code::ErrorCode;
pub use envelope::{ErrorDetails, ResponseEnvelope};
pub use legacy::ErrorResponse;
pub use tokmd::TokmdError;

#[cfg(test)]
mod tests;
