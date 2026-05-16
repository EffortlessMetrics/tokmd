#[cfg(feature = "analysis")]
const ROOTLESS_ANALYZE_PRESETS: &[&str] = &["receipt", "estimate"];

pub(crate) fn capabilities_json() -> String {
    #[cfg(feature = "analysis")]
    let modes = vec!["lang", "module", "export", "analyze"];
    #[cfg(not(feature = "analysis"))]
    let modes = vec!["lang", "module", "export"];

    #[cfg(feature = "analysis")]
    let rootless_presets = ROOTLESS_ANALYZE_PRESETS;
    #[cfg(not(feature = "analysis"))]
    let rootless_presets: &[&str] = &[];

    serde_json::json!({
        "modes": modes,
        "analyze": {
            "rootlessPresets": rootless_presets,
        },
    })
    .to_string()
}
