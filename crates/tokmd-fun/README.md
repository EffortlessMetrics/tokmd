# tokmd-fun

Fun renderers for tokmd analysis.

## Overview

This is a **Tier 3** novelty crate providing creative visualizations of code metrics. It generates 3D code city models and audio representations of codebases.

## Installation

```toml
[dependencies]
tokmd-fun = "1.3"
```

## Usage

### OBJ Code City

```rust
use tokmd_fun::{render_obj, ObjBuilding};

let buildings = vec![
    ObjBuilding {
        name: "src_main".to_string(),
        x: 0.0, y: 0.0,
        w: 1.0, d: 1.0,
        h: 100.0,  // Height = lines of code
    },
];

let obj_content = render_obj(&buildings);
std::fs::write("city.obj", obj_content)?;
```

### MIDI Sonification

```rust
use tokmd_fun::{render_midi, MidiNote};

let notes = vec![
    MidiNote {
        key: 60,       // Middle C
        velocity: 80,
        start: 0,
        duration: 480,
        channel: 0,
    },
];

let midi_bytes = render_midi(&notes, 120)?;  // 120 BPM
std::fs::write("code.midi", midi_bytes)?;
```

## Types

### ObjBuilding
```rust
pub struct ObjBuilding {
    pub name: String,  // Object name (sanitized)
    pub x: f32,        // X position
    pub y: f32,        // Y position
    pub w: f32,        // Width
    pub d: f32,        // Depth
    pub h: f32,        // Height (maps to code lines)
}
```

### MidiNote
```rust
pub struct MidiNote {
    pub key: u8,       // MIDI note (0-127)
    pub velocity: u8,  // Volume (0-127)
    pub start: u32,    // Start tick
    pub duration: u32, // Duration in ticks
    pub channel: u8,   // MIDI channel (0-15)
}
```

## OBJ Format

Generates standard Wavefront OBJ with:
- Named objects for each building
- 8 vertices per cube
- 6 faces per building
- Sanitized names (alphanumeric + underscore)

## MIDI Format

Generates Type 1 MIDI files:
- 480 ticks per quarter note
- Configurable tempo
- Supports all 16 MIDI channels

## Dependencies

- `midly` - MIDI file generation
- `anyhow` - Error handling

## License

MIT OR Apache-2.0
