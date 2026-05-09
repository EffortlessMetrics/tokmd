//! File-level diff statistics used by cockpit metrics.

/// File stat from git diff --numstat.
#[derive(Debug, Clone)]
pub struct FileStat {
    pub path: String,
    pub insertions: usize,
    pub deletions: usize,
}

impl AsRef<str> for FileStat {
    fn as_ref(&self) -> &str {
        &self.path
    }
}
