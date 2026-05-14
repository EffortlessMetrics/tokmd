use crate::cli::AstShadowCompareArgs;
use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use tokmd_analysis::ast::{
    AstLanguage, ShadowFileInput, ShadowLandmark, build_shadow_artifacts, normalize_shadow_path,
    write_shadow_artifacts,
};

pub fn run(args: AstShadowCompareArgs) -> Result<()> {
    let root = std::env::current_dir().context("resolve current directory")?;
    run_with_root(args, &root)
}

fn run_with_root(args: AstShadowCompareArgs, root: &Path) -> Result<()> {
    let inputs = collect_inputs(&args.paths, root)?;
    let shadow_inputs = inputs
        .iter()
        .map(|input| ShadowFileInput {
            path: input.path.as_str(),
            language: AstLanguage::Rust,
            source: input.source.as_str(),
            heuristic_landmarks: &input.heuristic_landmarks,
        })
        .collect::<Vec<_>>();

    let artifacts = build_shadow_artifacts(&shadow_inputs).context("build AST shadow artifacts")?;
    let paths = write_shadow_artifacts(&args.out, &artifacts)
        .with_context(|| format!("write AST shadow artifacts to {}", args.out.display()))?;
    if let Some(summary_path) = &args.summary_md {
        write_summary_md(summary_path, &args, &paths, &artifacts.diff, root)
            .with_context(|| format!("write AST shadow summary to {}", summary_path.display()))?;
    }

    let ast_files = artifacts
        .ast
        .get("files")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    let diff_files = artifacts
        .diff
        .get("files")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);

    println!(
        "AST shadow comparison written to {} ({} input file(s), {} diff file(s))",
        args.out.display(),
        ast_files,
        diff_files
    );
    println!("  heuristic: {}", paths.heuristic.display());
    println!("  ast: {}", paths.ast.display());
    println!("  diff: {}", paths.diff.display());
    if let Some(summary_path) = &args.summary_md {
        println!("  summary: {}", summary_path.display());
    }

    Ok(())
}

fn write_summary_md(
    summary_path: &Path,
    args: &AstShadowCompareArgs,
    paths: &tokmd_analysis::ast::ShadowArtifactPaths,
    diff: &serde_json::Value,
    root: &Path,
) -> Result<()> {
    let summary = render_summary_md(args, paths, diff, root)?;

    if let Some(parent) = summary_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("create summary parent {}", parent.display()))?;
    }

    fs::write(summary_path, summary)
        .with_context(|| format!("write summary {}", summary_path.display()))
}

fn render_summary_md(
    args: &AstShadowCompareArgs,
    paths: &tokmd_analysis::ast::ShadowArtifactPaths,
    diff: &serde_json::Value,
    root: &Path,
) -> Result<String> {
    let summary = diff
        .get("summary")
        .and_then(serde_json::Value::as_object)
        .context("diff artifact is missing summary object")?;
    let files = diff
        .get("files")
        .and_then(serde_json::Value::as_array)
        .context("diff artifact is missing files array")?;

    let mut markdown = String::new();
    markdown.push_str("# AST Shadow Comparison\n\n");
    markdown.push_str("Developer-facing heuristic-vs-AST comparison evidence. ");
    markdown.push_str("This is not a merge verdict, proof promotion, or public receipt.\n\n");

    markdown.push_str("## Summary\n\n");
    push_count_line(&mut markdown, "Files compared", summary, "files")?;
    push_count_line(&mut markdown, "Matched landmarks", summary, "matched")?;
    push_count_line(
        &mut markdown,
        "Heuristic-only landmarks",
        summary,
        "heuristic_only",
    )?;
    push_count_line(&mut markdown, "AST-only landmarks", summary, "ast_only")?;
    push_count_line(
        &mut markdown,
        "Parse-degraded files",
        summary,
        "parse_degraded",
    )?;
    push_count_line(&mut markdown, "Unsupported files", summary, "unsupported")?;

    let kind_counts = landmark_kind_counts(files)?;
    if !kind_counts.is_empty() {
        markdown.push_str("\n## Landmark Kinds\n\n");
        markdown.push_str("| Kind | Matched | Heuristic-only | AST-only |\n");
        markdown.push_str("| --- | ---: | ---: | ---: |\n");
        for (kind, counts) in kind_counts {
            markdown.push_str(&format!(
                "| `{kind}` | {} | {} | {} |\n",
                counts.matched, counts.heuristic_only, counts.ast_only
            ));
        }
    }

    markdown.push_str("\n## Artifacts\n\n");
    markdown.push_str(&format!(
        "- heuristic: `{}`\n",
        summary_display_path(&paths.heuristic, root)?
    ));
    markdown.push_str(&format!(
        "- ast: `{}`\n",
        summary_display_path(&paths.ast, root)?
    ));
    markdown.push_str(&format!(
        "- diff: `{}`\n",
        summary_display_path(&paths.diff, root)?
    ));

    markdown.push_str("\n## Files\n\n");
    for file in files {
        let path = string_field(file, "path")?;
        let status = string_field(file, "status")?;
        let matches = array_len(file, "matches")?;
        let heuristic_only = array_len(file, "heuristic_only")?;
        let ast_only = array_len(file, "ast_only")?;
        let parse_degraded = bool_field(file, "parse_degraded")?;
        let unsupported = bool_field(file, "unsupported")?;

        markdown.push_str(&format!("- `{path}`\n"));
        markdown.push_str(&format!("  - status: `{status}`\n"));
        markdown.push_str(&format!("  - matched landmarks: {matches}\n"));
        markdown.push_str(&format!("  - heuristic-only landmarks: {heuristic_only}\n"));
        markdown.push_str(&format!("  - AST-only landmarks: {ast_only}\n"));
        markdown.push_str(&format!("  - parse degraded: {parse_degraded}\n"));
        markdown.push_str(&format!("  - unsupported: {unsupported}\n"));
    }

    markdown.push_str("\n## Reproduce\n\n");
    markdown.push_str("```bash\n");
    markdown.push_str("cargo xtask ast-shadow-compare");
    for path in &args.paths {
        markdown.push_str(" \\\n  --path ");
        markdown.push_str(&summary_display_path(path, root)?);
    }
    markdown.push_str(" \\\n  --out ");
    markdown.push_str(&summary_display_path(&args.out, root)?);
    if let Some(summary_md) = &args.summary_md {
        markdown.push_str(" \\\n  --summary-md ");
        markdown.push_str(&summary_display_path(summary_md, root)?);
    }
    markdown.push_str("\n```\n");

    Ok(markdown)
}

#[derive(Default)]
struct LandmarkKindCounts {
    matched: usize,
    heuristic_only: usize,
    ast_only: usize,
}

fn landmark_kind_counts(
    files: &[serde_json::Value],
) -> Result<BTreeMap<String, LandmarkKindCounts>> {
    let mut counts = BTreeMap::<String, LandmarkKindCounts>::new();
    for file in files {
        add_landmark_kind_counts(&mut counts, file, "matches")?;
        add_landmark_kind_counts(&mut counts, file, "heuristic_only")?;
        add_landmark_kind_counts(&mut counts, file, "ast_only")?;
    }
    Ok(counts)
}

fn add_landmark_kind_counts(
    counts: &mut BTreeMap<String, LandmarkKindCounts>,
    file: &serde_json::Value,
    field: &str,
) -> Result<()> {
    let landmarks = file
        .get(field)
        .and_then(serde_json::Value::as_array)
        .with_context(|| format!("diff file entry is missing array field `{field}`"))?;
    for landmark in landmarks {
        let kind = string_field(landmark, "kind")?;
        let entry = counts.entry(kind.to_owned()).or_default();
        match field {
            "matches" => entry.matched += 1,
            "heuristic_only" => entry.heuristic_only += 1,
            "ast_only" => entry.ast_only += 1,
            _ => unreachable!("unexpected landmark diff field"),
        }
    }
    Ok(())
}

fn push_count_line(
    markdown: &mut String,
    label: &str,
    summary: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<()> {
    let count = summary
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .with_context(|| format!("diff summary is missing numeric field `{key}`"))?;
    markdown.push_str(&format!("- {label}: {count}\n"));
    Ok(())
}

fn string_field<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .with_context(|| format!("diff file entry is missing string field `{field}`"))
}

fn bool_field(value: &serde_json::Value, field: &str) -> Result<bool> {
    value
        .get(field)
        .and_then(serde_json::Value::as_bool)
        .with_context(|| format!("diff file entry is missing bool field `{field}`"))
}

fn array_len(value: &serde_json::Value, field: &str) -> Result<usize> {
    value
        .get(field)
        .and_then(serde_json::Value::as_array)
        .map(Vec::len)
        .with_context(|| format!("diff file entry is missing array field `{field}`"))
}

fn normalize_display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn summary_display_path(path: &Path, root: &Path) -> Result<String> {
    let display_path = if path.is_absolute() {
        path.strip_prefix(root)
            .with_context(|| {
                format!(
                    "AST shadow summary paths must stay under the repo root: {}",
                    path.display()
                )
            })?
            .to_path_buf()
    } else {
        path.to_path_buf()
    };
    Ok(normalize_display_path(&display_path))
}

#[derive(Debug)]
struct RunnerInput {
    path: String,
    source: String,
    heuristic_landmarks: Vec<ShadowLandmark>,
}

fn collect_inputs(paths: &[PathBuf], root: &Path) -> Result<Vec<RunnerInput>> {
    let mut inputs = paths
        .iter()
        .map(|path| collect_input(path, root))
        .collect::<Result<Vec<_>>>()?;
    inputs.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(inputs)
}

fn collect_input(path: &Path, root: &Path) -> Result<RunnerInput> {
    let rel_path = validate_repo_relative_rust_path(path, root)?;
    let full_path = root.join(&rel_path);
    let source =
        fs::read_to_string(&full_path).with_context(|| format!("read {}", rel_path.display()))?;
    let normalized = normalize_shadow_path(&rel_path.to_string_lossy());
    let heuristic_landmarks = heuristic_rust_landmarks(&source);

    Ok(RunnerInput {
        path: normalized,
        source,
        heuristic_landmarks,
    })
}

fn validate_repo_relative_rust_path(path: &Path, root: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        bail!(
            "AST shadow input paths must be repo-relative: {}",
            path.display()
        );
    }

    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::Prefix(_) | Component::RootDir
        )
    }) {
        bail!(
            "AST shadow input paths must stay inside the repo: {}",
            path.display()
        );
    }

    if path.extension() != Some(OsStr::new("rs")) {
        bail!(
            "AST shadow compare currently accepts only Rust `.rs` files: {}",
            path.display()
        );
    }

    let root = root
        .canonicalize()
        .with_context(|| format!("canonicalize repo root {}", root.display()))?;
    let full_path = root.join(path);
    let canonical = full_path
        .canonicalize()
        .with_context(|| format!("canonicalize input path {}", path.display()))?;
    if !canonical.starts_with(&root) {
        bail!(
            "AST shadow input path resolves outside the repo: {}",
            path.display()
        );
    }

    Ok(path.to_path_buf())
}

fn heuristic_rust_landmarks(source: &str) -> Vec<ShadowLandmark> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut landmarks = Vec::new();
    let mut line_index = 0usize;

    while line_index < lines.len() {
        let line = lines[line_index];
        let trimmed = line.trim_start();
        let line_number = line_index + 1;

        if trimmed.starts_with("use ") {
            let end_line = collect_use_end_line(&lines, line_index);
            let name = normalize_use_text(&lines[line_index..=end_line - 1]);
            landmarks.push(ShadowLandmark {
                kind: "import".to_owned(),
                name,
                start_line: line_number,
                end_line,
            });
            line_index = end_line;
            continue;
        }

        if let Some(name) = function_name_from_line(trimmed) {
            landmarks.push(ShadowLandmark {
                kind: "function".to_owned(),
                name,
                start_line: line_number,
                end_line: block_end_line(&lines, line_index),
            });
        }

        for control_flow in ["if", "for", "while", "match", "loop"] {
            if contains_token(trimmed, control_flow) {
                landmarks.push(ShadowLandmark {
                    kind: "control_flow".to_owned(),
                    name: control_flow.to_owned(),
                    start_line: line_number,
                    end_line: block_end_line(&lines, line_index),
                });
            }
        }

        line_index += 1;
    }

    landmarks.sort();
    landmarks.dedup();
    landmarks
}

fn collect_use_end_line(lines: &[&str], start: usize) -> usize {
    lines
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, line)| line.contains(';').then_some(index + 1))
        .unwrap_or(start + 1)
}

fn normalize_use_text(lines: &[&str]) -> String {
    lines
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .strip_prefix("use ")
        .unwrap_or("")
        .trim_end_matches(';')
        .trim()
        .to_owned()
}

fn function_name_from_line(line: &str) -> Option<String> {
    let fn_start = find_token(line, "fn")?;
    let after_fn = line.get(fn_start + 2..)?.trim_start();
    let name = after_fn
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>();
    (!name.is_empty()).then_some(name)
}

fn contains_token(line: &str, token: &str) -> bool {
    find_token(line, token).is_some()
}

fn find_token(line: &str, token: &str) -> Option<usize> {
    line.match_indices(token)
        .find(|(index, _)| token_boundary(line, *index, token.len()))
        .map(|(index, _)| index)
}

fn token_boundary(line: &str, start: usize, len: usize) -> bool {
    let before = start
        .checked_sub(1)
        .and_then(|index| line.as_bytes().get(index))
        .copied();
    let after = line.as_bytes().get(start + len).copied();
    !is_ident_byte(before) && !is_ident_byte(after)
}

fn is_ident_byte(byte: Option<u8>) -> bool {
    byte.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn block_end_line(lines: &[&str], start: usize) -> usize {
    let mut depth = 0isize;
    let mut saw_open = false;

    for (index, line) in lines.iter().enumerate().skip(start) {
        for byte in line.bytes() {
            match byte {
                b'{' => {
                    saw_open = true;
                    depth += 1;
                }
                b'}' if saw_open => {
                    depth -= 1;
                    if depth <= 0 {
                        return index + 1;
                    }
                }
                _ => {}
            }
        }
    }

    start + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn rejects_absolute_paths() {
        let root = tempfile::tempdir().expect("tempdir");
        let absolute = root.path().join("src/lib.rs");
        let error = validate_repo_relative_rust_path(&absolute, root.path())
            .expect_err("absolute paths should be rejected");

        assert!(error.to_string().contains("repo-relative"));
    }

    #[test]
    fn rejects_parent_paths() {
        let root = tempfile::tempdir().expect("tempdir");
        let error = validate_repo_relative_rust_path(Path::new("../lib.rs"), root.path())
            .expect_err("parent paths should be rejected");

        assert!(error.to_string().contains("inside the repo"));
    }

    #[test]
    fn rejects_non_rust_paths() -> Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(root.path().join("README.md"), "# docs\n")?;

        let error = validate_repo_relative_rust_path(Path::new("README.md"), root.path())
            .expect_err("non-Rust paths should be rejected");

        assert!(error.to_string().contains("`.rs` files"));
        Ok(())
    }

    #[test]
    fn heuristic_extracts_first_slice_landmarks() {
        let source = r#"
use std::{
    fs,
    path::Path,
};

pub fn compute(value: usize) -> usize {
    if value == 0 {
        return 0;
    }

    for item in 0..value {
        while item > 1 {
            break;
        }
    }

    match value {
        1 => loop {
            break 1;
        },
        _ => value,
    }
}
"#;

        let landmarks = heuristic_rust_landmarks(source);
        let observed = landmarks
            .iter()
            .map(|landmark| (landmark.kind.as_str(), landmark.name.as_str()))
            .collect::<Vec<_>>();

        assert!(observed.contains(&("import", "std::{ fs, path::Path, }")));
        assert!(observed.contains(&("function", "compute")));
        assert!(observed.contains(&("control_flow", "if")));
        assert!(observed.contains(&("control_flow", "for")));
        assert!(observed.contains(&("control_flow", "while")));
        assert!(observed.contains(&("control_flow", "match")));
        assert!(observed.contains(&("control_flow", "loop")));
    }

    #[test]
    fn runner_writes_deterministic_artifacts() -> Result<()> {
        let root = tempfile::tempdir()?;
        let fixture_dir = root.path().join("fixtures/ast-shadow/rust");
        fs::create_dir_all(&fixture_dir)?;
        fs::write(
            fixture_dir.join("basic.rs"),
            "use std::fs;\n\npub fn compute(value: usize) -> usize {\n    if value > 0 {\n        value\n    } else {\n        0\n    }\n}\n",
        )?;
        let out = root.path().join("target/tokmd-ast-shadow");
        let args = AstShadowCompareArgs {
            paths: vec![PathBuf::from("fixtures/ast-shadow/rust/basic.rs")],
            out: out.clone(),
            summary_md: None,
        };

        run_with_root(args.clone(), root.path())?;
        let first = fs::read_to_string(out.join("diff.json"))?;
        run_with_root(args, root.path())?;
        let second = fs::read_to_string(out.join("diff.json"))?;

        assert_eq!(first, second);
        assert!(out.join("heuristic.json").exists());
        assert!(out.join("ast.json").exists());
        assert!(out.join("diff.json").exists());
        assert!(first.contains("\"schema\": \"tokmd.ast_shadow.v1\""));
        assert!(!first.contains(root.path().to_string_lossy().as_ref()));
        Ok(())
    }

    #[test]
    fn runner_writes_markdown_summary_when_requested() -> Result<()> {
        let root = tempfile::tempdir()?;
        let fixture_dir = root.path().join("fixtures/ast-shadow/rust");
        fs::create_dir_all(&fixture_dir)?;
        fs::write(
            fixture_dir.join("basic.rs"),
            "use std::fs;\n\npub fn compute(value: usize) -> usize {\n    if value > 0 {\n        value\n    } else {\n        0\n    }\n}\n",
        )?;
        let out = root.path().join("target/tokmd-ast-shadow");
        let summary_md = root.path().join("target/tokmd-ast-shadow/summary.md");
        let args = AstShadowCompareArgs {
            paths: vec![PathBuf::from("fixtures/ast-shadow/rust/basic.rs")],
            out,
            summary_md: Some(summary_md.clone()),
        };

        run_with_root(args, root.path())?;
        let summary = fs::read_to_string(summary_md)?;

        assert!(summary.contains("# AST Shadow Comparison"));
        assert!(summary.contains("- Files compared: 1"));
        assert!(summary.contains("- `fixtures/ast-shadow/rust/basic.rs`"));
        assert!(summary.contains("cargo xtask ast-shadow-compare"));
        assert!(summary.contains("--summary-md"));
        assert!(!summary.contains(&normalize_display_path(root.path())));
        Ok(())
    }

    #[test]
    fn summary_includes_landmark_kind_counts() -> Result<()> {
        let root = tempfile::tempdir()?;
        let paths = tokmd_analysis::ast::ShadowArtifactPaths {
            heuristic: root.path().join("target/tokmd-ast-shadow/heuristic.json"),
            ast: root.path().join("target/tokmd-ast-shadow/ast.json"),
            diff: root.path().join("target/tokmd-ast-shadow/diff.json"),
        };
        let args = AstShadowCompareArgs {
            paths: vec![PathBuf::from("src/lib.rs")],
            out: PathBuf::from("target/tokmd-ast-shadow"),
            summary_md: Some(PathBuf::from("target/tokmd-ast-shadow/summary.md")),
        };
        let diff = serde_json::json!({
            "summary": {
                "files": 1,
                "matched": 1,
                "heuristic_only": 2,
                "ast_only": 1,
                "parse_degraded": 0,
                "unsupported": 0
            },
            "files": [
                {
                    "path": "src/lib.rs",
                    "status": "compared",
                    "matches": [
                        {"kind": "function", "name": "run", "start_line": 1, "end_line": 3}
                    ],
                    "heuristic_only": [
                        {"kind": "control_flow", "name": "if", "start_line": 2, "end_line": 2},
                        {"kind": "function", "name": "fixture", "start_line": 8, "end_line": 8}
                    ],
                    "ast_only": [
                        {"kind": "import", "name": "std::fs", "start_line": 1, "end_line": 1}
                    ],
                    "parse_degraded": false,
                    "unsupported": false
                }
            ]
        });

        let summary = render_summary_md(&args, &paths, &diff, root.path())?;

        assert!(summary.contains("## Landmark Kinds"));
        assert!(summary.contains("| Kind | Matched | Heuristic-only | AST-only |"));
        assert!(summary.contains("| `control_flow` | 0 | 1 | 0 |"));
        assert!(summary.contains("| `function` | 1 | 1 | 0 |"));
        assert!(summary.contains("| `import` | 0 | 0 | 1 |"));
        Ok(())
    }

    #[test]
    fn summary_rejects_paths_outside_repo_root() -> Result<()> {
        let root = tempfile::tempdir()?;
        let outside = tempfile::tempdir()?;
        let fixture_dir = root.path().join("fixtures/ast-shadow/rust");
        fs::create_dir_all(&fixture_dir)?;
        fs::write(
            fixture_dir.join("basic.rs"),
            "use std::fs;\n\npub fn compute() {}\n",
        )?;
        let args = AstShadowCompareArgs {
            paths: vec![PathBuf::from("fixtures/ast-shadow/rust/basic.rs")],
            out: root.path().join("target/tokmd-ast-shadow"),
            summary_md: Some(outside.path().join("summary.md")),
        };

        let error = run_with_root(args, root.path())
            .expect_err("summary paths outside the repo should fail");

        assert!(error.to_string().contains("AST shadow summary"));
        assert!(!outside.path().join("summary.md").exists());
        Ok(())
    }
}
