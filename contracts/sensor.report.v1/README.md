# sensor.report.v1

Stable JSON contract for multi-sensor PR reports.

## Problem

Use this contract when several sensors need one envelope, one verdict order,
and one place to carry evidence without coupling the director to any one tool.

## What it gives you

- required top-level fields: `schema`, `tool`, `generated_at`, `verdict`, `summary`, `findings`
- optional `artifacts`, `capabilities`, and opaque `data`
- verdict precedence: `fail > pending > warn > pass > skip`
- findings keyed by `check_id` and `code`
- explicit capabilities so "nothing ran" stays different from "all passed"

## Quick use / integration notes

`schema.json` is the contract. `examples/pass.json` and `examples/fail.json`
show the intended shape.

`tokmd cockpit --sensor-mode --artifacts-dir ...` emits this envelope in the
current tokmd flow.

## Go deeper

Tutorial: [Root README](../../README.md)
How-to: `tokmd cockpit --sensor-mode --artifacts-dir ...`
Reference: [schema.json](schema.json)
Reference: [pass.json](examples/pass.json) and [fail.json](examples/fail.json)
Explanation: [tokmd-envelope](../../crates/tokmd-envelope/README.md)
