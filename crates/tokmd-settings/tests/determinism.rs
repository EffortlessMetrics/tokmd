use std::collections::BTreeMap;
use tokmd_settings::{TomlConfig, ViewProfile};

#[test]
fn test_toml_config_determinism() {
    let mut view = BTreeMap::new();
    // Insert in reverse order to test sorting
    view.insert("zebra".to_string(), ViewProfile::default());
    view.insert("beta".to_string(), ViewProfile::default());
    view.insert("alpha".to_string(), ViewProfile::default());

    let config = TomlConfig {
        view,
        ..Default::default()
    };

    let json = serde_json::to_string(&config).expect("failed to serialize");

    // Check view keys order
    let p_alpha = json.find("\"alpha\":").expect("alpha profile missing");
    let p_beta = json.find("\"beta\":").expect("beta profile missing");
    let p_zebra = json.find("\"zebra\":").expect("zebra profile missing");

    assert!(p_alpha < p_beta, "alpha should be before beta");
    assert!(p_beta < p_zebra, "beta should be before zebra");
}
