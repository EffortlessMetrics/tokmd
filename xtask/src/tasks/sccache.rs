use crate::cli::SccacheArgs;
use crate::tasks::build_guard::{ScopedTempDir, ensure_min_free_space};
use anyhow::{Context, Result, bail};
use cargo_metadata::MetadataCommand;
use std::ffi::{OsStr, OsString};
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

    let (sccache_program, _) = ensure_sccache_available()?;
    let server_port = resolved_server_port()?;
    let basedirs = resolved_basedirs(&args.basedirs)?;
    let temp_target_dir = redirected_target_dir(
        &args.cargo_args,
        std::env::var_os("CARGO_TARGET_DIR").is_some(),
    )?;

    ensure_min_free_space(
        temp_target_dir
            .as_ref()
            .map(ScopedTempDir::path)
            .unwrap_or(std::path::Path::new(".")),
        "sccache",
    )?;

    let mut command = Command::new("cargo");
    command.args(&args.cargo_args);
    command.env("RUSTC_WRAPPER", &sccache_program);
    command.env("SCCACHE_SERVER_PORT", &server_port);
    if let Some(value) = basedirs.as_ref() {
        command.env("SCCACHE_BASEDIRS", value);
        println!("sccache: SCCACHE_BASEDIRS={}", value.to_string_lossy());
    }
    if let Some(target_dir) = temp_target_dir.as_ref() {
        command.env("CARGO_TARGET_DIR", target_dir.path());
        println!(
            "sccache: CARGO_TARGET_DIR={} (disposable)",
            target_dir.path().display()
        );
    }

    let disable_incremental = should_disable_incremental(args.keep_incremental);
    if disable_incremental {
        // sccache cannot reuse incrementally compiled Rust crates.
        command.env("CARGO_INCREMENTAL", "0");
    }

    println!(
        "sccache: RUSTC_WRAPPER={}",
        PathBuf::from(&sccache_program).display()
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
    let (sccache_program, version) = ensure_sccache_available()?;
    let port = resolved_server_port()?;
    let basedirs = resolved_basedirs(&args.basedirs)?;
    println!("sccache: found {version}");
    if PathBuf::from(&sccache_program).components().count() > 1 {
        println!(
            "sccache: using {}",
            PathBuf::from(&sccache_program).display()
        );
    }
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
    let (sccache_program, _) = ensure_sccache_available()?;
    let basedirs = resolved_basedirs(&config.basedirs)?;
    let mut command = Command::new(sccache_program);
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

fn ensure_sccache_available() -> Result<(OsString, String)> {
    sccache_command()
}

fn sccache_command() -> Result<(OsString, String)> {
    let program = OsString::from("sccache");
    if let Some(version) = probe_sccache(program.as_os_str())? {
        return Ok((program, version));
    }

    if let Some(path) = discover_windows_sccache_path()
        && let Some(version) = probe_sccache(path.as_os_str())?
    {
        return Ok((path.into_os_string(), version));
    }

    bail!("sccache is not installed. {}", install_hint());
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

fn probe_sccache(program: &OsStr) -> Result<Option<String>> {
    let display = PathBuf::from(program).display().to_string();
    let output = match Command::new(program).arg("--version").output() {
        Ok(output) => output,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to invoke `{display}`"));
        }
    };
    if !output.status.success() {
        bail!(
            "`{display} --version` failed (exit code: {}). {}",
            output.status.code().unwrap_or(-1),
            install_hint()
        );
    }
    Ok(Some(
        String::from_utf8_lossy(&output.stdout).trim().to_string(),
    ))
}

fn discover_windows_sccache_path() -> Option<PathBuf> {
    if !cfg!(windows) {
        return None;
    }

    let cargo_home = std::env::var_os("CARGO_HOME").map(PathBuf::from);
    let user_profile = std::env::var_os("USERPROFILE").map(PathBuf::from);
    let local_app_data = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);

    windows_explicit_sccache_candidates(cargo_home.as_deref(), user_profile.as_deref())
        .into_iter()
        .find(|path| path.is_file())
        .or_else(|| {
            let packages_root = local_app_data?
                .join("Microsoft")
                .join("WinGet")
                .join("Packages");
            winget_sccache_path(&packages_root)
        })
}

fn windows_explicit_sccache_candidates(
    cargo_home: Option<&Path>,
    user_profile: Option<&Path>,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(cargo_home) = cargo_home {
        candidates.push(cargo_home.join("bin").join("sccache.exe"));
    }
    if let Some(user_profile) = user_profile {
        candidates.push(user_profile.join(".cargo").join("bin").join("sccache.exe"));
        candidates.push(user_profile.join("scoop").join("shims").join("sccache.exe"));
    }
    candidates
}

fn winget_sccache_path(packages_root: &Path) -> Option<PathBuf> {
    let mut packages = read_dir_paths(packages_root).ok()?;
    packages.sort();
    packages.reverse();
    for package in packages {
        let name = package.file_name()?.to_string_lossy();
        if !name.starts_with("Mozilla.sccache") {
            continue;
        }

        let direct = package.join("sccache.exe");
        if direct.is_file() {
            return Some(direct);
        }

        let mut nested = read_dir_paths(&package).ok()?;
        nested.sort();
        nested.reverse();
        for child in nested {
            let candidate = child.join("sccache.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn read_dir_paths(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    std::fs::read_dir(root)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect()
}

fn redirected_target_dir(args: &[String], has_target_dir: bool) -> Result<Option<ScopedTempDir>> {
    if !should_use_ephemeral_target_dir(args, has_target_dir) {
        return Ok(None);
    }

    Ok(Some(ScopedTempDir::new("sccache-target")?))
}

fn default_server_port_for_key(key: &str) -> u16 {
    const PORT_BASE: u16 = 45_000;
    const PORT_SPAN: u16 = 1_000;

    PORT_BASE + (stable_hash64(key.as_bytes()) % u64::from(PORT_SPAN)) as u16
}

fn stable_hash64(bytes: &[u8]) -> u64 {
    // FNV-1a gives us a tiny, dependency-free stable hash for local port selection.
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn workspace_root() -> Result<std::path::PathBuf> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("failed to load cargo metadata for xtask workspace root")?;
    Ok(metadata.workspace_root.into_std_path_buf())
}

fn should_use_ephemeral_target_dir(args: &[String], has_target_dir: bool) -> bool {
    if has_target_dir {
        return false;
    }

    matches!(
        args.first().map(String::as_str),
        Some("check" | "clippy" | "test")
    )
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
        should_disable_incremental, should_use_ephemeral_target_dir, stable_hash64,
        windows_explicit_sccache_candidates, winget_sccache_path,
    };
    use std::ffi::OsStr;
    use std::path::{Path, PathBuf};

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
    fn stable_hash64_is_fixed_for_known_input() {
        assert_eq!(
            stable_hash64(b"C:/Code/Rust/tokmd"),
            0x1016ae5c107f4a1b,
            "stable hash should not drift across toolchain upgrades"
        );
    }

    #[test]
    fn windows_explicit_candidates_cover_known_user_installs() {
        let cargo_home = Path::new("C:/Users/steven/.cargo");
        let user_profile = Path::new("C:/Users/steven");

        let candidates = windows_explicit_sccache_candidates(Some(cargo_home), Some(user_profile));

        assert_eq!(
            candidates,
            vec![
                PathBuf::from("C:/Users/steven/.cargo/bin/sccache.exe"),
                PathBuf::from("C:/Users/steven/.cargo/bin/sccache.exe"),
                PathBuf::from("C:/Users/steven/scoop/shims/sccache.exe"),
            ]
        );
    }

    #[test]
    fn winget_sccache_path_discovers_nested_binary() {
        let root = temp_dir("winget-sccache");
        let package = root.join("Mozilla.sccache_Microsoft.Winget.Source_8wekyb3d8bbwe");
        let version = package.join("sccache-v0.10.0-x86_64-pc-windows-msvc");
        std::fs::create_dir_all(&version).unwrap();
        let binary = version.join("sccache.exe");
        std::fs::write(&binary, b"").unwrap();

        assert_eq!(winget_sccache_path(&root), Some(binary));
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
    fn should_use_ephemeral_target_dir_for_heavy_validation_commands() {
        let args = vec![
            "test".to_string(),
            "-p".to_string(),
            "xtask".to_string(),
            "--no-run".to_string(),
        ];

        assert!(should_use_ephemeral_target_dir(&args, false));
        assert!(!should_use_ephemeral_target_dir(&args, true));
    }

    #[test]
    fn should_not_use_ephemeral_target_dir_for_build_runs() {
        let args = vec!["build".to_string(), "-p".to_string(), "tokmd".to_string()];
        assert!(!should_use_ephemeral_target_dir(&args, false));
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
