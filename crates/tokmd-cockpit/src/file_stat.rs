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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filestat_as_ref() {
        let stat = FileStat {
            path: "src/main.rs".to_string(),
            insertions: 10,
            deletions: 5,
        };
        let s: &str = stat.as_ref();
        assert_eq!(s, "src/main.rs");
    }
}
