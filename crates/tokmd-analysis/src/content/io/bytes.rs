//! Byte-level content helpers for text detection, hashing, and entropy.

use std::path::Path;

use anyhow::Result;

pub fn as_text(bytes: &[u8]) -> Option<&str> {
    if bytes.contains(&0) {
        return None;
    }
    std::str::from_utf8(bytes).ok()
}


pub fn hash_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub fn hash_file(path: &Path, max_bytes: usize) -> Result<String> {
    let bytes = super::read_head(path, max_bytes)?;
    Ok(hash_bytes(&bytes))
}

pub fn entropy_bits_per_byte(bytes: &[u8]) -> f32 {
    if bytes.is_empty() {
        return 0.0;
    }
    let mut counts = [0u32; 256];
    for b in bytes {
        counts[*b as usize] += 1;
    }
    let len = bytes.len() as f32;
    let mut entropy = 0.0f32;
    for count in counts {
        if count == 0 {
            continue;
        }
        let p = count as f32 / len;
        entropy -= p * p.log2();
    }
    entropy
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use super::*;

    #[test]
    fn hash_file_returns_correct_blake3_hash() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("hash_test.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"test content").unwrap();

        let hash = hash_file(&path, 1000).unwrap();

        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64);
        let expected = hash_bytes(b"test content");
        assert_eq!(hash, expected);
    }

    #[test]
    fn hash_file_deterministic() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("deterministic.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"same content every time").unwrap();

        let hash1 = hash_file(&path, 1000).unwrap();
        let hash2 = hash_file(&path, 1000).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn hash_file_different_content_different_hash() {
        let tmp = tempfile::tempdir().unwrap();

        let path1 = tmp.path().join("file1.txt");
        let mut f1 = File::create(&path1).unwrap();
        f1.write_all(b"content A").unwrap();

        let path2 = tmp.path().join("file2.txt");
        let mut f2 = File::create(&path2).unwrap();
        f2.write_all(b"content B").unwrap();

        let hash1 = hash_file(&path1, 1000).unwrap();
        let hash2 = hash_file(&path2, 1000).unwrap();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn hash_file_respects_max_bytes() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("long_file.txt");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"abcdefghij").unwrap();

        let hash_limited = hash_file(&path, 5).unwrap();
        let expected = hash_bytes(b"abcde");
        assert_eq!(hash_limited, expected);

        let hash_full = hash_file(&path, 1000).unwrap();
        assert_ne!(hash_limited, hash_full);
    }
}
