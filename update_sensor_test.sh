#!/bin/bash
sed -i 's/fn render_sensor_md_includes_findings_and_gates() {/fn render_sensor_md_includes_findings_and_gates() -> Result<(), Box<dyn std::error::Error>> {/g' crates/tokmd/src/commands/sensor.rs
sed -i 's/"gates": serde_json::to_value(gates).unwrap(),/"gates": serde_json::to_value(gates)?,/g' crates/tokmd/src/commands/sensor.rs

# We need to add Ok(()) to the end of the test.
# The test ends with:
#        assert!(md.contains("### Gates (warn)"));
#        assert!(md.contains("mutation"));
#    }
# We'll use perl to find the end of the function and insert Ok(()).
perl -0777 -pi -e 's/(assert!\(md.contains\("mutation"\)\);\n    )\}/$1    Ok\(\)\n\}/s' crates/tokmd/src/commands/sensor.rs
