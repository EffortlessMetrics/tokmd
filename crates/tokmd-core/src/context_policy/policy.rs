//! Inclusion policy assignment for classified context files.

use tokmd_types::{FileClassification, InclusionPolicy};

/// Assign an inclusion policy based on size and file classifications.
#[must_use]
pub fn assign_policy(
    tokens: usize,
    file_cap: usize,
    classifications: &[FileClassification],
) -> (InclusionPolicy, Option<String>) {
    if tokens <= file_cap {
        return (InclusionPolicy::Full, None);
    }

    let skip_classes = [
        FileClassification::Generated,
        FileClassification::DataBlob,
        FileClassification::Vendored,
    ];

    if classifications.iter().any(|c| skip_classes.contains(c)) {
        let class_names: Vec<&str> = classifications.iter().map(classification_name).collect();
        return (
            InclusionPolicy::Skip,
            Some(format!(
                "{} file exceeds cap ({} > {} tokens)",
                class_names.join("+"),
                tokens,
                file_cap
            )),
        );
    }

    (
        InclusionPolicy::HeadTail,
        Some(format!(
            "file exceeds cap ({} > {} tokens); head+tail included",
            tokens, file_cap
        )),
    )
}

fn classification_name(classification: &FileClassification) -> &'static str {
    match classification {
        FileClassification::Generated => "generated",
        FileClassification::Fixture => "fixture",
        FileClassification::Vendored => "vendored",
        FileClassification::Lockfile => "lockfile",
        FileClassification::Minified => "minified",
        FileClassification::DataBlob => "data_blob",
        FileClassification::Sourcemap => "sourcemap",
    }
}
