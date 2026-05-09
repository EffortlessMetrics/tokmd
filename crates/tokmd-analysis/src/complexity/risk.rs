//! Cyclomatic scoring and risk classification for complexity reports.

use tokmd_analysis_types::ComplexityRisk;

pub(super) fn estimate_cyclomatic(lang: &str, text: &str) -> usize {
    let mut complexity = 1usize;

    let keywords: &[&str] = match lang.to_lowercase().as_str() {
        "rust" => &["if ", "match ", "while ", "for ", "loop ", "?", "&&", "||"],
        "javascript" | "typescript" => {
            &["if ", "case ", "while ", "for ", "?", "&&", "||", "catch "]
        }
        "python" => &["if ", "elif ", "while ", "for ", "except ", " and ", " or "],
        "go" => &["if ", "case ", "for ", "select ", "&&", "||"],
        "c" | "c++" | "java" | "c#" | "php" => {
            &["if ", "case ", "while ", "for ", "?", "&&", "||", "catch "]
        }
        "ruby" => &[
            "if ", "elsif ", "unless ", "while ", "until ", "for ", "when ", "rescue ", " and ",
            " or ",
        ],
        _ => &[],
    };

    let lower = text.to_lowercase();
    for keyword in keywords {
        complexity += lower.matches(keyword).count();
    }

    complexity
}

pub(super) fn classify_risk_extended(
    function_count: usize,
    max_function_length: usize,
    cyclomatic: usize,
    cognitive: Option<usize>,
    max_nesting: Option<usize>,
) -> ComplexityRisk {
    let mut score = 0;

    if function_count > 50 {
        score += 2;
    } else if function_count > 20 {
        score += 1;
    }

    if max_function_length > 100 {
        score += 3;
    } else if max_function_length > 50 {
        score += 2;
    } else if max_function_length > 25 {
        score += 1;
    }

    if cyclomatic > 50 {
        score += 3;
    } else if cyclomatic > 20 {
        score += 2;
    } else if cyclomatic > 10 {
        score += 1;
    }

    if let Some(cog) = cognitive {
        if cog > 100 {
            score += 3;
        } else if cog > 50 {
            score += 2;
        } else if cog > 25 {
            score += 1;
        }
    }

    if let Some(nesting) = max_nesting {
        if nesting > 8 {
            score += 3;
        } else if nesting > 5 {
            score += 2;
        } else if nesting > 4 {
            score += 1;
        }
    }

    match score {
        0..=1 => ComplexityRisk::Low,
        2..=4 => ComplexityRisk::Moderate,
        5..=7 => ComplexityRisk::High,
        _ => ComplexityRisk::Critical,
    }
}
