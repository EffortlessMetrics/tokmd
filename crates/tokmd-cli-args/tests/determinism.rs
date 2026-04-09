use std::collections::BTreeMap;
use tokmd_cli_args::{Profile, UserConfig};

#[test]
fn test_user_config_determinism() -> Result<(), Box<dyn std::error::Error>> {
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

    let json = serde_json::to_string(&config)?;

    // We expect "alpha" < "beta" < "zebra"
    // Find indices of keys in the JSON string

    // Check profiles keys
    let p_alpha = json.find("\"alpha\":").ok_or("alpha profile missing")?;
    let p_beta = json.find("\"beta\":").ok_or("beta profile missing")?;
    let p_zebra = json.find("\"zebra\":").ok_or("zebra profile missing")?;

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
    let r_alpha = json.find("\"owner/alpha\":").ok_or("alpha repo missing")?;
    let r_beta = json.find("\"owner/beta\":").ok_or("beta repo missing")?;
    let r_zebra = json.find("\"owner/zebra\":").ok_or("zebra repo missing")?;

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
    Ok(())
}
