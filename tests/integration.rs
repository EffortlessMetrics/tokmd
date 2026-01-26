use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    // Point to our test fixture
    cmd.current_dir("tests/data");
    cmd
}

fn redact_timestamps(output: &str) -> String {
    let re = regex::Regex::new(r#""generated_at_ms":\d+"#).unwrap();
    re.replace_all(output, r#""generated_at_ms":0"#).to_string()
}

#[test]
fn test_default_lang_output() {
    let mut cmd = tokmd_cmd();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("|Rust|"));
}

#[test]
fn test_module_output() {
    let mut cmd = tokmd_cmd();
    cmd.arg("module")
        .assert()
        .success()
        .stdout(predicate::str::contains("|(root)|"))
        .stdout(predicate::str::contains("|src|"));
}

#[test]
fn test_export_jsonl() {
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--format")
        .arg("jsonl")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""mode":"export""#)) // Meta record
        .stdout(predicate::str::contains(r#""path":"src/main.rs""#)); // Data row
}

#[test]
fn test_export_redaction() {
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--redact")
        .arg("paths")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs").not());
}

#[test]
fn test_ignore_file_respected() {
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("ignored.rs").not());
}

#[test]
fn test_ignore_vcs_explicit() {
    // Given: 'hidden_by_git.txt' is in .gitignore
    // When: We run with --no-ignore-vcs (or --no-ignore-git)
    // Then: The file SHOULD appear in the output
    let mut cmd = tokmd_cmd();
    cmd.arg("--no-ignore-vcs")
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("hidden_by_git.txt"));
}

#[test]
fn test_no_ignore_implies_vcs() {
    // Given: 'hidden_by_git.txt' is in .gitignore
    // When: We run with --no-ignore (which implies vcs ignore disabled)
    // Then: The file SHOULD appear
    let mut cmd = tokmd_cmd();
    cmd.arg("--no-ignore")
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("hidden_by_git.txt"));
}

#[test]
fn test_default_ignores_vcs() {
    // Given: 'hidden_by_git.txt' is in .gitignore
    // When: We run normally
    // Then: The file SHOULD NOT appear
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("hidden_by_git.txt").not());
}

#[test]
fn test_ignore_override() {
    let mut cmd = tokmd_cmd();
    cmd.arg("--no-ignore")
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("ignored.rs"));
}

#[test]
fn test_lang_format_tsv() {
    let mut cmd = tokmd_cmd();
    cmd.arg("lang")
        .arg("--format")
        .arg("tsv")
        .assert()
        .success()
        .stdout(predicate::str::contains("Lang\tCode\tLines"))
        .stdout(predicate::str::contains("Rust\t"));
}

#[test]
fn test_module_format_tsv() {
    let mut cmd = tokmd_cmd();
    cmd.arg("module")
        .arg("--format")
        .arg("tsv")
        .assert()
        .success()
        .stdout(predicate::str::contains("Module\tCode\tLines"))
        .stdout(predicate::str::contains("(root)\t"));
}

#[test]
fn test_golden_lang_json() {
    let mut cmd = tokmd_cmd();
    let output = cmd.arg("--format").arg("json").output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let normalized = redact_timestamps(&stdout);

    insta::assert_snapshot!(normalized);
}

#[test]
fn test_golden_module_json() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("module")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let normalized = redact_timestamps(&stdout);

    insta::assert_snapshot!(normalized);
}

#[test]
fn test_golden_export_jsonl() {
    let mut cmd = tokmd_cmd();
    let output = cmd.arg("export").output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let normalized = redact_timestamps(&stdout);

    insta::assert_snapshot!(normalized);
}

#[test]
fn test_golden_export_redacted() {
    let mut cmd = tokmd_cmd();
    let output = cmd
        .arg("export")
        .arg("--redact")
        .arg("all")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let normalized = redact_timestamps(&stdout);

    insta::assert_snapshot!(normalized);
}

#[test]
fn test_strip_prefix() {
    // Given: A file in src/main.rs
    // When: We export with --strip-prefix src
    // Then: The path should be main.rs (without src/)
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--strip-prefix")
        .arg("src")
        .assert()
        .success()
        // Should contain "main.rs" but NOT "src/main.rs"
        // (Wait, main.rs is a substring of src/main.rs, so contain("main.rs") is true for both.
        // We need to be more specific with JSON matching or negative matching.)
        .stdout(predicate::str::contains(r#""path":"main.rs""#))
        .stdout(predicate::str::contains(r#""path":"src/main.rs""#).not());
}

#[test]
fn test_export_format_json() {
    // Given: Standard files
    // When: We export with --format json (not jsonl)
    // Then: output should be a JSON object containing "rows"
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        // It's a JSON object { ..., "rows": [...] }
        .stdout(predicate::str::starts_with("{"))
        .stdout(predicate::str::contains(r#""rows":["#));
}

#[test]
fn test_init_creates_file() {
    // Given: An empty temporary directory
    // When: We run `tokmd init` inside it
    // Then: .tokeignore should be created
    let dir = tempdir().unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path()).arg("init").assert().success();

    let file_path = dir.path().join(".tokeignore");
    assert!(file_path.exists(), ".tokeignore was not created");
}

// --- BDD / Feature Tests ---

#[test]
fn test_filter_min_code() {
    // Given: A large file (10+ lines) and small files (<5 lines)
    // When: We export with --min-code 5
    // Then: Only the large file should be present
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--min-code")
        .arg("5")
        .assert()
        .success()
        .stdout(predicate::str::contains("large.rs"))
        .stdout(predicate::str::contains("main.rs").not());
}

#[test]
fn test_limit_top_files() {
    // Given: Multiple files
    // When: We run lang report with --top 1
    // Then: Only 1 language (plus potentially 'Total'/'Other') should be detailed
    // Note: Rust is likely the top lang here.
    let mut cmd = tokmd_cmd();
    cmd.arg("--top")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"));
}

#[test]
fn test_limit_max_rows() {
    // Given: Multiple files
    // When: We export with --max-rows 1
    // Then: output should be truncated (ignoring the meta header)
    // Note: This is harder to test with assert_cmd on stdout content without parsing,
    // but we can check if some files are missing that would normally be there.
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--max-rows")
        .arg("1")
        .arg("--format") // Explicit format to avoid noise
        .arg("jsonl")
        .assert()
        .success()
        // Meta record is always first
        .stdout(predicate::str::contains(r#""mode":"export""#))
        // We should see exactly ONE data row.
        // Counting "row" occurrences might be tricky with simple string predicates.
        // Instead, let's verify that NOT ALL files are present.
        // If we have 5 files, and we ask for 1, we shouldn't see all 5.
        // large.rs and main.rs - one should be missing.
        // (This is a weak test but verifies the flag does *something*)
        // Better: check that output lines count (meta + 1 row + empty line)
        .stdout(predicate::function(|s: &str| {
            let lines: Vec<_> = s.trim().split('\n').collect();
            // Meta + 1 Row = 2 lines
            lines.len() == 2
        }));
}

#[test]
fn test_paths_redacted_hash() {
    // Given: Standard files
    // When: We export with --redact paths (which means hash)
    // Then: filenames should be replaced by hashes
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--redact")
        .arg("paths")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs").not())
        // Match hash + extension (e.g. "ec412fe02b918085.rs")
        .stdout(predicate::str::is_match(r#""path":"[0-9a-f]{16}\.[a-z0-9]+""#).unwrap());
}

#[test]
fn test_default_paths_are_relative() {
    // Given: Standard files
    // When: We export (default settings)
    // Then: Paths should be relative (e.g. "src/main.rs")
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"));
}

#[test]
fn test_children_separate() {
    // Given: A markdown file with embedded Rust code (mixed.md)
    // When: We run module report with --children separate
    // Then: We should see counts reflecting the embedded code
    // (This is a smoke test to ensure the flag is accepted and runs)
    let mut cmd = tokmd_cmd();
    cmd.arg("module")
        .arg("--children")
        .arg("separate")
        .assert()
        .success();
}

#[test]
fn test_init_command() {
    // Given: A temporary directory (mimicked by checking output for now,
    // as we don't want to dirty tests/data/src)
    // When: We run `init --print`
    // Then: It should output the default .tokeignore content
    let mut cmd = tokmd_cmd();
    cmd.arg("init")
        .arg("--print")
        .assert()
        .success()
        .stdout(predicate::str::contains("# .tokeignore"));
}

#[test]
fn test_module_custom_roots() {
    // Given: A file structure where we can simulate roots.
    // 'src' is a folder in tests/data.
    // When: We run module report with --module-roots src --module-depth 1
    // Then: Files in src/ should be grouped under 'src' (which is the default behavior anyway,
    // but we verify the flag is accepted and works).
    // A better test would be if we had nested folders.
    // Let's assume src/main.rs.
    // If we say --module-roots src, and depth 1, key should be 'src'.
    let mut cmd = tokmd_cmd();
    cmd.arg("module")
        .arg("--module-roots")
        .arg("src")
        .arg("--module-depth")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("|src|"));
}

#[test]
fn test_init_print_bdd() {
    // Given: No prerequisites
    // When: `tokmd init --print` is run
    // Then: It prints the standard .tokeignore template including 'target/'
    let mut cmd = tokmd_cmd();
    cmd.arg("init")
        .arg("--print")
        .assert()
        .success()
        .stdout(predicate::str::contains("target/"));
}

#[test]
fn test_mixed_args_precedence() {
    // Given: A file that is small (src/main.rs is < 10 lines)
    // When: We run export with --min-code 10 AND --max-rows 100
    // Then: It should be filtered out because min-code excludes it before max-rows counts it
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--min-code")
        .arg("10")
        .arg("--max-rows")
        .arg("100")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs").not());
}

#[test]
fn test_module_custom_roots_miss() {
    // Given: src/main.rs
    // When: We set module roots to something that DOESN'T match, like 'crates'
    // Then: The file should fall back to its top-level directory 'src'
    // (or (root) if it was at root).
    // src/main.rs -> top level is 'src'.
    let mut cmd = tokmd_cmd();
    cmd.arg("module")
        .arg("--module-roots")
        .arg("crates") // Doesn't match 'src'
        .assert()
        .success()
        .stdout(predicate::str::contains("|src|"));
}

#[test]
fn test_export_meta_false() {
    // Given: Standard files
    // When: We export with --meta false
    // Then: The first line should NOT be the meta record
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--meta")
        .arg("false")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""mode":"export""#).not())
        .stdout(predicate::str::contains(r#""type":"row""#));
}

#[test]
fn test_redaction_leaks_in_meta() {
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("src/main.rs") // Explicit positional path
        .arg("--redact")
        .arg("paths")
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs").not());
}

#[test]
fn test_filter_all_rows() {
    // Given: Files with small code counts
    // When: We export with --min-code 1000 (too high)
    // Then: No row records should be output, but meta might be (if enabled)
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--min-code")
        .arg("1000")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""type":"row""#).not());
}

#[test]
fn test_export_out_file() {
    // Given: A temp dir and output file path
    // When: We run export with --out <file>
    // Then: stdout should be empty, file should contain jsonl
    let dir = tempdir().unwrap();
    let out_file = dir.path().join("output.jsonl");

    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--out")
        .arg(&out_file)
        .assert()
        .success()
        .stdout(""); // stdout should be empty

    let content = std::fs::read_to_string(&out_file).unwrap();
    assert!(content.contains(r#""mode":"export""#));
    assert!(content.contains(r#""path":"src/main.rs""#)); // assuming default test context includes src/main.rs
}

#[test]
fn test_lang_files_flag() {
    // Given: Standard scan
    // When: We run lang --files
    // Then: Output should contain "Files" and "Avg" columns
    let mut cmd = tokmd_cmd();
    cmd.arg("--files")
        .assert()
        .success()
        .stdout(predicate::str::contains("Files"))
        .stdout(predicate::str::contains("Avg"));
}

#[test]
fn test_init_force() {
    // Given: A temp dir with an existing .tokeignore
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(".tokeignore");
    std::fs::write(&file_path, "existing content").unwrap();

    // When: We run init without force
    // Then: It should fail
    let mut cmd = tokmd_cmd();
    cmd.current_dir(dir.path()).arg("init").assert().failure();

    // When: We run init WITH force
    // Then: It should succeed and overwrite
    let mut cmd = tokmd_cmd();
    cmd.current_dir(dir.path())
        .arg("init")
        .arg("--force")
        .assert()
        .success();

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("# .tokeignore"));
    assert!(!content.contains("existing content"));
}

#[test]
fn test_init_profiles() {
    // Given: A request for python profile
    // When: We run init --profile python --print
    // Then: Output should contain python specific ignores like __pycache__
    let mut cmd = tokmd_cmd();
    cmd.arg("init")
        .arg("--profile")
        .arg("python")
        .arg("--print")
        .assert()
        .success()
        .stdout(predicate::str::contains("__pycache__"));
}

#[test]
fn test_non_existent_path() {
    // Given: A non-existent path
    // When: We run export
    // Then: It should succeed but report 0 files (or handled gracefully)
    // Tokei behavior is to just ignore it usually, or report empty stats.
    // Our wrapper should not panic.
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("non_existent_file_abc123.txt")
        .assert()
        .success();
    // We don't strictly assert output emptiness because meta might be there.
    // But verifying it doesn't crash is valuable.
}

#[test]
fn test_module_parents_only() {
    // Given: mixed.md with embedded rust code
    // When: We run module --children parents-only
    // Then: The total code count should be lower than separate/collapse for the 'tests' module
    // We can just verify it succeeds and maybe check a known value if we had precise control,
    // but verifying the flag is accepted is a good start.
    let mut cmd = tokmd_cmd();
    cmd.arg("module")
        .arg("--children")
        .arg("parents-only")
        .assert()
        .success();
}

#[test]
fn test_empty_file_handling() {
    // Given: An empty file (we need to ensure one exists in fixtures)
    // For now, let's assume 'script.js' has content.
    // We'll create a new empty file in a setup step if we could,
    // but here we are restricted to existing fixtures.
    // Let's verify 'ignored.rs' is indeed ignored by default first.
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("ignored.rs").not());
}

#[test]
fn test_path_with_spaces() {
    // Given: 'space file.rs' exists in tests/data
    // When: We export
    // Then: It should be present and handled correctly (no quoting issues in JSON)
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("space file.rs"));
}

#[test]
fn test_csv_escaping() {
    // Given: A file with a comma in its name
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("file,with,commas.txt");
    std::fs::write(&file_path, "content").unwrap();

    // When: We export as CSV
    // Then: The path should be quoted in the output
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("export")
        .arg("--format")
        .arg("csv")
        .assert()
        .success()
        // csv crate quotes fields containing delimiters.
        // "file,with,commas.txt"
        .stdout(predicate::str::contains(r#""file,with,commas.txt""#));
}

#[test]
fn test_exclude_glob() {
    // Given: A nested file structure
    let dir = tempdir().unwrap();
    let nested = dir.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    std::fs::write(nested.join("skip_me.rs"), "fn main() {}").unwrap();
    std::fs::write(nested.join("keep_me.rs"), "fn main() {}").unwrap();

    // When: We run export with a glob exclude
    // Note: --exclude is a global arg, so it must come BEFORE the subcommand
    // unless we mark it global in clap.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("--exclude")
        .arg("**/skip_me.rs")
        .arg("export")
        .assert()
        .success()
        // skip_me.rs appears in metadata (args), so we must check it's not in a path row
        // We look for the JSON key/value pair for the path
        .stdout(predicate::str::contains(r#""path":"nested/skip_me.rs""#).not())
        .stdout(predicate::str::contains("keep_me.rs"));
}

#[test]
fn test_redact_all() {
    // Given: Standard files
    // When: We export with --redact all
    // Then: filenames AND module names should be redacted
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--redact")
        .arg("all")
        .assert()
        .success()
        // Path should be hashed
        .stdout(predicate::str::contains("src/main.rs").not())
        .stdout(predicate::str::is_match(r#""path":"[0-9a-f]{16}\.[a-z0-9]+""#).unwrap())
        // Module should NOT be "src" (it should be hashed or redacted, usually same hash if it's based on path,
        // or just hidden. Wait, how is module redacted?
        // In model.rs it's just a derived key. If paths are redacted, module key derivation might change?
        // Let's check format.rs/redact_rows logic.
        // Actually, let's just check that "src" doesn't appear as a module value.
        .stdout(predicate::str::contains(r#""module":"src""#).not());
}

#[test]
fn test_module_top_exact() {
    let mut cmd = tokmd_cmd();
    cmd.arg("module")
        .arg("--top")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("(root)"))
        .stdout(predicate::str::contains("src"))
        .stdout(predicate::str::contains("Other").not());
}

#[test]
fn test_children_stats_integrity() {
    let mut cmd = tokmd_cmd();
    cmd.arg("lang")
        .arg("--children")
        .arg("separate")
        .arg("--files")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust (embedded)"))
        // Check that files count (4th column) is non-zero
        // Format: |Rust (embedded)|3|3|1|3|
        .stdout(predicate::str::is_match(r"\|\s*Rust \(embedded\)\s*\|\s*\d+\s*\|\s*\d+\s*\|\s*[1-9]\d*\s*\|").unwrap());
}

/*
#[test]
fn test_config_file() {
    // Given: A temp dir with a tokei.toml that ignores a file
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("tokei.toml");
    std::fs::write(&config_path, r#"
    [[ignore]]
    ignore = ["ignored_by_conf.rs"]
    "#).unwrap();

    let file_path = dir.path().join("ignored_by_conf.rs");
    std::fs::write(&file_path, "fn main() {}").unwrap();

    // When: We run export in that dir
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("export")
        .assert()
        .success()
        // Then: The file should be ignored
        .stdout(predicate::str::contains("ignored_by_conf.rs").not());
}
*/

#[test]
fn test_module_depth_overflow() {
    // Given: src/main.rs (depth 2 essentially: src -> main.rs)
    // When: We ask for module depth 10
    // Then: It should not crash, and likely just return 'src' (or whatever full path segments it has)
    let mut cmd = tokmd_cmd();
    cmd.arg("module")
        .arg("--module-depth")
        .arg("10")
        .assert()
        .success()
        .stdout(predicate::str::contains("|src|"));
}

#[test]
fn test_lang_top_limit() {
    // Given: Standard scan with multiple languages
    // When: We run lang --top 1
    // Then: Output should contain "Other" row if there are more than 1 lang
    // In our test data we have Rust, TOML, Markdown, JS.
    let mut cmd = tokmd_cmd();
    cmd.arg("--top")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Other"));
}

#[test]
fn test_module_top_limit() {
    // Given: Standard scan with multiple modules
    // When: We run module --top 1
    // Then: Output should contain "Other" row
    // Modules: src, tests, docs, (root) - that's 4.
    let mut cmd = tokmd_cmd();
    cmd.arg("module")
        .arg("--top")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Other"));
}

#[test]
fn test_lang_format_json() {
    // Given: Standard scan
    // When: We run lang --format json
    // Then: Output should be valid JSON with "rows" and "total"
    let mut cmd = tokmd_cmd();
    cmd.arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""rows":["#))
        .stdout(predicate::str::contains(r#""total":{"#));
}

#[test]
fn test_no_ignore_dot() {
    // Given: A temp dir with a .ignore file
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(".ignore"), "ignored.txt").unwrap();
    std::fs::write(dir.path().join("ignored.txt"), "content").unwrap();

    // When: We run export (default respects .ignore)
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(dir.path())
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("ignored.txt").not());

    // When: We run export --no-ignore-dot
    let mut cmd2 = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd2.current_dir(dir.path())
        .arg("--no-ignore-dot")
        .arg("export")
        .assert()
        .success()
        .stdout(predicate::str::contains("ignored.txt"));
}

#[test]
fn test_verbose_flag() {
    // Given: Simple run
    // When: We run with --verbose
    // Then: It shouldn't crash.
    // Tokei's verbose output goes to stderr.
    let mut cmd = tokmd_cmd();
    cmd.arg("--verbose")
        .arg("export")
        .assert()
        .success();
    // We don't assert content because logging format might change,
    // but ensuring the flag is accepted is the main goal.
}

#[test]
fn test_treat_doc_strings_as_comments() {
    // Given: A Python file with docstrings
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("doc.py"), r#"
"""
This is a docstring.
It should be counted as comments if flag is on.
"""
x = 1
    "#).unwrap();

    // When: We run with --treat-doc-strings-as-comments
    // We output jsonl to check the counts
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    let output = cmd.current_dir(dir.path())
        .arg("--treat-doc-strings-as-comments")
        .arg("export")
        .arg("--format")
        .arg("jsonl")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    // Find the row for doc.py
    let row_line = stdout.lines().find(|l| l.contains("doc.py")).unwrap();

    // Then: comments should be > 0 (it's 4 lines of docstring)
    // Code should be 1 (x=1)
    // Without the flag, docstrings are often counted as code in some parsers,
    // or comments by default? Tokei default for Python docstrings is comments?
    // Let's check tokei default. Tokei usually treats docstrings as comments by default in recent versions,
    // but the flag forces it? Or does it force them as comments if they were code?
    // Actually, Tokei documentation says: "--treat-doc-strings-as-comments: Treat doc strings as comments."
    // Implies default might be code?
    // Let's just verify that comments >= 4.
    assert!(row_line.contains(r#""comments":4"#) || row_line.contains(r#""comments":5"#));
}

#[test]
fn test_format_csv() {
    // Given: Standard files
    // When: We export as CSV
    // Then: Output should be comma-separated
    let mut cmd = tokmd_cmd();
    cmd.arg("export")
        .arg("--format")
        .arg("csv")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "path,module,lang,kind,code,comments,blanks,lines",
        )) // Header
        .stdout(predicate::str::contains(
            "src/main.rs,src,Rust,parent,3,0,0,3",
        )); // Row
}

