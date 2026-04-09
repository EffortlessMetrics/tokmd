mod common;

use assert_cmd::Command;
use clap::ValueEnum;
use predicates::prelude::*;
use tokmd_cli_args::AnalysisPreset;

fn tokmd_cmd() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tokmd"));
    cmd.current_dir(common::fixture_root());
    cmd
}

#[test]
fn completions_bash_output_contains_command() {
    let mut cmd = tokmd_cmd();
    cmd.arg("completions")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("tokmd"));
}

#[test]
fn completions_bash_include_dynamic_preset_values() {
    let expected = AnalysisPreset::value_variants()
        .iter()
        .filter_map(|preset| {
            preset
                .to_possible_value()
                .map(|value| value.get_name().to_string())
        })
        .collect::<Vec<_>>()
        .join(" ");

    let mut cmd = tokmd_cmd();
    cmd.arg("completions")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}

#[test]
fn completions_bash_include_dynamic_format_values() {
    let mut cmd = tokmd_cmd();
    cmd.arg("completions")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("compgen -W \"md json\""));
}
