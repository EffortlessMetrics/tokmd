//! Human-readable rendering of resolved configuration for `--show-config`.
//!
//! This surface is diagnostic only. It reports the configuration sources and
//! profile-layered values that `tokmd` already resolves at startup, so users
//! can see which `tokmd.toml`, legacy JSON config, or profile is actually in
//! effect. It must not emit or alter any receipt or machine-readable output.

use std::io::Write;

use anyhow::Result;

use super::{ConfigContext, ResolvedConfig};

/// Where the active profile/view name was selected from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileSource {
    /// Selected via the `--profile` / `--view` CLI flag.
    Cli,
    /// Selected via the `TOKMD_PROFILE` environment variable.
    Env,
    /// No profile selector was provided.
    None,
}

impl ProfileSource {
    fn label(self) -> &'static str {
        match self {
            ProfileSource::Cli => "--profile",
            ProfileSource::Env => "TOKMD_PROFILE",
            ProfileSource::None => "(none)",
        }
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn opt_str(value: Option<&str>) -> String {
    value.map_or_else(|| "(default)".to_string(), str::to_string)
}

fn opt_display<T: std::fmt::Display>(value: Option<T>) -> String {
    value.map_or_else(|| "(default)".to_string(), |v| v.to_string())
}

fn opt_list(value: Option<Vec<String>>) -> String {
    match value {
        Some(v) if !v.is_empty() => v.join(", "),
        Some(_) => "(empty)".to_string(),
        None => "(default)".to_string(),
    }
}

/// Render the resolved configuration report to the given sink.
///
/// This is the testable core; [`print_config_report`] is the thin stdout
/// wrapper used by the CLI.
pub fn write_config_report<W: Write>(
    out: &mut W,
    ctx: &ConfigContext,
    profile_name: Option<&str>,
    profile_source: ProfileSource,
    resolved: &ResolvedConfig,
) -> Result<()> {
    writeln!(out, "tokmd configuration")?;
    writeln!(out)?;
    writeln!(out, "Config sources (in precedence order):")?;
    match &ctx.toml_path {
        Some(path) => writeln!(out, "  TOML config:    {}", path.display())?,
        None => writeln!(out, "  TOML config:    (none found)")?,
    }
    writeln!(
        out,
        "  Legacy JSON:    {}",
        if ctx.json.is_some() {
            "loaded (~/.config/tokmd/config.json)"
        } else {
            "(none)"
        }
    )?;

    writeln!(out)?;
    writeln!(out, "Active profile:")?;
    match profile_name {
        Some(name) => {
            writeln!(
                out,
                "  name:           {name} (from {})",
                profile_source.label()
            )?;
            let matched_view = resolved.toml_view.is_some();
            let matched_json = resolved.json_profile.is_some();
            writeln!(out, "  matched TOML view:     {}", yes_no(matched_view))?;
            writeln!(out, "  matched JSON profile:  {}", yes_no(matched_json))?;
            if !matched_view && !matched_json {
                writeln!(
                    out,
                    "  note: profile \"{name}\" did not match any TOML view or JSON profile"
                )?;
            }
        }
        None => writeln!(out, "  name:           (none)")?,
    }

    writeln!(out)?;
    writeln!(out, "Resolved values:")?;
    writeln!(out, "  format:         {}", opt_str(resolved.format()))?;
    writeln!(out, "  top:            {}", opt_display(resolved.top()))?;
    writeln!(out, "  files:          {}", opt_display(resolved.files()))?;
    writeln!(
        out,
        "  module_roots:   {}",
        opt_list(resolved.module_roots())
    )?;
    writeln!(
        out,
        "  module_depth:   {}",
        opt_display(resolved.module_depth())
    )?;
    writeln!(out, "  children:       {}", opt_str(resolved.children()))?;
    writeln!(
        out,
        "  min_code:       {}",
        opt_display(resolved.min_code())
    )?;
    writeln!(
        out,
        "  max_rows:       {}",
        opt_display(resolved.max_rows())
    )?;
    writeln!(out, "  redact:         {}", opt_str(resolved.redact()))?;
    writeln!(out, "  meta:           {}", opt_display(resolved.meta()))?;

    Ok(())
}

/// Print the resolved configuration report to stdout.
pub fn print_config_report(
    ctx: &ConfigContext,
    profile_name: Option<&str>,
    profile_source: ProfileSource,
    resolved: &ResolvedConfig,
) -> Result<()> {
    let mut stdout = std::io::stdout().lock();
    write_config_report(&mut stdout, ctx, profile_name, profile_source, resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use tokmd_settings::Profile;

    fn render(
        ctx: &ConfigContext,
        name: Option<&str>,
        source: ProfileSource,
        resolved: &ResolvedConfig,
    ) -> Result<String> {
        let mut buf = Vec::new();
        write_config_report(&mut buf, ctx, name, source, resolved)?;
        Ok(String::from_utf8(buf)?)
    }

    #[test]
    fn config_explain_reports_no_sources() -> Result<()> {
        let ctx = ConfigContext::default();
        let resolved = ResolvedConfig::default();
        let out = render(&ctx, None, ProfileSource::None, &resolved)?;
        assert!(out.contains("TOML config:    (none found)"), "{out}");
        assert!(out.contains("Legacy JSON:    (none)"), "{out}");
        assert!(out.contains("name:           (none)"), "{out}");
        assert!(out.contains("format:         (default)"), "{out}");
        assert!(out.contains("module_roots:   (default)"), "{out}");
        Ok(())
    }

    #[test]
    fn config_explain_flags_unmatched_profile() -> Result<()> {
        let ctx = ConfigContext::default();
        let resolved = ResolvedConfig::default();
        let out = render(&ctx, Some("ci"), ProfileSource::Cli, &resolved)?;
        assert!(out.contains("name:           ci (from --profile)"), "{out}");
        assert!(out.contains("matched TOML view:     no"), "{out}");
        assert!(out.contains("did not match"), "{out}");
        Ok(())
    }

    #[test]
    fn config_explain_shows_resolved_profile_values() -> Result<()> {
        let profile = Profile {
            format: Some("json".to_string()),
            top: Some(5),
            ..Profile::default()
        };
        let resolved = ResolvedConfig {
            toml_view: None,
            json_profile: Some(&profile),
            toml: None,
        };
        let ctx = ConfigContext::default();
        let out = render(&ctx, Some("default"), ProfileSource::Env, &resolved)?;
        assert!(
            out.contains("name:           default (from TOKMD_PROFILE)"),
            "{out}"
        );
        assert!(out.contains("matched JSON profile:  yes"), "{out}");
        assert!(out.contains("format:         json"), "{out}");
        assert!(out.contains("top:            5"), "{out}");
        assert!(!out.contains("did not match"), "{out}");
        Ok(())
    }
}
