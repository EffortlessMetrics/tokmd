//! Snapshot tests for tokmd-fun – wave 50.
//!
//! Covers: OBJ rendering (empty, single, multi-building, special names),
//! MIDI rendering (single note, chord, silence).

use tokmd_fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// ── 1. Empty buildings list ──────────────────────────────────────────

#[test]
fn snapshot_obj_empty() {
    let output = render_obj(&[]);
    insta::assert_snapshot!(output);
}

// ── 2. Single building ──────────────────────────────────────────────

#[test]
fn snapshot_obj_single_building() {
    let buildings = vec![ObjBuilding {
        name: "main".to_string(),
        x: 0.0,
        y: 0.0,
        w: 3.0,
        d: 3.0,
        h: 10.0,
    }];
    let output = render_obj(&buildings);
    insta::assert_snapshot!(output);
}

// ── 3. Multi-building code city ─────────────────────────────────────

#[test]
fn snapshot_obj_code_city() {
    let buildings = vec![
        ObjBuilding {
            name: "Rust".to_string(),
            x: 0.0,
            y: 0.0,
            w: 4.0,
            d: 4.0,
            h: 12.0,
        },
        ObjBuilding {
            name: "Python".to_string(),
            x: 5.0,
            y: 0.0,
            w: 3.0,
            d: 3.0,
            h: 6.0,
        },
        ObjBuilding {
            name: "TOML".to_string(),
            x: 9.0,
            y: 0.0,
            w: 1.5,
            d: 1.5,
            h: 2.0,
        },
        ObjBuilding {
            name: "Markdown".to_string(),
            x: 0.0,
            y: 5.0,
            w: 2.0,
            d: 2.0,
            h: 1.0,
        },
    ];
    let output = render_obj(&buildings);
    insta::assert_snapshot!(output);
}

// ── 4. Building with special characters in name ─────────────────────

#[test]
fn snapshot_obj_special_name() {
    let buildings = vec![ObjBuilding {
        name: "C++ (embedded)".to_string(),
        x: 0.0,
        y: 0.0,
        w: 2.0,
        d: 2.0,
        h: 3.0,
    }];
    let output = render_obj(&buildings);
    insta::assert_snapshot!(output);
}

// ── 5. Single MIDI note ─────────────────────────────────────────────

#[test]
fn snapshot_midi_single_note() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    }];
    let bytes = render_midi(&notes, 120).unwrap();
    let hex: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    insta::assert_snapshot!(hex.join(" "));
}

// ── 6. MIDI chord (simultaneous notes) ──────────────────────────────

#[test]
fn snapshot_midi_chord() {
    let notes = vec![
        MidiNote {
            key: 60,
            velocity: 80,
            start: 0,
            duration: 960,
            channel: 0,
        },
        MidiNote {
            key: 64,
            velocity: 80,
            start: 0,
            duration: 960,
            channel: 0,
        },
        MidiNote {
            key: 67,
            velocity: 80,
            start: 0,
            duration: 960,
            channel: 0,
        },
    ];
    let bytes = render_midi(&notes, 90).unwrap();
    let hex: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    insta::assert_snapshot!(hex.join(" "));
}
