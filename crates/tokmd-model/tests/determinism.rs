use std::fs;
use tokei::{Config, Languages};
use tokmd_model::{create_export_data, create_lang_report, create_module_report};
use tokmd_types::{ChildIncludeMode, ChildrenMode};

fn simple_shuffle<T>(vec: &mut [T], seed: u64) {
    let len = vec.len();
    if len <= 1 {
        return;
    }
    let mut rng = seed;
    for i in (1..len).rev() {
        // Linear Congruential Generator
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = (rng as usize) % (i + 1);
        vec.swap(i, j);
    }
}

// Helper to shuffle reports in languages
fn shuffle_languages(languages: &mut Languages, seed: u64) {
    for (_lang_type, lang) in languages.iter_mut() {
        simple_shuffle(&mut lang.reports, seed);
    }
}

#[test]
fn test_aggregation_determinism() {
    // 1. Setup temp dir with files
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();

    // Create a rich structure of files to ensure multiple aggregation paths
    let files = vec![
        // Root files
        ("src/main.rs", "fn main() { println!(\"Hello\"); }"),
        ("src/lib.rs", "pub fn foo() {}"),
        ("src/utils.rs", "pub fn bar() {}"),
        ("src/common.rs", "pub fn baz() {}"),
        // Deep nesting
        ("src/modules/mod.rs", "pub mod a; pub mod b;"),
        ("src/modules/a.rs", "pub fn a() { /* comment */ }"),
        ("src/modules/b.rs", "pub fn b() { let x = 1; }"),
        // Other languages
        (
            "Cargo.toml",
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        ),
        ("README.md", "# Test\n\nThis is a test."),
        ("build.rs", "fn main() {}"),
        // Tests
        ("tests/foo.rs", "fn test_foo() {}"),
        ("tests/bar.rs", "fn test_bar() {}"),
        ("tests/baz.rs", "fn test_baz() {}"),
        // Scripts
        ("scripts/run.sh", "#!/bin/bash\necho hello"),
        ("scripts/setup.py", "print('setup')"),
    ];

    for (path, content) in files {
        let p = root.join(path);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, content).unwrap();
    }

    // 2. Scan with tokei
    let mut languages = Languages::new();
    let config = Config::default();
    languages.get_statistics(&[root.to_path_buf()], &[], &config);

    // Verify we have enough data
    assert!(!languages.is_empty());

    // 3. Define params
    let module_roots = vec![
        "src".to_string(),
        "tests".to_string(),
        "scripts".to_string(),
    ];
    let module_depth = 1;
    let top = 10;

    // 4. Baseline Runs
    // Generate baseline reports from the initial scan order (whatever tokei returned)
    let base_module = create_module_report(
        &languages,
        &module_roots,
        module_depth,
        ChildIncludeMode::Separate,
        top,
    );
    let base_lang = create_lang_report(&languages, top, true, ChildrenMode::Separate);
    let base_export = create_export_data(
        &languages,
        &module_roots,
        module_depth,
        ChildIncludeMode::Separate,
        None,
        0,
        100,
    );

    // 5. Shuffle and Compare
    // Run 50 iterations with different shuffle seeds
    for seed in 1..=50 {
        // Tokei::Languages does not implement Clone directly (it derefs to BTreeMap),
        // so calling .clone() returns a BTreeMap, which is not a Languages struct.
        // We must reconstruct the Languages struct manually.
        let mut shuffled = Languages::new();
        for (k, v) in languages.iter() {
            shuffled.insert(*k, v.clone());
        }

        shuffle_languages(&mut shuffled, seed);

        let mod_rep = create_module_report(
            &shuffled,
            &module_roots,
            module_depth,
            ChildIncludeMode::Separate,
            top,
        );
        let lang_rep = create_lang_report(&shuffled, top, true, ChildrenMode::Separate);
        let export = create_export_data(
            &shuffled,
            &module_roots,
            module_depth,
            ChildIncludeMode::Separate,
            None,
            0,
            100,
        );

        // Assert Module Report Determinism
        let base_json = serde_json::to_string_pretty(&base_module).unwrap();
        let shuf_json = serde_json::to_string_pretty(&mod_rep).unwrap();
        assert_eq!(
            base_json, shuf_json,
            "Seed {}: Module report mismatch. Aggregation must be independent of input order.",
            seed
        );

        // Assert Lang Report Determinism
        let base_l_json = serde_json::to_string_pretty(&base_lang).unwrap();
        let shuf_l_json = serde_json::to_string_pretty(&lang_rep).unwrap();
        assert_eq!(
            base_l_json, shuf_l_json,
            "Seed {}: Lang report mismatch. Aggregation must be independent of input order.",
            seed
        );

        // Assert Export Data Determinism
        let base_e_json = serde_json::to_string_pretty(&base_export).unwrap();
        let shuf_e_json = serde_json::to_string_pretty(&export).unwrap();
        assert_eq!(
            base_e_json, shuf_e_json,
            "Seed {}: Export data mismatch. Sorting must be stable.",
            seed
        );
    }
}
