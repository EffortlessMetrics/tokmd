use std::collections::BTreeMap;
use tokmd_config::{Profile, TomlConfig, UserConfig, ViewProfile};

#[test]
fn test_user_config_determinism() {
    let mut profiles = BTreeMap::new();
    // Insert in reverse order to test sorting
    profiles.insert("zebra".to_string(), Profile::default());
    profiles.insert("beta".to_string(), Profile::default());
    profiles.insert("alpha".to_string(), Profile::default());

    let mut repos = BTreeMap::new();
    repos.insert("owner/zebra".to_string(), "profile1".to_string());
    repos.insert("owner/beta".to_string(), "profile3".to_string());
    repos.insert("owner/alpha".to_string(), "profile2".to_string());

    let config = UserConfig { profiles, repos };

    let json = serde_json::to_string(&config).expect("failed to serialize");

    // We expect "alpha" < "beta" < "zebra"
    // Find indices of keys in the JSON string

    // Check profiles keys
    let p_alpha = json.find("\"alpha\":").expect("alpha profile missing");
    let p_beta = json.find("\"beta\":").expect("beta profile missing");
    let p_zebra = json.find("\"zebra\":").expect("zebra profile missing");

    assert!(
        p_alpha < p_beta,
        "profiles: alpha ({}) should be before beta ({})",
        p_alpha,
        p_beta
    );
    assert!(
        p_beta < p_zebra,
        "profiles: beta ({}) should be before zebra ({})",
        p_beta,
        p_zebra
    );

    // Check repos keys
    let r_alpha = json.find("\"owner/alpha\":").expect("alpha repo missing");
    let r_beta = json.find("\"owner/beta\":").expect("beta repo missing");
    let r_zebra = json.find("\"owner/zebra\":").expect("zebra repo missing");

    assert!(
        r_alpha < r_beta,
        "repos: alpha ({}) should be before beta ({})",
        r_alpha,
        r_beta
    );
    assert!(
        r_beta < r_zebra,
        "repos: beta ({}) should be before zebra ({})",
        r_beta,
        r_zebra
    );
}

#[test]
fn test_toml_config_determinism() {
    let mut config = TomlConfig::default();
    // Insert in reverse order to test sorting
    config
        .view
        .insert("zebra".to_string(), ViewProfile::default());
    config
        .view
        .insert("beta".to_string(), ViewProfile::default());
    config
        .view
        .insert("alpha".to_string(), ViewProfile::default());
    config
        .view
        .insert("gamma".to_string(), ViewProfile::default());
    config
        .view
        .insert("delta".to_string(), ViewProfile::default());

    let json = serde_json::to_string(&config).expect("failed to serialize");

    // We expect "alpha" < "beta" < "delta" < "gamma" < "zebra"
    // Find indices of keys in the JSON string

    // Check view keys
    let p_alpha = json.find("\"alpha\":").expect("alpha view missing");
    let p_beta = json.find("\"beta\":").expect("beta view missing");
    let p_delta = json.find("\"delta\":").expect("delta view missing");
    let p_gamma = json.find("\"gamma\":").expect("gamma view missing");
    let p_zebra = json.find("\"zebra\":").expect("zebra view missing");

    assert!(p_alpha < p_beta, "alpha < beta");
    assert!(p_beta < p_delta, "beta < delta");
    assert!(p_delta < p_gamma, "delta < gamma");
    assert!(p_gamma < p_zebra, "gamma < zebra");
}
