use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::Result;
use tokmd_config as cli;

/// Exit codes for check-ignore:
/// - 0: Path is ignored
/// - 1: Path is not ignored
/// - 2: Error occurred
const EXIT_IGNORED: i32 = 0;
const EXIT_NOT_IGNORED: i32 = 1;

pub(crate) fn handle(args: cli::CliCheckIgnoreArgs, global: &cli::GlobalArgs) -> Result<()> {
    let mut any_ignored = false;
    let mut any_not_ignored = false;

    for path in &args.paths {
        let result = check_path(path, global, args.verbose)?;
        if result.ignored {
            any_ignored = true;
        } else {
            any_not_ignored = true;
        }
        print_result(&result, args.verbose);
    }

    // Exit code: 0 if all ignored, 1 if any not ignored
    if any_not_ignored {
        std::process::exit(EXIT_NOT_IGNORED);
    } else if any_ignored {
        std::process::exit(EXIT_IGNORED);
    }

    Ok(())
}

struct CheckResult {
    path: String,
    ignored: bool,
    reasons: Vec<IgnoreReason>,
}

#[derive(Clone)]
enum IgnoreReason {
    Git {
        source: String,
        pattern: String,
        line: Option<usize>,
    },
    GitTracked, // File is tracked by git; gitignore rules don't apply
    Tokeignore {
        pattern: String,
    },
    ExcludeFlag {
        pattern: String,
    },
    NotFound,
}

fn check_path(path: &Path, global: &cli::GlobalArgs, verbose: bool) -> Result<CheckResult> {
    let path_str = path.display().to_string();
    let mut reasons = Vec::new();
    let mut ignored = false;

    // Check if path exists
    if !path.exists() {
        reasons.push(IgnoreReason::NotFound);
        return Ok(CheckResult {
            path: path_str,
            ignored: false,
            reasons,
        });
    }

    // 1. Check git ignore (if git is available and we're in a repo)
    if let Some(git_reason) = check_git_ignore(path, verbose) {
        reasons.push(git_reason);
        ignored = true;
    } else if is_git_tracked(path) {
        // File is tracked by git; gitignore rules don't apply
        reasons.push(IgnoreReason::GitTracked);
    }

    // 2. Check --exclude patterns from CLI
    for pattern in &global.excluded {
        if matches_glob(pattern, &path_str) {
            reasons.push(IgnoreReason::ExcludeFlag {
                pattern: pattern.clone(),
            });
            ignored = true;
        }
    }

    // 3. Check .tokeignore
    if let Some(tokeignore_reason) = check_tokeignore(path, &path_str) {
        reasons.push(tokeignore_reason);
        ignored = true;
    }

    // 4. Note if ignore flags would affect this
    if !ignored && verbose {
        if global.no_ignore {
            eprintln!("  note: --no-ignore is set, ignores disabled");
        }
        if global.no_ignore_vcs {
            eprintln!("  note: --no-ignore-vcs is set, VCS ignores disabled");
        }
        if global.no_ignore_dot {
            eprintln!("  note: --no-ignore-dot is set, .ignore/.tokeignore disabled");
        }
    }

    Ok(CheckResult {
        path: path_str,
        ignored,
        reasons,
    })
}

fn check_git_ignore(path: &Path, verbose: bool) -> Option<IgnoreReason> {
    // Try to use git check-ignore -v
    let output = Command::new("git")
        .args(["check-ignore", "-v", "--"])
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .ok()?;

    if output.status.success() {
        // Parse output: <source>:<line>:<pattern>\t<path>
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next()?;

        // Format: ".gitignore:5:*.log\tpath/to/file.log"
        if let Some(tab_pos) = line.find('\t') {
            let source_part = &line[..tab_pos];
            if let Some(last_colon) = source_part.rfind(':') {
                let pattern = source_part[last_colon + 1..].to_string();
                let rest = &source_part[..last_colon];

                let (source, line_num) = if let Some(colon) = rest.rfind(':') {
                    let src = rest[..colon].to_string();
                    let ln = rest[colon + 1..].parse::<usize>().ok();
                    (src, ln)
                } else {
                    (rest.to_string(), None)
                };

                return Some(IgnoreReason::Git {
                    source,
                    pattern,
                    line: line_num,
                });
            }
        }

        // Fallback: just report it's ignored by git
        if verbose {
            return Some(IgnoreReason::Git {
                source: "git".to_string(),
                pattern: stdout.trim().to_string(),
                line: None,
            });
        }

        return Some(IgnoreReason::Git {
            source: "(unknown)".to_string(),
            pattern: "(unknown)".to_string(),
            line: None,
        });
    }

    None
}

fn is_git_tracked(path: &Path) -> bool {
    Command::new("git")
        .args(["ls-files", "--error-unmatch", "--"])
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn check_tokeignore(base_path: &Path, path_str: &str) -> Option<IgnoreReason> {
    // Look for .tokeignore in current directory and parents
    let mut dir = base_path.parent();
    while let Some(d) = dir {
        let tokeignore = d.join(".tokeignore");
        if let Ok(content) = std::fs::read_to_string(&tokeignore) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if matches_glob(line, path_str) {
                    return Some(IgnoreReason::Tokeignore {
                        pattern: line.to_string(),
                    });
                }
            }
        }
        dir = d.parent();
    }

    // Also check current working directory
    let cwd_tokeignore = Path::new(".tokeignore");
    if let Ok(content) = std::fs::read_to_string(cwd_tokeignore) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if matches_glob(line, path_str) {
                return Some(IgnoreReason::Tokeignore {
                    pattern: line.to_string(),
                });
            }
        }
    }

    None
}

fn matches_glob(pattern: &str, path: &str) -> bool {
    // Simple glob matching (could use the glob crate for more accuracy)
    // Handle common patterns: *, **, ?

    // Normalize path separators
    let path = path.replace('\\', "/");
    let pattern = pattern.replace('\\', "/");

    // Handle negation
    let (negated, pattern) = if let Some(stripped) = pattern.strip_prefix('!') {
        (true, stripped)
    } else {
        (false, pattern.as_str())
    };

    let matches = if pattern.contains("**") {
        // Double-star: match any path segments
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');

            let prefix_matches = prefix.is_empty() || path.starts_with(prefix);
            let suffix_matches = suffix.is_empty() || path.ends_with(suffix);
            prefix_matches && suffix_matches
        } else {
            false
        }
    } else if pattern.contains('*') {
        // Single star: match within path segment
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            path.starts_with(parts[0]) && path.ends_with(parts[1])
        } else {
            // Multiple stars - simplistic handling
            path.contains(pattern.replace('*', "").as_str())
        }
    } else {
        // Exact match or suffix match
        path == pattern
            || path.ends_with(&format!("/{}", pattern))
            || path.starts_with(&format!("{}/", pattern))
    };

    if negated { !matches } else { matches }
}

fn print_result(result: &CheckResult, verbose: bool) {
    if result.ignored {
        println!("{}: ignored", result.path);
        if verbose {
            for reason in &result.reasons {
                match reason {
                    IgnoreReason::Git {
                        source,
                        pattern,
                        line,
                    } => {
                        if let Some(ln) = line {
                            println!("  gitignore: {}:{} -> {}", source, ln, pattern);
                        } else {
                            println!("  gitignore: {} -> {}", source, pattern);
                        }
                    }
                    IgnoreReason::Tokeignore { pattern } => {
                        println!("  .tokeignore: {}", pattern);
                    }
                    IgnoreReason::ExcludeFlag { pattern } => {
                        println!("  --exclude: {}", pattern);
                    }
                    IgnoreReason::GitTracked => {
                        println!("  git: tracked (gitignore rules don't apply)");
                    }
                    IgnoreReason::NotFound => {
                        println!("  (file not found)");
                    }
                }
            }
        }
    } else {
        println!("{}: not ignored", result.path);
        if verbose {
            for reason in &result.reasons {
                match reason {
                    IgnoreReason::NotFound => println!("  (file not found)"),
                    IgnoreReason::GitTracked => {
                        println!("  note: tracked by git; gitignore rules don't apply");
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokmd_config::GlobalArgs;

    #[test]
    fn matches_glob_handles_double_star() {
        assert!(matches_glob("**/main.rs", "src/main.rs"));
        assert!(matches_glob("src/**", "src/nested/main.rs"));
    }

    #[test]
    fn matches_glob_handles_single_star() {
        assert!(matches_glob("*.rs", "src/main.rs"));
        assert!(matches_glob("src/*.rs", "src/main.rs"));
    }

    #[test]
    fn matches_glob_handles_negation() {
        assert!(!matches_glob("!*.rs", "src/main.rs"));
        assert!(matches_glob("!*.js", "src/main.rs"));
    }

    #[test]
    fn check_tokeignore_matches_parent_dir() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(dir.path().join(".tokeignore"), "*.rs\n").unwrap();

        let file_path = src_dir.join("main.rs");
        std::fs::write(&file_path, "fn main() {}\n").unwrap();

        let reason = check_tokeignore(&file_path, file_path.to_string_lossy().as_ref());
        assert!(matches!(
            reason,
            Some(IgnoreReason::Tokeignore { pattern }) if pattern == "*.rs"
        ));
    }

    #[test]
    fn check_path_reports_not_found() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing.rs");
        let result = check_path(&missing, &GlobalArgs::default(), false).unwrap();
        assert!(!result.ignored);
        assert!(
            result
                .reasons
                .iter()
                .any(|r| matches!(r, IgnoreReason::NotFound))
        );
    }

    #[test]
    fn check_path_honors_exclude_flag() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("skip.rs");
        std::fs::write(&file_path, "fn skip() {}\n").unwrap();

        let mut global = GlobalArgs::default();
        global.excluded.push("*.rs".to_string());

        let result = check_path(&file_path, &global, false).unwrap();
        assert!(result.ignored);
        assert!(
            result.reasons.iter().any(|r| {
                matches!(r, IgnoreReason::ExcludeFlag { pattern } if pattern == "*.rs")
            })
        );
    }
}
