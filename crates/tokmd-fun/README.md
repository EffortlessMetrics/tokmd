# tokmd-fun

Generate novelty outputs from tokmd metrics.

## Problem

Use this crate when you want a derived artifact for demos, visualizations, or
experimentation without changing the core receipt model.

## What it gives you

- `render_obj` with `ObjBuilding`
- `render_midi` with `MidiNote`
- Wavefront OBJ output for code-city style models
- Type 1 MIDI output with configurable tempo

## Quick use / integration notes

```toml
[dependencies]
tokmd-fun = { workspace = true }
```

Feed metrics into the renderers, then write the returned OBJ or MIDI bytes to
disk.

## Go deeper

Tutorial: [Root README](../../README.md)
How-to: [Recipes](../../docs/recipes.md)
Reference: [Source](src/lib.rs)
Explanation: [Architecture](../../docs/architecture.md)
