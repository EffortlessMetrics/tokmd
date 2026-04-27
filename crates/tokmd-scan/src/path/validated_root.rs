use std::fs;
use std::path::{Path, PathBuf};

use super::RootViolation;

#[derive(Debug, Clone)]
pub(crate) struct ValidatedRoot {
    input: PathBuf,
    canonical: PathBuf,
}

impl ValidatedRoot {
    pub(crate) fn new(path: impl AsRef<Path>) -> Result<Self, RootViolation> {
        let input = path.as_ref().to_path_buf();
        if input.as_os_str().is_empty() {
            return Err(RootViolation::Empty);
        }
        if !input.exists() {
            return Err(RootViolation::Missing(input));
        }

        let canonical =
            fs::canonicalize(&input).map_err(|source| RootViolation::CanonicalizeFailed {
                path: input.clone(),
                source,
            })?;

        Ok(Self { input, canonical })
    }

    pub(crate) fn input(&self) -> &Path {
        &self.input
    }

    pub(crate) fn canonical(&self) -> &Path {
        &self.canonical
    }
}
