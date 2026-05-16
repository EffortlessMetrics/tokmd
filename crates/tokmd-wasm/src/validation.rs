use tokmd_core::error::TokmdError;

#[cfg(feature = "analysis")]
pub(crate) fn validate_analyze_args_json(args_json: &str) -> Result<(), TokmdError> {
    let args: serde_json::Value =
        serde_json::from_str(args_json).map_err(TokmdError::invalid_json)?;
    let obj = args.get("analyze").unwrap_or(&args);

    match obj.get("preset").and_then(serde_json::Value::as_str) {
        Some(preset) if tokmd_core::supports_rootless_in_memory_analyze_preset(preset) => Ok(()),
        Some(preset) => Err(TokmdError::not_implemented(format!(
            "tokmd-wasm currently supports analyze only with preset=\"receipt\" or preset=\"estimate\" for in-memory inputs; got {preset:?}"
        ))),
        None => Ok(()),
    }
}

pub(crate) fn validate_mode_args_json(mode: &str, args_json: &str) -> Result<(), TokmdError> {
    #[cfg(feature = "analysis")]
    if mode == "analyze" {
        return validate_analyze_args_json(args_json);
    }

    let _ = (mode, args_json);
    Ok(())
}
