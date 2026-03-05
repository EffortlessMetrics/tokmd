//! Property-based tests for `MemFs` using proptest.

use std::path::Path;
use tokmd_io_port::{MemFs, ReadFs};

use proptest::prelude::*;
use proptest::collection::vec as pvec;

fn path_segment() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,7}".prop_map(|s| s)
}

fn forward_slash_path() -> impl Strategy<Value = String> {
    pvec(path_segment(), 1..=4).prop_map(|segments| segments.join("/"))
}

proptest! {
    #[test]
    fn prop_insert_bytes_roundtrip(
        path in forward_slash_path(),
        content in pvec(any::<u8>(), 0..1024),
    ) {
        let mut fs = MemFs::new();
        fs.add_bytes(&path, content.clone());
        let read_back = fs.read_bytes(Path::new(&path)).unwrap();
        prop_assert_eq!(read_back, content);
    }
}

proptest! {
    #[test]
    fn prop_insert_string_roundtrip(
        path in forward_slash_path(),
        content in ".*",
    ) {
        let mut fs = MemFs::new();
        fs.add_file(&path, content.clone());
        let read_back = fs.read_to_string(Path::new(&path)).unwrap();
        prop_assert_eq!(read_back, content);
    }
}

proptest! {
    #[test]
    fn prop_exists_consistency(
        path in forward_slash_path(),
        content in ".*",
    ) {
        let mut fs = MemFs::new();
        fs.add_file(&path, content);
        let p = Path::new(&path);
        prop_assert_eq!(fs.exists(p), fs.is_file(p) || fs.is_dir(p));
    }
}

proptest! {
    #[test]
    fn prop_file_dir_exclusive(
        path in forward_slash_path(),
        content in ".*",
    ) {
        let mut fs = MemFs::new();
        fs.add_file(&path, content);
        let p = Path::new(&path);
        prop_assert!(!(fs.is_file(p) && fs.is_dir(p)));
    }
}

proptest! {
    #[test]
    fn prop_stored_path_is_file(
        path in forward_slash_path(),
        content in pvec(any::<u8>(), 0..512),
    ) {
        let mut fs = MemFs::new();
        fs.add_bytes(&path, content);
        prop_assert!(fs.is_file(Path::new(&path)));
        prop_assert!(fs.exists(Path::new(&path)));
    }
}

proptest! {
    #[test]
    fn prop_overwrite_preserves_file(
        path in forward_slash_path(),
        content1 in pvec(any::<u8>(), 0..256),
        content2 in pvec(any::<u8>(), 0..256),
    ) {
        let mut fs = MemFs::new();
        fs.add_bytes(&path, content1);
        fs.add_bytes(&path, content2.clone());
        prop_assert!(fs.is_file(Path::new(&path)));
        let read_back = fs.read_bytes(Path::new(&path)).unwrap();
        prop_assert_eq!(read_back, content2);
    }
}

proptest! {
    #[test]
    fn prop_parent_dirs_exist(
        segments in pvec(path_segment(), 2..=4),
        content in ".*",
    ) {
        let path = segments.join("/");
        let mut fs = MemFs::new();
        fs.add_file(&path, content);
        for i in 1..segments.len() {
            let prefix = segments[..i].join("/");
            prop_assert!(
                fs.is_dir(Path::new(&prefix)),
                "expected {} to be a directory",
                prefix
            );
        }
    }
}

proptest! {
    #[test]
    fn prop_bytes_string_consistency(
        path in forward_slash_path(),
        content in "[ -~]{0,200}",
    ) {
        let mut fs = MemFs::new();
        fs.add_file(&path, content.clone());
        let as_string = fs.read_to_string(Path::new(&path)).unwrap();
        let as_bytes = fs.read_bytes(Path::new(&path)).unwrap();
        prop_assert_eq!(as_bytes, as_string.as_bytes().to_vec());
    }
}

proptest! {
    #[test]
    fn prop_missing_path_errors(
        existing in forward_slash_path(),
        missing in forward_slash_path(),
    ) {
        prop_assume!(existing != missing);
        let mut fs = MemFs::new();
        fs.add_file(&existing, "exists");
        prop_assert!(fs.read_to_string(Path::new(&missing)).is_err());
        prop_assert!(fs.read_bytes(Path::new(&missing)).is_err());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]
    #[test]
    fn prop_bulk_insert_all_retrievable(
        entries in pvec((forward_slash_path(), pvec(any::<u8>(), 0..128)), 1..20),
    ) {
        let mut fs = MemFs::new();
        for (path, content) in &entries {
            fs.add_bytes(path, content.clone());
        }
        let mut expected = std::collections::BTreeMap::new();
        for (path, content) in &entries {
            expected.insert(path.clone(), content.clone());
        }
        for (path, content) in &expected {
            let read_back = fs.read_bytes(Path::new(path)).unwrap();
            prop_assert_eq!(read_back, content.clone());
        }
    }
}