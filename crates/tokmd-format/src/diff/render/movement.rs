//! Language movement classification for diff Markdown rendering.

use tokmd_types::DiffRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LanguageMovement {
    pub changed: usize,
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
}

impl LanguageMovement {
    pub fn from_rows(rows: &[DiffRow]) -> Self {
        let added = rows
            .iter()
            .filter(|r| r.old_code == 0 && r.new_code > 0)
            .count();
        let removed = rows
            .iter()
            .filter(|r| r.old_code > 0 && r.new_code == 0)
            .count();
        let modified = rows.len().saturating_sub(added + removed);

        Self {
            changed: rows.len(),
            added,
            removed,
            modified,
        }
    }
}
