//! LCOV report parsing.
//!
//! Parses the `lcov.info` line-coverage format into a per-file lookup of
//! `line number → hit count`. Records are normalised to repo-relative paths
//! and merged when the same `SF:` reappears (a quirk some tools emit).

use std::collections::BTreeMap;
use std::path::Path;

/// `file path → { line number → hit count }`.
pub(super) type LcovData = BTreeMap<String, BTreeMap<usize, usize>>;

/// Parse the textual contents of an `lcov.info` file.
///
/// `repo_root` is used to make absolute `SF:` paths repo-relative.
pub(super) fn parse_lcov(repo_root: &Path, content: &str) -> LcovData {
    let mut lcov_data: LcovData = BTreeMap::new();
    let mut current_file: Option<String> = None;
    let mut current_lines: BTreeMap<usize, usize> = BTreeMap::new();

    for line in content.lines() {
        if let Some(sf) = line.strip_prefix("SF:") {
            current_file = Some(normalize_source_path(repo_root, sf));
            current_lines.clear();
        } else if let Some(da) = line.strip_prefix("DA:") {
            if current_file.is_some()
                && let Some((line_no, count)) = parse_da_record(da)
            {
                current_lines.insert(line_no, count);
            }
        } else if line == "end_of_record"
            && let Some(file) = current_file.take()
        {
            let lines = std::mem::take(&mut current_lines);
            merge_record(&mut lcov_data, file, lines);
        }
    }

    // Some generators omit the trailing `end_of_record`; flush any pending
    // record so its data is not silently dropped.
    if let Some(file) = current_file.take() {
        let lines = std::mem::take(&mut current_lines);
        merge_record(&mut lcov_data, file, lines);
    }

    lcov_data
}

fn normalize_source_path(repo_root: &Path, raw: &str) -> String {
    let path = raw.replace('\\', "/");
    if let Ok(abs) = Path::new(&path).canonicalize()
        && let Ok(rel) = abs.strip_prefix(repo_root.canonicalize().unwrap_or_default())
    {
        return rel.to_string_lossy().replace('\\', "/");
    }
    path
}

fn parse_da_record(da: &str) -> Option<(usize, usize)> {
    let (line_no, count) = da.split_once(',')?;
    Some((line_no.parse().ok()?, count.parse().ok()?))
}

fn merge_record(lcov_data: &mut LcovData, file: String, lines: BTreeMap<usize, usize>) {
    match lcov_data.entry(file) {
        std::collections::btree_map::Entry::Occupied(mut entry) => {
            entry.get_mut().extend(lines);
        }
        std::collections::btree_map::Entry::Vacant(entry) => {
            entry.insert(lines);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> std::path::PathBuf {
        // SF: paths in these fixtures must NOT exist relative to cwd, otherwise
        // `normalize_source_path` will canonicalize them and the key in the
        // result map becomes an absolute path. We use a deliberately exotic
        // prefix so the canonicalize attempt always fails and the raw repo-
        // relative path is preserved verbatim.
        std::path::PathBuf::from("/__tokmd_lcov_test_nonexistent_root__")
    }

    const FAKE_PATH: &str = "__tokmd_lcov_test_nonexistent_src__/lib.rs";

    #[test]
    fn parses_single_record() {
        let content = format!("SF:{FAKE_PATH}\nDA:1,1\nDA:2,0\nend_of_record\n");
        let data = parse_lcov(&root(), &content);
        let file = data.get(FAKE_PATH).expect("file present");
        assert_eq!(file.get(&1), Some(&1));
        assert_eq!(file.get(&2), Some(&0));
    }

    #[test]
    fn flushes_unterminated_final_record() {
        let content = format!("SF:{FAKE_PATH}\nDA:2,1\n");
        let data = parse_lcov(&root(), &content);
        assert_eq!(data.get(FAKE_PATH).and_then(|m| m.get(&2)), Some(&1));
    }

    #[test]
    fn merges_duplicate_records_for_same_file() {
        let content = format!(
            "SF:{FAKE_PATH}\nDA:1,1\nend_of_record\nSF:{FAKE_PATH}\nDA:2,3\nend_of_record\n"
        );
        let data = parse_lcov(&root(), &content);
        let file = data.get(FAKE_PATH).expect("file present");
        assert_eq!(file.get(&1), Some(&1));
        assert_eq!(file.get(&2), Some(&3));
    }

    #[test]
    fn skips_malformed_da_lines() {
        let content = format!("SF:{FAKE_PATH}\nDA:abc\nDA:1,xyz\nDA:3,7\nend_of_record\n");
        let data = parse_lcov(&root(), &content);
        let file = data.get(FAKE_PATH).expect("file present");
        assert_eq!(file.len(), 1);
        assert_eq!(file.get(&3), Some(&7));
    }

    #[test]
    fn ignores_da_before_sf() {
        let content = format!("DA:1,1\nSF:{FAKE_PATH}\nDA:2,1\nend_of_record\n");
        let data = parse_lcov(&root(), &content);
        let file = data.get(FAKE_PATH).expect("file present");
        assert_eq!(file.len(), 1);
        assert_eq!(file.get(&2), Some(&1));
    }
}
