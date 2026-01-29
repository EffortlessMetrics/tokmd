# tokmd-fun

## Purpose

Fun renderers for tokmd analysis. This is a **Tier 3** novelty crate providing creative visualizations.

## Responsibility

- 3D code city visualization (OBJ format)
- Audio representation (MIDI format)
- Eco-label generation
- **NOT** for serious analysis output

## Public API

### OBJ Rendering
```rust
pub fn render_obj(buildings: &[ObjBuilding]) -> String

pub struct ObjBuilding {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,  // width
    pub d: f32,  // depth
    pub h: f32,  // height (maps to code lines)
}
```

### MIDI Rendering
```rust
pub fn render_midi(notes: &[MidiNote], tempo_bpm: u16) -> Result<Vec<u8>>

pub struct MidiNote {
    pub key: u8,        // MIDI note number (0-127)
    pub velocity: u8,   // Volume (0-127)
    pub start: u32,     // Start tick
    pub duration: u32,  // Duration in ticks
    pub channel: u8,    // MIDI channel (0-15)
}
```

## Implementation Details

### OBJ Format
- Generates cubes with configurable dimensions
- Maps code metrics to building height
- Sanitizes object names (alphanumeric + underscore)

### MIDI Format
- Ticks per quarter note: 480
- Supports 16 MIDI channels
- Generates Type 1 MIDI files
- Notes mapped from file metrics

## Dependencies

- `midly` (MIDI support)
- `anyhow`

## Testing

```bash
cargo test -p tokmd-fun
```

Minimal unit tests (mostly integration).

## Do NOT

- Add serious analysis features
- Depend on analysis computation crates
- Make this a required dependency
