//! Deterministic row sorting helpers for model receipts.

use tokmd_types::{FileRow, LangRow, ModuleRow};

pub(crate) fn sort_lang_rows(rows: &mut [LangRow]) {
    rows.sort_unstable_by(|a, b| b.code.cmp(&a.code).then_with(|| a.lang.cmp(&b.lang)));
}

pub(crate) fn sort_module_rows(rows: &mut [ModuleRow]) {
    rows.sort_unstable_by(|a, b| b.code.cmp(&a.code).then_with(|| a.module.cmp(&b.module)));
}

pub(crate) fn sort_file_rows(rows: &mut [FileRow]) {
    rows.sort_unstable_by(|a, b| b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_types::FileKind;

    fn lang_row(lang: &str, code: usize) -> LangRow {
        LangRow {
            lang: lang.to_string(),
            code,
            lines: code,
            files: 1,
            bytes: 0,
            tokens: 0,
            avg_lines: code,
        }
    }

    fn module_row(module: &str, code: usize) -> ModuleRow {
        ModuleRow {
            module: module.to_string(),
            code,
            lines: code,
            files: 1,
            bytes: 0,
            tokens: 0,
            avg_lines: code,
        }
    }

    fn file_row(path: &str, code: usize) -> FileRow {
        FileRow {
            path: path.to_string(),
            module: "(root)".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code,
            comments: 0,
            blanks: 0,
            lines: code,
            bytes: 0,
            tokens: 0,
        }
    }

    #[test]
    fn lang_rows_sort_unstable_by_code_desc_then_name() {
        let mut rows = vec![
            lang_row("TypeScript", 10),
            lang_row("Rust", 20),
            lang_row("Python", 10),
        ];

        sort_lang_rows(&mut rows);

        assert_eq!(
            rows.into_iter().map(|row| row.lang).collect::<Vec<_>>(),
            ["Rust", "Python", "TypeScript"]
        );
    }

    #[test]
    fn module_rows_sort_unstable_by_code_desc_then_name() {
        let mut rows = vec![
            module_row("web", 12),
            module_row("crates", 20),
            module_row("docs", 12),
        ];

        sort_module_rows(&mut rows);

        assert_eq!(
            rows.into_iter().map(|row| row.module).collect::<Vec<_>>(),
            ["crates", "docs", "web"]
        );
    }

    #[test]
    fn file_rows_sort_unstable_by_code_desc_then_path() {
        let mut rows = vec![
            file_row("src/z.rs", 8),
            file_row("src/a.rs", 8),
            file_row("src/lib.rs", 20),
        ];

        sort_file_rows(&mut rows);

        assert_eq!(
            rows.into_iter().map(|row| row.path).collect::<Vec<_>>(),
            ["src/lib.rs", "src/a.rs", "src/z.rs"]
        );
    }
}
