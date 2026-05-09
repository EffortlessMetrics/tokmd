use tokmd_types::cockpit::Composition;

/// Compute composition metrics.
pub fn compute_composition<S: AsRef<str>>(files: &[S]) -> Composition {
    let mut code = 0;
    let mut test = 0;
    let mut docs = 0;
    let mut config = 0;

    for file in files.iter() {
        let path = file.as_ref().to_lowercase();
        if path.ends_with(".rs")
            || path.ends_with(".js")
            || path.ends_with(".ts")
            || path.ends_with(".py")
        {
            if path.contains("test") || path.contains("_spec") {
                test += 1;
            } else {
                code += 1;
            }
        } else if path.ends_with(".md") || path.contains("/docs/") {
            docs += 1;
        } else if path.ends_with(".toml")
            || path.ends_with(".json")
            || path.ends_with(".yml")
            || path.ends_with(".yaml")
        {
            config += 1;
        }
    }

    let total = (code + test + docs + config) as f64;
    let (code_pct, test_pct, docs_pct, config_pct) = if total > 0.0 {
        (
            code as f64 / total,
            test as f64 / total,
            docs as f64 / total,
            config as f64 / total,
        )
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    let test_ratio = if code > 0 {
        test as f64 / code as f64
    } else if test > 0 {
        1.0
    } else {
        0.0
    };

    Composition {
        code_pct,
        test_pct,
        docs_pct,
        config_pct,
        test_ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composition_mixed_files() {
        let files = vec![
            "src/main.rs",
            "src/lib.rs",
            "tests/test_main.rs",
            "README.md",
            "Cargo.toml",
        ];
        let comp = compute_composition(&files);
        assert!(comp.code_pct > 0.0);
        assert!(comp.test_pct > 0.0);
        assert!(comp.docs_pct > 0.0);
        assert!(comp.config_pct > 0.0);
    }

    #[test]
    fn test_composition_empty_input() {
        let files: Vec<&str> = vec![];
        let comp = compute_composition(&files);
        assert_eq!(comp.code_pct, 0.0);
        assert_eq!(comp.test_pct, 0.0);
        assert_eq!(comp.test_ratio, 0.0);
    }

    #[test]
    fn test_composition_only_code() {
        let files = vec!["src/main.rs", "src/lib.rs"];
        let comp = compute_composition(&files);
        assert_eq!(comp.code_pct, 1.0);
        assert_eq!(comp.test_pct, 0.0);
        assert_eq!(comp.test_ratio, 0.0);
    }

    #[test]
    fn test_composition_test_ratio() {
        let files = vec![
            "src/main.rs",
            "src/lib.rs",
            "tests/test_main.rs",
            "tests/test_lib.rs",
        ];
        let comp = compute_composition(&files);
        // 2 code files, 2 test files -> ratio = 1.0
        assert_eq!(comp.test_ratio, 1.0);
    }

    #[test]
    fn test_composition_only_tests() {
        let files = vec!["tests/test_main.rs", "tests/test_lib.rs"];
        let comp = compute_composition(&files);
        assert_eq!(comp.code_pct, 0.0);
        assert_eq!(comp.test_pct, 1.0);
        // No code files, but tests exist -> test_ratio = 1.0
        assert_eq!(comp.test_ratio, 1.0);
    }
}
