use std::collections::{BTreeMap, BTreeSet};

use tokmd_analysis_types::{TopicClouds, TopicTerm};
use tokmd_types::{ExportData, FileKind, FileRow};

const TOP_K: usize = 8;

pub(crate) fn build_topic_clouds(export: &ExportData) -> TopicClouds {
    let parents: Vec<&FileRow> = export
        .rows
        .iter()
        .filter(|r| r.kind == FileKind::Parent)
        .collect();

    let stopwords = build_stopwords(export);
    let mut terms_by_module: BTreeMap<String, BTreeMap<String, u32>> = BTreeMap::new();
    let mut df_map: BTreeMap<String, u32> = BTreeMap::new();

    for row in parents {
        let terms = tokenize_path(&row.path, &stopwords);
        if terms.is_empty() {
            continue;
        }
        let weight = weight_for_row(row);
        let module_terms = terms_by_module.entry(row.module.clone()).or_default();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for term in terms {
            *module_terms.entry(term.clone()).or_insert(0) += weight;
            seen.insert(term);
        }
        for term in seen {
            *df_map.entry(term).or_insert(0) += 1;
        }
    }

    let module_count = terms_by_module.len() as f64;
    let mut per_module: BTreeMap<String, Vec<TopicTerm>> = BTreeMap::new();
    let mut overall_tf: BTreeMap<String, u32> = BTreeMap::new();

    for (module, tf_map) in &terms_by_module {
        let mut rows: Vec<TopicTerm> = tf_map
            .iter()
            .map(|(term, tf)| {
                let df = *df_map.get(term).unwrap_or(&0);
                let score = score_term(*tf, df, module_count);
                TopicTerm {
                    term: term.clone(),
                    score,
                    tf: *tf,
                    df,
                }
            })
            .collect();
        rows.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.term.cmp(&b.term))
        });
        rows.truncate(TOP_K);
        per_module.insert(module.clone(), rows);

        for (term, tf) in tf_map {
            *overall_tf.entry(term.clone()).or_insert(0) += *tf;
        }
    }

    let mut overall: Vec<TopicTerm> = overall_tf
        .iter()
        .map(|(term, tf)| {
            let df = *df_map.get(term).unwrap_or(&0);
            let score = score_term(*tf, df, module_count);
            TopicTerm {
                term: term.clone(),
                score,
                tf: *tf,
                df,
            }
        })
        .collect();
    overall.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.term.cmp(&b.term))
    });
    overall.truncate(TOP_K);

    TopicClouds {
        per_module,
        overall,
    }
}

fn score_term(tf: u32, df: u32, module_count: f64) -> f64 {
    let tf = tf as f64;
    let df = df as f64;
    let idf = ((module_count + 1.0) / (df + 1.0)).ln() + 1.0;
    tf * idf
}

fn weight_for_row(row: &FileRow) -> u32 {
    let weight = u32::try_from(row.tokens).unwrap_or(u32::MAX);
    weight.max(1)
}

fn tokenize_path(path: &str, stopwords: &BTreeSet<String>) -> Vec<String> {
    let mut out = Vec::new();
    for part in path.replace('\\', "/").split('/') {
        if part.is_empty() {
            continue;
        }
        for token in part.split(['_', '-', '.']).filter(|t| !t.is_empty()) {
            let term = token.to_lowercase();
            if stopwords.contains(&term) {
                continue;
            }
            out.push(term);
        }
    }
    out
}

fn build_stopwords(export: &ExportData) -> BTreeSet<String> {
    let mut stop = BTreeSet::new();
    let base = [
        "src",
        "lib",
        "mod",
        "index",
        "test",
        "tests",
        "impl",
        "main",
        "bin",
        "pkg",
        "package",
        "target",
        "build",
        "dist",
        "out",
        "gen",
        "generated",
    ];
    for word in base {
        stop.insert(word.to_string());
    }
    let extensions = [
        "rs", "js", "ts", "tsx", "jsx", "py", "go", "java", "kt", "kts", "rb", "php", "c", "cc",
        "cpp", "h", "hpp", "cs", "swift", "m", "mm", "scala", "sql", "toml", "yaml", "yml", "json",
        "md", "markdown", "txt", "lock", "cfg", "ini", "env", "nix", "zig", "dart",
    ];
    for ext in extensions {
        stop.insert(ext.to_string());
    }
    for root in &export.module_roots {
        stop.insert(root.to_lowercase());
    }
    stop
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_config::ChildIncludeMode;
    use tokmd_types::{ExportData, FileKind, FileRow};

    #[test]
    fn topic_clouds_are_deterministic() {
        let rows = vec![
            FileRow {
                path: "crates/auth/src/login.rs".to_string(),
                module: "crates/auth".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 10,
                comments: 0,
                blanks: 0,
                lines: 10,
                bytes: 100,
                tokens: 50,
            },
            FileRow {
                path: "crates/auth/src/token.rs".to_string(),
                module: "crates/auth".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 10,
                comments: 0,
                blanks: 0,
                lines: 10,
                bytes: 100,
                tokens: 50,
            },
            FileRow {
                path: "crates/payments/src/stripe_api.rs".to_string(),
                module: "crates/payments".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 10,
                comments: 0,
                blanks: 0,
                lines: 10,
                bytes: 100,
                tokens: 50,
            },
            FileRow {
                path: "crates/payments/src/refund.rs".to_string(),
                module: "crates/payments".to_string(),
                lang: "Rust".to_string(),
                kind: FileKind::Parent,
                code: 10,
                comments: 0,
                blanks: 0,
                lines: 10,
                bytes: 100,
                tokens: 50,
            },
        ];
        let export = ExportData {
            rows,
            module_roots: vec!["crates".to_string()],
            module_depth: 2,
            children: ChildIncludeMode::Separate,
        };

        let topics = build_topic_clouds(&export);
        let auth = topics.per_module.get("crates/auth").unwrap();
        let payments = topics.per_module.get("crates/payments").unwrap();

        assert!(auth.iter().any(|t| t.term == "login"));
        assert!(auth.iter().any(|t| t.term == "token"));
        assert!(payments.iter().any(|t| t.term == "stripe"));
        assert!(payments.iter().any(|t| t.term == "refund"));
    }
}
