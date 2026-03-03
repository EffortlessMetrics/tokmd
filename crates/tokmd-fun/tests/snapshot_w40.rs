use tokmd_fun::{render_midi, render_obj, MidiNote, ObjBuilding};

// ── eco-label / code-city rendering ───────────────────────────────────

#[test]
fn snapshot_obj_eco_label() {
    // A small "eco-label" city: one tiny building per language
    let buildings = vec![
        ObjBuilding {
            name: "Rust".to_string(),
            x: 0.0,
            y: 0.0,
            w: 2.0,
            d: 2.0,
            h: 5.0,
        },
        ObjBuilding {
            name: "TOML".to_string(),
            x: 3.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        },
    ];
    let output = render_obj(&buildings);
    insta::assert_snapshot!(output);
}

// ── novelty output (MIDI) ─────────────────────────────────────────────

#[test]
fn snapshot_midi_novelty() {
    let notes = vec![
        MidiNote {
            key: 60,
            velocity: 100,
            start: 0,
            duration: 480,
            channel: 0,
        },
        MidiNote {
            key: 64,
            velocity: 80,
            start: 480,
            duration: 480,
            channel: 0,
        },
        MidiNote {
            key: 67,
            velocity: 60,
            start: 960,
            duration: 480,
            channel: 1,
        },
    ];
    let bytes = render_midi(&notes, 120).unwrap();
    // Snapshot the hex representation for deterministic binary comparison
    let hex: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    insta::assert_snapshot!(hex.join(" "));
}
