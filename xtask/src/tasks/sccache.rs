use crate::cli::SccacheArgs;
use anyhow::{Context, Result, bail};
use cargo_metadata::MetadataCommand;
use std::collections::hash_map::DefaultHasher;
use std::ffi::{OsStr, OsString};
use std::hash::{Hash, Hasher};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(args: SccacheArgs) -> Result<()> {
    if args.check {
        return run_check(&args);
    }
    if args.stats {
        return run_sccache_tool(&["--show-stats"], &args);
    }
    if args.stop {
        return run_sccache_tool(&["--stop-server"], &args);
    }
    if args.cargo_args.is_empty() {
        bail!(
            "provide a cargo command after `--`, for example `cargo with-sccache test --workspace --all-features`"
        );
    }

    ensure_sccache_available()?;
    let server_port = resolved_server_port()?;
    let basedirs = resolved_basedirs(&args.basedirs)?;

    let mut command = Command::new("cargo");
    command.args(&args.cargo_args);
    command.env("RUSTC_WRAPPER", "sccache");
    command.env("SCCACHE_SERVER_PORT", &server_port);
    if let Some(value) = basedirs.as_ref() {
        command.env("SCCACHE_BASEDIRS", value);
        println!("sccache: SCCACHE_BASEDIRS={}", value.to_string_lossy());
    }
    if let Some(target_dir) = redirected_target_dir(&args.cargo_args)? {
        command.env("CARGO_TARGET_DIR", &target_dir);
        println!("sccache: CARGO_TARGET_DIR={}", target_dir.display());
    }

    let disable_incremental = should_disable_incremental(args.keep_incremental);
    if disable_incremental {
        // sccache cannot reuse incrementally compiled Rust crates.
        command.env("CARGO_INCREMENTAL", "0");
    }

    println!(
        "sccache: RUSTC_WRAPPER={}",
        PathBuf::from("sccache").display()
    );
    if disable_incremental {
        println!("sccache: CARGO_INCREMENTAL=0");
    } else {
        println!("sccache: keeping existing incremental configuration");
    }
    println!("sccache: cargo {}", display_cargo_args(&args.cargo_args));

    let status = command
        .status()
        .context("failed to run cargo under sccache")?;
    if !status.success() {
        bail!(
            "cargo command failed under sccache (exit code: {})",
            status.code().unwrap_or(-1)
        );
    }

    println!("sccache: run `cargo sccache-stats` to inspect cache hits");
    Ok(())
}

fn run_check(args: &SccacheArgs) -> Result<()> {
    let version = sccache_version()?;
    let port = resolved_server_port()?;
    let basedirs = resolved_basedirs(&args.basedirs)?;
    println!("sccache: found {version}");
    println!("sccache: opt in with `cargo with-sccache test --workspace --all-features`");
    println!(
        "sccache: use `cargo xtask sccache --basedir <PATH> -- test ...` to reuse cache entries across worktrees"
    );
    println!("sccache: use `cargo sccache-stats` to inspect hit rates");
    println!("sccache: using server port {port}");
    if let Some(value) = basedirs {
        println!("sccache: using basedirs {}", value.to_string_lossy());
    }
    println!(
        "sccache: this wrapper defaults CARGO_INCREMENTAL=0 unless you pass --keep-incremental"
    );
    Ok(())
}

fn run_sccache_tool(args: &[&str], config: &SccacheArgs) -> Result<()> {
    ensure_sccache_available()?;
    let basedirs = resolved_basedirs(&config.basedirs)?;
    let mut command = Command::new("sccache");
    command.args(args);
    command.env("SCCACHE_SERVER_PORT", resolved_server_port()?);
    if let Some(value) = basedirs {
        command.env("SCCACHE_BASEDIRS", value);
    }
    let status = command
        .status()
        .with_context(|| format!("failed to run `sccache {}`", args.join(" ")))?;
    if !status.success() {
        bail!(
            "`sccache {}` failed with exit code {}",
            args.join(" "),
            status.code().unwrap_or(-1)
        );
    }
    Ok(())
}

fn ensure_sccache_available() -> Result<()> {
    sccache_version().map(|_| ())
}

fn resolved_server_port() -> Result<String> {
    if let Some(value) = std::env::var_os("SCCACHE_SERVER_PORT") {
        let value = value.to_string_lossy().trim().to_string();
        if !value.is_empty() {
            return Ok(value);
        }
    }

    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("failed to load cargo metadata for sccache port selection")?;
    Ok(default_server_port_for_key(metadata.workspace_root.as_str()).to_string())
}

fn resolved_basedirs(configured: &[PathBuf]) -> Result<Option<OsString>> {
    compose_basedirs(
        configured,
        &workspace_root()?,
        std::env::var_os("SCCACHE_BASEDIRS").as_deref(),
    )
}

fn compose_basedirs(
    configured: &[PathBuf],
    workspace_root: &Path,
    inherited: Option<&OsStr>,
) -> Result<Option<OsString>> {
    if configured.is_empty() {
        return Ok(inherited.and_then(nonempty_os_string));
    }

    let mut resolved = Vec::with_capacity(configured.len());
    for path in configured {
        let path = if path.is_absolute() {
            path.clone()
        } else {
            workspace_root.join(path)
        };
        std::fs::metadata(&path)
            .with_context(|| format!("sccache basedir does not exist: {}", path.display()))?;
        resolved.push(path);
    }

    std::env::join_paths(resolved)
        .map(Some)
        .context("failed to compose SCCACHE_BASEDIRS from configured paths")
}

fn nonempty_os_string(value: &OsStr) -> Option<OsString> {
    let trimmed = value.to_string_lossy().trim().to_string();
    (!trimmed.is_empty()).then(|| OsString::from(trimmed))
}

fn sccache_version() -> Result<String> {
    let output = match Command::new("sccache").arg("--version").output() {
        Ok(output) => output,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bail!("sccache is not installed. {}", install_hint())
        }
        Err(error) => return Err(error).context("failed to invoke `sccache --version`"),
    };
    if !output.status.success() {
        bail!(
            "`sccache --version` failed (exit code: {}). {}",
            output.status.code().unwrap_or(-1),
            install_hint()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn redirected_target_dir(args: &[String]) -> Result<Option<PathBuf>> {
    if !should_isolate_xtask_tests(
        args,
        cfg!(windows),
        std::env::var_os("CARGO_TARGET_DIR").is_some(),
    ) {
        return Ok(None);
    }

    Ok(Some(workspace_root()?.join("target").join("xtask-sccache")))
}

fn default_server_port_for_key(key: &str) -> u16 {
    const PORT_BASE: u16 = 45_000;
    const PORT_SPAN: u16 = 1_000;

    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    PORT_BASE + (hasher.finish() % u64::from(PORT_SPAN)) as u16
}

fn workspace_root() -> Result<std::path::PathBuf> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("failed to load cargo metadata for xtask workspace root")?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

fn should_isolate_xtask_tests(args: &[String], is_windows: bool, has_target_dir: bool) -> bool {
    if !is_windows || has_target_dir || args.first().map(String::as_str) != Some("test") {
        return false;
    }

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if matches!(arg.as_str(), "-p" | "--package")
            && iter.next().map(String::as_str) == Some("xtask")
        {
            return true;
        }
    }

    false
}

fn should_disable_incremental(keep_incremental: bool) -> bool {
    !keep_incremental
}

fn display_cargo_args(args: &[String]) -> String {
    args.join(" ")
}

fn install_hint() -> &'static str {
    if cfg!(windows) {
        "Install via `winget install Mozilla.sccache`, `scoop install sccache`, or `cargo install sccache --locked`."
    } else if cfg!(target_os = "macos") {
        "Install via `brew install sccache` or `cargo install sccache --locked`."
    } else {
        "Install via your package manager or `cargo install sccache --locked`."
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compose_basedirs, default_server_port_for_key, display_cargo_args, install_hint,
        should_disable_incremental, should_isolate_xtask_tests,
    };
    use std::ffi::OsStr;
    use std::path::PathBuf;

    #[test]
    fn should_disable_incremental_when_unset() {
        assert!(should_disable_incremental(false));
    }

    #[test]
    fn should_disable_incremental_even_if_the_parent_env_was_set() {
        assert!(should_disable_incremental(false));
    }

    #[test]
    fn should_respect_keep_incremental_flag() {
        assert!(!should_disable_incremental(true));
    }

    #[test]
    fn display_cargo_args_is_human_readable() {
        let args = vec![
            "test".to_string(),
            "--workspace".to_string(),
            "--all-features".to_string(),
        ];
        assert_eq!(display_cargo_args(&args), "test --workspace --all-features");
    }

    #[test]
    fn install_hint_mentions_known_install_path() {
        let hint = install_hint();
        assert!(
            hint.contains("cargo install sccache --locked"),
            "install hint should mention cargo installation"
        );
    }

    #[test]
    fn default_server_port_is_deterministic_and_in_repo_range() {
        let first = default_server_port_for_key("C:/Code/Rust/tokmd");
        let second = default_server_port_for_key("C:/Code/Rust/tokmd");
        let other = default_server_port_for_key("C:/Code/Rust/other");

        assert_eq!(first, second);
        assert!((45_000..46_000).contains(&first));
        assert!((45_000..46_000).contains(&other));
    }

    #[test]
    fn compose_basedirs_prefers_explicit_paths() {
        let root = temp_dir("basedirs");
        let first = root.join("repo-a");
        let second = root.join("repo-b");
        std::fs::create_dir_all(&first).unwrap();
        std::fs::create_dir_all(&second).unwrap();

        let value = compose_basedirs(
            &[PathBuf::from("repo-a"), second.clone()],
            &root,
            Some(OsStr::new("ignored")),
        )
        .expect("basedirs should compose")
        .expect("basedirs should be present");
        let actual: Vec<PathBuf> = std::env::split_paths(&value).collect();
        assert_eq!(actual, vec![first, second]);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn compose_basedirs_falls_back_to_inherited_env() {
        let root = temp_dir("basedirs-env");
        let inherited = if cfg!(windows) {
            OsStr::new(r"C:\Code\Rust;D:\Cache")
        } else {
            OsStr::new("/code/rust:/cache")
        };

        let value = compose_basedirs(&[], &root, Some(inherited))
            .expect("inherited env should be accepted")
            .expect("inherited env should be present");
        assert_eq!(value, inherited);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn should_isolate_windows_xtask_test_runs() {
        let args = vec![
            "test".to_string(),
            "-p".to_string(),
            "xtask".to_string(),
            "--no-run".to_string(),
        ];

        assert!(should_isolate_xtask_tests(&args, true, false));
        assert!(!should_isolate_xtask_tests(&args, true, true));
        assert!(!should_isolate_xtask_tests(&args, false, false));
    }

    #[test]
    fn should_not_isolate_non_xtask_or_non_test_invocations() {
        let check_args = vec!["check".to_string(), "-p".to_string(), "xtask".to_string()];
        let other_pkg_args = vec!["test".to_string(), "-p".to_string(), "tokmd".to_string()];

        assert!(!should_isolate_xtask_tests(&check_args, true, false));
        assert!(!should_isolate_xtask_tests(&other_pkg_args, true, false));
    }

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "tokmd-sccache-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }
}
