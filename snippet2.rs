fn test_sensor_report_example_fail_validates() -> Result<()> {
    let schema = load_sensor_report_schema()?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| anyhow::anyhow!("Failed to compile schema: {}", e))?;

    // Read the fail example from contracts
    let example_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("contracts")
        .join("sensor.report.v1")
        .join("examples")
        .join("fail.json");

    let content = std::fs::read_to_string(&example_path)
        .with_context(|| format!("Failed to read {}", example_path.display()))?;
    let json: Value = serde_json::from_str(&content)?;

    if !validator.is_valid(&json) {
