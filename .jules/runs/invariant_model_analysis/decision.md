# Decision

## Problem
The `tokmd-analysis` crate contains several modules for static analysis of codebases, including `license` detection logic. There are some implicit invariants around how metadata licenses are parsed. For example, `parse_toml_key` makes a best effort to parse `key = "value"` or `key = 'value'`, extracting the string. We can ensure we don't regress parsing behavior for different quote types or whitespace. We also have `parse_package_json_license` that parses JSON licenses.

## Option A (Recommended)
Add property tests asserting that:
1.  `extract_quoted` never panics on arbitrary string inputs.
2.  `extract_quoted` successfully parses any string wrapped in matching single or double quotes, correctly unwrapping the content.
3.  Any valid TOML/JSON layout for metadata files always retains the exact string if properly formatted.
4.  `package.json` license strings round-trip successfully, regardless of extraneous whitespace or nested object layout (the existing test just tests top-level strings).

I'll write tests directly calling `build_license_report` with these invariants to be safe since it's the public interface for the module.

## Option B
Add property tests for `tokmd_analysis::near_dup` or `maintainability`, testing edge cases in threshold logic. We already have several property tests there (e.g. `similarity_in_unit_range`), and `license` offers clearer text-parsing invariants.

## Decision
Option A. It tests string parsing invariants around quote extraction and format structure in the license parser, an important model piece.
