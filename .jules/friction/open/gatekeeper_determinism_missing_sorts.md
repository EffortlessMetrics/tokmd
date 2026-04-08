# Friction Item: Missing deterministic BTreeMap sorts outside core-pipeline

## Description
The Gatekeeper persona was tasked with checking and tightening deterministic output shape across the `core-pipeline` shard. Through memory and policy references, it was established that `BTreeMap` structures serialized as arrays must define their order via an explicit `Vec::sort_by()` to prevent shape instability during serialization.

The `core-pipeline` paths (tokmd-types, scan, model, format) correctly implement explicit `.sort_by()` logic for serialization paths.

However, searching the broader codebase revealed missing deterministic explicit sort invocations on `ImportEdge` and `TodoTagRow` vectors aggregated from `BTreeMap`s within the `crates/tokmd-analysis-content` directory. Because this falls outside the allowed `core-pipeline` paths, patching it directly violates the prompt directive.

## Suggested Action
A `Prover` or `Builder` persona running within the `analysis` or `core-analysis` shard should patch `crates/tokmd-analysis-content/src/content.rs` to include the explicit `.sort_by()` rules for the output structures to align with the rest of the repository's strict determinism strategy.
