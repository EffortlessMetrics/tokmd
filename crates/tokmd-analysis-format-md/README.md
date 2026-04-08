# tokmd-analysis-format-md

Markdown rendering for tokmd analysis receipts.

Extracted from `tokmd-analysis-format` to enforce Single Responsibility Principle (issue #998).

## Usage

This crate is an implementation detail of `tokmd-analysis-format` and is not intended for direct consumption. Use the parent crate's `render()` function instead.

## What changed

**Before:** ~850 lines of Markdown formatting logic lived inside `tokmd-analysis-format/src/lib.rs` alongside 9 other format renderers.

**After:** Markdown rendering is isolated in this microcrate, and `tokmd-analysis-format` delegates via a thin wrapper.
