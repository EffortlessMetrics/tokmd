use std::fs;
use std::path::Path;

use tempfile::TempDir;
use tokei::{Config, Languages};
use tokmd_model::{InMemoryRowInput, collect_file_rows, collect_in_memory_file_rows};
use tokmd_types::ChildIncludeMode;

fn write_file(root: &Path, rel: &str, contents: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, contents).expect("write fixture");
}

#[test]
fn collect_in_memory_file_rows_matches_filesystem_rows_with_embedded_children() {
    let tmp = TempDir::new().expect("temp dir");
    let html = "<html>\n<script>\nconst value = 1;\n</script>\n</html>\n";
    let rust = "pub fn alpha() -> usize { 1 }\n";
    write_file(tmp.path(), "web/index.html", html);
    write_file(tmp.path(), "src/lib.rs", rust);

    let mut languages = Languages::new();
    languages.get_statistics(&[tmp.path().to_path_buf()], &[], &Config::default());
    let expected = collect_file_rows(
        &languages,
        &[],
        1,
        ChildIncludeMode::Separate,
        Some(tmp.path()),
    );

    let inputs = vec![
        InMemoryRowInput::new(Path::new("web/index.html"), html.as_bytes()),
        InMemoryRowInput::new(Path::new("src/lib.rs"), rust.as_bytes()),
    ];
    let actual = collect_in_memory_file_rows(
        &inputs,
        &[],
        1,
        ChildIncludeMode::Separate,
        &Config::default(),
    );

    assert_eq!(actual, expected);
}
