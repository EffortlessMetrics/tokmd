# tokmd-sensor

Sensor contract and substrate builder for tokmd.

## Problem
Sensors should share one scan-and-diff substrate instead of each re-running the expensive parts.

## What it gives you
- The `EffortlessSensor` trait.
- `substrate_builder::build_substrate(...) -> Result<RepoSubstrate>`.

## API / usage notes
- Implement `EffortlessSensor` for a sensor that consumes `RepoSubstrate` and returns `SensorReport`.
- `build_substrate` runs the scan once, normalizes diff membership, and builds the shared substrate.
- Keep sensor implementations in their own crates; this crate is the contract layer.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [tokmd-envelope](../tokmd-envelope/README.md)
- Reference: [Architecture](../../docs/architecture.md)
- Explanation: [Design](../../docs/design.md)
