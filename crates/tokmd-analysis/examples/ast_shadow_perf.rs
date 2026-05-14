use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tokmd_analysis::ast::{
    AstLanguage, ShadowFileInput, ShadowLandmark, build_shadow_artifacts, parse_rust_landmarks,
};

const SCHEMA: &str = "tokmd.ast_shadow_perf.v1";
const PARSER_CRATE: &str = "tree-sitter-rust";

#[derive(Debug)]
struct Config {
    iterations: usize,
    files: usize,
    functions_per_file: usize,
    out: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            iterations: 10,
            files: 8,
            functions_per_file: 12,
            out: PathBuf::from("target/perf/ast-shadow-perf.json"),
        }
    }
}

#[derive(Debug, Serialize)]
struct PerfReceipt {
    schema: &'static str,
    schema_version: u32,
    language: &'static str,
    parser_crate: &'static str,
    target: PerfTarget,
    parse: PerfTiming,
    artifacts: PerfTiming,
    status: PerfStatus,
}

#[derive(Debug, Serialize)]
struct PerfTarget {
    synthetic: bool,
    source_files: usize,
    functions_per_file: usize,
    source_bytes: usize,
}

#[derive(Debug, Serialize)]
struct PerfTiming {
    operation: &'static str,
    iterations: usize,
    total_ms: u128,
    average_us: u128,
    observed_items: usize,
}

#[derive(Debug, Serialize)]
struct PerfStatus {
    ok: bool,
    parse_error_count: usize,
    artifact_file_observations: usize,
}

fn main() -> Result<()> {
    let config = parse_args(env::args().skip(1))?;
    let receipt = run_benchmark(&config)?;

    if let Some(parent) = config.out.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(&receipt).context("serialize AST perf receipt")?;
    fs::write(&config.out, format!("{json}\n"))
        .with_context(|| format!("write {}", config.out.display()))?;
    println!(
        "ast shadow perf receipt written to {} ({} file(s), {} iteration(s))",
        config.out.display(),
        config.files,
        config.iterations
    );
    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Config> {
    let mut config = Config::default();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--iterations" => {
                config.iterations = parse_positive_usize("--iterations", args.next())?;
            }
            "--files" => {
                config.files = parse_positive_usize("--files", args.next())?;
            }
            "--functions-per-file" => {
                config.functions_per_file =
                    parse_positive_usize("--functions-per-file", args.next())?;
            }
            "--out" => {
                config.out = PathBuf::from(
                    args.next()
                        .context("--out requires a receipt output path argument")?,
                );
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => bail!("unknown argument `{other}`; pass --help for usage"),
        }
    }

    Ok(config)
}

fn parse_positive_usize(flag: &str, value: Option<String>) -> Result<usize> {
    let value = value.with_context(|| format!("{flag} requires a positive integer argument"))?;
    let parsed = value
        .parse::<usize>()
        .with_context(|| format!("{flag} must be a positive integer"))?;
    if parsed == 0 {
        bail!("{flag} must be greater than zero");
    }
    Ok(parsed)
}

fn print_usage() {
    println!(
        "\
Usage: cargo run -p tokmd-analysis --features ast --example ast_shadow_perf -- [OPTIONS]

Options:
  --iterations <N>           Number of parse/artifact loops to run (default: 10)
  --files <N>                Synthetic Rust file count (default: 8)
  --functions-per-file <N>   Synthetic function count per file (default: 12)
  --out <PATH>               JSON receipt path (default: target/perf/ast-shadow-perf.json)
"
    );
}

fn run_benchmark(config: &Config) -> Result<PerfReceipt> {
    let sources = synthetic_sources(config.files, config.functions_per_file);
    let source_bytes = sources.iter().map(String::len).sum::<usize>();

    let parse_start = Instant::now();
    let mut parse_error_count = 0;
    let mut observed_landmarks = 0;
    for _ in 0..config.iterations {
        for source in &sources {
            let shadow = parse_rust_landmarks(source)?;
            parse_error_count += usize::from(shadow.has_error);
            observed_landmarks += shadow.landmarks.len();
        }
    }
    let parse_total_us = parse_start.elapsed().as_micros();

    let paths = (0..config.files)
        .map(|index| format!("synthetic/file_{index}.rs"))
        .collect::<Vec<_>>();
    let empty_heuristic: &[ShadowLandmark] = &[];
    let inputs = sources
        .iter()
        .enumerate()
        .map(|(index, source)| ShadowFileInput {
            path: paths[index].as_str(),
            language: AstLanguage::Rust,
            source,
            heuristic_landmarks: empty_heuristic,
        })
        .collect::<Vec<_>>();

    let artifact_start = Instant::now();
    let mut artifact_file_observations = 0;
    for _ in 0..config.iterations {
        let artifacts = build_shadow_artifacts(&inputs)?;
        artifact_file_observations += artifacts
            .ast
            .get("files")
            .and_then(serde_json::Value::as_array)
            .map_or(0, Vec::len);
    }
    let artifact_total_us = artifact_start.elapsed().as_micros();

    Ok(PerfReceipt {
        schema: SCHEMA,
        schema_version: 1,
        language: "rust",
        parser_crate: PARSER_CRATE,
        target: PerfTarget {
            synthetic: true,
            source_files: config.files,
            functions_per_file: config.functions_per_file,
            source_bytes,
        },
        parse: PerfTiming {
            operation: "parse_rust_landmarks",
            iterations: config.iterations,
            total_ms: parse_total_us / 1_000,
            average_us: average_us(parse_total_us, config.iterations * config.files),
            observed_items: observed_landmarks,
        },
        artifacts: PerfTiming {
            operation: "build_shadow_artifacts",
            iterations: config.iterations,
            total_ms: artifact_total_us / 1_000,
            average_us: average_us(artifact_total_us, config.iterations),
            observed_items: artifact_file_observations,
        },
        status: PerfStatus {
            ok: parse_error_count == 0,
            parse_error_count,
            artifact_file_observations,
        },
    })
}

fn average_us(total_us: u128, operations: usize) -> u128 {
    total_us / operations.max(1) as u128
}

fn synthetic_sources(files: usize, functions_per_file: usize) -> Vec<String> {
    (0..files)
        .map(|file_index| synthetic_source(file_index, functions_per_file))
        .collect()
}

fn synthetic_source(file_index: usize, functions_per_file: usize) -> String {
    let mut source = String::from("use std::{fs, path::Path};\n\n");
    source.push_str(&format!("pub mod module_{file_index} {{\n"));
    for function_index in 0..functions_per_file {
        source.push_str(&format!(
            "\
    pub fn function_{file_index}_{function_index}(value: usize) -> usize {{
        if value == 0 {{
            return 0;
        }}
        for item in 0..value {{
            while item > 1 {{
                break;
            }}
        }}
        match value {{
            1 => loop {{
                break 1;
            }},
            _ => value,
        }}
    }}

"
        ));
    }
    source.push_str("}\n");
    source
}
