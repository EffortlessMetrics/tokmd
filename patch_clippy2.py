import sys
import re

def process(filepath):
    with open(filepath, "r") as f:
        content = f.read()

    # We only have three remaining warnings, so we can just do simple replacements.

    content = content.replace(
        'let _ = write!(out, "- Evidence: `{}`\\n", archetype.evidence.join("`, `"));',
        'let _ = writeln!(out, "- Evidence: `{}`", archetype.evidence.join("`, `"));'
    )

    content = content.replace(
        """            let _ = write!(
                out,
                "- Overall: `{}`\\n",
                topics
                    .overall
                    .iter()
                    .map(|t| t.term.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );""",
        """            let _ = writeln!(
                out,
                "- Overall: `{}`",
                topics
                    .overall
                    .iter()
                    .map(|t| t.term.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );"""
    )

    content = content.replace(
        """                let _ = write!(
                    out,
                    "|{}|{}|{}|{}|\\n",
                    row.category,
                    row.files,
                    row.bytes,
                    row.extensions.join(", ")
                );""",
        """                let _ = writeln!(
                    out,
                    "|{}|{}|{}|{}|",
                    row.category,
                    row.files,
                    row.bytes,
                    row.extensions.join(", ")
                );"""
    )

    with open(filepath, "w") as f:
        f.write(content)

process("crates/tokmd-analysis-format/src/lib.rs")
