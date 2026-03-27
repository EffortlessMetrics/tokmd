# tokmd-envelope

Cross-fleet `SensorReport` contracts for multi-sensor integration.

## Problem
Sensors and policy checks need one shared envelope for verdicts, findings, artifacts, and gate output.

## What it gives you
- `SensorReport`, `ToolMeta`, and `Verdict`
- `Finding`, `FindingSeverity`, and `FindingLocation`
- `GateResults`, `GateItem`, `Artifact`, and capability status types

## API / usage notes
- This crate is the shared JSON contract for sensors, gate, and cockpit-style consumers.
- Keep the envelope shape stable and version it deliberately.
- `src/lib.rs` is the canonical field reference.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [tokmd-sensor](../tokmd-sensor/README.md)
- Reference: [Schema](../../docs/SCHEMA.md)
- Explanation: [Architecture](../../docs/architecture.md)
