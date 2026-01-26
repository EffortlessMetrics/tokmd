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
    cmd.current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

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
