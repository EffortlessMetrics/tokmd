use std::collections::BTreeSet;

struct Entry {
    canonical: &'static str,
    aliases: &'static [&'static str],
    summary: &'static str,
}

const ENTRIES: &[Entry] = &[
    Entry {
        canonical: "doc_density",
        aliases: &["documentation_density", "docs"],
        summary: "Ratio of comment lines to total code+comment lines.",
    },
    Entry {
        canonical: "whitespace_ratio",
        aliases: &["whitespace"],
        summary: "Ratio of blank lines to code+comment lines.",
    },
    Entry {
        canonical: "verbosity",
        aliases: &["bytes_per_line"],
        summary: "Average bytes per line; higher values often indicate denser lines.",
    },
    Entry {
        canonical: "test_density",
        aliases: &["tests"],
        summary: "Share of code lines in test files vs production files.",
    },
    Entry {
        canonical: "todo_density",
        aliases: &["todo", "fixme"],
        summary: "TODO/FIXME/HACK/XXX markers per KLOC.",
    },
    Entry {
        canonical: "polyglot_entropy",
        aliases: &["language_entropy", "polyglot"],
        summary: "Language distribution entropy; higher means code spread across more languages.",
    },
    Entry {
        canonical: "gini",
        aliases: &["distribution_gini"],
        summary: "Inequality of file sizes; higher means concentration in fewer files.",
    },
    Entry {
        canonical: "avg_cyclomatic",
        aliases: &["cyclomatic"],
        summary: "Average branching complexity across analyzed files.",
    },
    Entry {
        canonical: "max_cyclomatic",
        aliases: &[],
        summary: "Highest cyclomatic complexity found in a single file.",
    },
    Entry {
        canonical: "avg_cognitive",
        aliases: &["cognitive"],
        summary: "Average cognitive complexity (human understandability cost).",
    },
    Entry {
        canonical: "max_nesting_depth",
        aliases: &["nesting_depth"],
        summary: "Deepest observed nesting level in analyzed code.",
    },
    Entry {
        canonical: "maintainability_index",
        aliases: &["mi"],
        summary: "SEI-style maintainability score from complexity and size inputs.",
    },
    Entry {
        canonical: "technical_debt_ratio",
        aliases: &["debt_ratio", "technical_debt"],
        summary: "Complexity points per KLOC as a heuristic debt signal.",
    },
    Entry {
        canonical: "halstead",
        aliases: &["halstead_volume", "halstead_effort"],
        summary: "Halstead software-science metrics derived from operators/operands.",
    },
    Entry {
        canonical: "complexity_histogram",
        aliases: &["histogram"],
        summary: "Bucketed distribution of cyclomatic complexity values.",
    },
    Entry {
        canonical: "hotspots",
        aliases: &["git_hotspots"],
        summary: "Files with high change frequency and high size-based impact.",
    },
    Entry {
        canonical: "bus_factor",
        aliases: &["ownership"],
        summary: "Approximate author concentration by module from git history.",
    },
    Entry {
        canonical: "freshness",
        aliases: &["staleness"],
        summary: "Recency of file changes; stale files exceed threshold days.",
    },
    Entry {
        canonical: "code_age_distribution",
        aliases: &["code_age", "age_buckets"],
        summary: "Bucketed file age distribution plus recent-vs-prior refresh trend.",
    },
    Entry {
        canonical: "coupling",
        aliases: &["module_coupling"],
        summary: "Modules frequently changed together in commits.",
    },
    Entry {
        canonical: "predictive_churn",
        aliases: &["churn"],
        summary: "Trend model of module change velocity over recent commits.",
    },
    Entry {
        canonical: "duplicate_waste",
        aliases: &["dup", "duplication"],
        summary: "Redundant bytes from exact duplicate files.",
    },
    Entry {
        canonical: "duplication_density",
        aliases: &["dup_density"],
        summary: "Duplicate waste density overall and by module.",
    },
    Entry {
        canonical: "imports",
        aliases: &["import_graph"],
        summary: "Observed dependency edges across files/modules from import statements.",
    },
    Entry {
        canonical: "entropy_suspects",
        aliases: &["entropy"],
        summary: "Files with suspiciously high entropy indicating packed/binary-like content.",
    },
    Entry {
        canonical: "license_radar",
        aliases: &["license"],
        summary: "Heuristic SPDX/license detection from metadata and text.",
    },
    Entry {
        canonical: "archetype",
        aliases: &["project_archetype"],
        summary: "Repository type inference from structural signals (workspace, web app, etc.).",
    },
    Entry {
        canonical: "context_window_fit",
        aliases: &["window_fit", "context_fit"],
        summary: "Estimated token fit against a target model context window.",
    },
];

fn normalize(key: &str) -> String {
    key.trim()
        .to_ascii_lowercase()
        .replace([' ', '-', '.'], "_")
}

pub fn lookup(key: &str) -> Option<String> {
    let wanted = normalize(key);
    for entry in ENTRIES {
        if normalize(entry.canonical) == wanted {
            return Some(format!("{}: {}", entry.canonical, entry.summary));
        }
        if entry.aliases.iter().any(|a| normalize(a) == wanted) {
            return Some(format!("{}: {}", entry.canonical, entry.summary));
        }
    }
    None
}

pub fn catalog() -> String {
    let mut keys: BTreeSet<&'static str> = BTreeSet::new();
    for entry in ENTRIES {
        keys.insert(entry.canonical);
    }
    let mut out = String::from("Available metric/finding keys:\n");
    for key in keys {
        out.push_str("- ");
        out.push_str(key);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_finds_canonical_key() {
        let value = lookup("avg_cyclomatic").expect("canonical key should resolve");
        assert!(value.starts_with("avg_cyclomatic:"));
        assert!(value.contains("complexity"));
    }

    #[test]
    fn lookup_finds_alias_with_normalization() {
        let value = lookup("Distribution-Gini").expect("alias should resolve");
        assert!(value.starts_with("gini:"));
    }

    #[test]
    fn catalog_is_sorted_and_unique() {
        let catalog = catalog();
        let keys: Vec<&str> = catalog
            .lines()
            .skip(1)
            .filter_map(|line| line.strip_prefix("- "))
            .collect();

        assert!(
            !keys.is_empty(),
            "catalog should include at least one key line"
        );

        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(keys, sorted, "catalog keys should be sorted and unique");
    }
}
