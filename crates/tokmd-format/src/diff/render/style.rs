//! Delta formatting and optional ANSI coloring for diff Markdown.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffColorMode {
    Off,
    Ansi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffRenderOptions {
    pub compact: bool,
    pub color: DiffColorMode,
}

impl Default for DiffRenderOptions {
    fn default() -> Self {
        Self {
            compact: false,
            color: DiffColorMode::Off,
        }
    }
}

pub(super) fn format_delta(delta: i64) -> String {
    if delta > 0 {
        format!("+{}", delta)
    } else {
        delta.to_string()
    }
}

pub(super) fn format_delta_colored(delta: i64, mode: DiffColorMode) -> String {
    let raw = format_delta(delta);
    if mode == DiffColorMode::Off {
        return raw;
    }
    if delta > 0 {
        format!("\x1b[32m{}\x1b[0m", raw)
    } else if delta < 0 {
        format!("\x1b[31m{}\x1b[0m", raw)
    } else {
        format!("\x1b[33m{}\x1b[0m", raw)
    }
}

pub(super) fn format_pct_delta_colored(delta_pct: f64, mode: DiffColorMode) -> String {
    let raw = format!("{:+.1}%", delta_pct);
    if mode == DiffColorMode::Off {
        return raw;
    }
    if delta_pct > 0.0 {
        format!("\x1b[32m{}\x1b[0m", raw)
    } else if delta_pct < 0.0 {
        format!("\x1b[31m{}\x1b[0m", raw)
    } else {
        format!("\x1b[33m{}\x1b[0m", raw)
    }
}

#[cfg(test)]
mod tests {
    use super::format_delta;

    #[test]
    fn test_format_delta() {
        // Kills mutants in format_delta function
        assert_eq!(format_delta(5), "+5");
        assert_eq!(format_delta(0), "0");
        assert_eq!(format_delta(-3), "-3");
    }
}
