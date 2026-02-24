//! Single-responsibility module-key derivation for deterministic grouping.

/// Compute a module key from an input path.
///
/// Rules:
/// - Root-level files become `"(root)"`.
/// - If the first directory segment is in `module_roots`, include up to
///   `module_depth` directory segments.
/// - Otherwise, the module key is the first directory segment.
#[must_use]
pub fn module_key(path: &str, module_roots: &[String], module_depth: usize) -> String {
    let mut p = path.replace('\\', "/");
    if let Some(stripped) = p.strip_prefix("./") {
        p = stripped.to_string();
    }
    p = p.trim_start_matches('/').to_string();

    module_key_from_normalized(&p, module_roots, module_depth)
}

/// Compute a module key from a normalized path.
///
/// Expected input format:
/// - forward slashes only
/// - no leading `./`
/// - no leading `/`
#[must_use]
pub fn module_key_from_normalized(
    path: &str,
    module_roots: &[String],
    module_depth: usize,
) -> String {
    let Some((dir_part, _file_part)) = path.rsplit_once('/') else {
        return "(root)".to_string();
    };

    let mut dirs = dir_part
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".");
    let first = match dirs.next() {
        Some(s) => s,
        None => return "(root)".to_string(),
    };

    if !module_roots.iter().any(|r| r == first) {
        return first.to_string();
    }

    let depth_needed = module_depth.max(1);
    let mut key = String::with_capacity(dir_part.len());
    key.push_str(first);

    for _ in 1..depth_needed {
        if let Some(seg) = dirs.next() {
            key.push('/');
            key.push_str(seg);
        } else {
            break;
        }
    }

    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_key_root_level_file() {
        assert_eq!(module_key("Cargo.toml", &["crates".into()], 2), "(root)");
        assert_eq!(module_key("./Cargo.toml", &["crates".into()], 2), "(root)");
    }

    #[test]
    fn module_key_respects_root_and_depth() {
        let roots = vec!["crates".into(), "packages".into()];
        assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 2), "crates/foo");
        assert_eq!(
            module_key("packages/bar/src/main.rs", &roots, 2),
            "packages/bar"
        );
        assert_eq!(module_key("crates/foo/src/lib.rs", &roots, 1), "crates");
    }

    #[test]
    fn module_key_for_non_root_is_first_directory() {
        let roots = vec!["crates".into()];
        assert_eq!(module_key("src/lib.rs", &roots, 2), "src");
        assert_eq!(module_key("tools/gen.rs", &roots, 2), "tools");
    }

    #[test]
    fn module_key_depth_overflow_does_not_include_filename() {
        let roots = vec!["crates".into()];
        assert_eq!(module_key("crates/foo.rs", &roots, 2), "crates");
        assert_eq!(
            module_key("crates/foo/src/lib.rs", &roots, 10),
            "crates/foo/src"
        );
    }

    #[test]
    fn module_key_from_normalized_handles_empty_segments() {
        let roots = vec!["crates".into()];
        assert_eq!(
            module_key_from_normalized("crates//foo/src/lib.rs", &roots, 2),
            "crates/foo"
        );
    }
}
