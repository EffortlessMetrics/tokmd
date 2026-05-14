use crate::cli::AstShadowCompareArgs;
use anyhow::{Context, Result, bail};
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

    Ok(())
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
}
