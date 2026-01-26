#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use std::path::Path;
    use tokmd_model::{module_key, normalize_path};

    proptest! {
        #[test]
        fn test_normalize_path_never_crashes(s in "\\PC*") {
            let p = Path::new(&s);
            let _ = normalize_path(p, None);
        }

        #[test]
        fn test_normalize_path_always_forward_slash(s in "\\PC*") {
            let p = Path::new(&s);
            let normalized = normalize_path(p, None);
            prop_assert!(!normalized.contains('\\'));
        }

        #[test]
        fn test_normalize_path_strips_leading_dots(s in "\\PC*") {
            let p = Path::new(&s);
            let normalized = normalize_path(p, None);
            prop_assert!(!normalized.starts_with("./"));
        }

        #[test]
        fn test_module_key_never_crashes(
            path in "\\PC*",
            ref roots in prop::collection::vec("\\PC*", 0..5),
            depth in 0usize..10
        ) {
            let _ = module_key(&path, roots, depth);
        }

        #[test]
        fn test_module_key_root_fallback(path in "[a-zA-Z0-9_]+\\.[a-z]+") {
            // A simple filename should always be (root)
            // e.g. "Cargo.toml"
            let key = module_key(&path, &[], 2);
            prop_assert_eq!(key, "(root)");
        }
    }
}
