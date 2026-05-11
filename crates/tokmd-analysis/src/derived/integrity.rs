use tokmd_analysis_types::IntegrityReport;
use tokmd_types::FileRow;

pub(super) fn build_integrity_report(rows: &[&FileRow]) -> IntegrityReport {
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort_unstable_by(|&a, &b| compare_integrity_rows(a, b));

    let mut hasher = blake3::Hasher::new();
    let mut first = true;
    for row in sorted_rows {
        if !first {
            hasher.update(b"\n");
        }
        first = false;
        hasher.update(row.path.as_bytes());
        hasher.update(b":");
        hasher.update(row.bytes.to_string().as_bytes());
        hasher.update(b":");
        hasher.update(row.lines.to_string().as_bytes());
    }

    IntegrityReport {
        algo: "blake3".to_string(),
        hash: hasher.finalize().to_hex().to_string(),
        entries: rows.len(),
    }
}

fn compare_integrity_rows(a: &FileRow, b: &FileRow) -> std::cmp::Ordering {
    let a_bytes = a.path.as_bytes();
    let b_bytes = b.path.as_bytes();
    let min_len = a_bytes.len().min(b_bytes.len());

    let ord = a_bytes[..min_len].cmp(&b_bytes[..min_len]);
    if ord != std::cmp::Ordering::Equal {
        return ord;
    }

    if a_bytes.len() == b_bytes.len() {
        return compare_usize_pair_ascii(a.bytes, a.lines, b.bytes, b.lines);
    }

    if a_bytes.len() < b_bytes.len() {
        b':'.cmp(&b_bytes[min_len])
    } else {
        a_bytes[min_len].cmp(&b':')
    }
}

fn compare_usize_pair_ascii(
    a_bytes: usize,
    a_lines: usize,
    b_bytes: usize,
    b_lines: usize,
) -> std::cmp::Ordering {
    let mut a_buf = [0_u8; 64];
    let mut b_buf = [0_u8; 64];
    let a_len = write_usize_pair_ascii(&mut a_buf, a_bytes, a_lines);
    let b_len = write_usize_pair_ascii(&mut b_buf, b_bytes, b_lines);
    a_buf[..a_len].cmp(&b_buf[..b_len])
}

fn write_usize_pair_ascii(buf: &mut [u8; 64], bytes: usize, lines: usize) -> usize {
    let mut len = write_usize_ascii(&mut buf[..], bytes);
    buf[len] = b':';
    len += 1;
    len + write_usize_ascii(&mut buf[len..], lines)
}

fn write_usize_ascii(buf: &mut [u8], value: usize) -> usize {
    if value == 0 {
        buf[0] = b'0';
        return 1;
    }

    let mut digits = 0;
    let mut n = value;
    while n > 0 {
        digits += 1;
        n /= 10;
    }

    debug_assert!(digits <= buf.len());

    let mut n = value;
    for idx in (0..digits).rev() {
        buf[idx] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    digits
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_types::FileKind;

    fn make_row(path: &str, bytes: usize, lines: usize) -> FileRow {
        FileRow {
            path: path.to_string(),
            module: "mod".to_string(),
            lang: "rust".to_string(),
            kind: FileKind::Parent,
            code: 0,
            comments: 0,
            blanks: 0,
            lines,
            bytes,
            tokens: 0,
        }
    }

    #[test]
    fn compare_integrity_rows_matches_string_sort() {
        let cases = vec![
            ("a", 10, 10, "b", 10, 10),
            ("a", 10, 10, "a", 10, 10),
            ("a", 10, 10, "a", 20, 10),
            ("a", 100, 10, "a", 20, 10),
            ("a", 10, 10, "a.b", 10, 10),
            ("a.b", 10, 10, "a", 10, 10),
            ("foo", 10, 10, "foo.bar", 10, 10),
            ("foo.bar", 10, 10, "foo", 10, 10),
            ("foo", 10, 10, "foo_bar", 10, 10),
            ("a", usize::MAX, 0, "a", 9, usize::MAX),
            ("a", 0, usize::MAX, "a", 0, 9),
            ("a", 999, 0, "a", 1000, 0),
            ("a", 10, 2, "a", 10, 10),
        ];

        for (p1, b1, l1, p2, b2, l2) in cases {
            let r1 = make_row(p1, b1, l1);
            let r2 = make_row(p2, b2, l2);

            let s1 = format!("{p1}:{b1}:{l1}");
            let s2 = format!("{p2}:{b2}:{l2}");
            let expected = s1.cmp(&s2);
            let actual = compare_integrity_rows(&r1, &r2);

            assert_eq!(actual, expected, "Failed for {s1} vs {s2}");
        }
    }
}
