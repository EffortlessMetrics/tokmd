#![cfg(feature = "fun")]
//! Extended golden snapshot tests for tokmd-format::fun outputs.
//!
//! Covers additional OBJ city and MIDI scenarios for rendering stability.

use tokmd_format::fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// ===========================================================================
// OBJ snapshots — varied city layouts
// ===========================================================================

#[test]
fn snapshot_obj_dense_grid() {
    let buildings: Vec<ObjBuilding> = (0..4)
        .flat_map(|row| {
            (0..4).map(move |col| ObjBuilding {
                name: format!("m{}_{}", row, col),
                x: col as f32 * 3.0,
                y: row as f32 * 3.0,
                w: 2.0,
                d: 2.0,
                h: ((row + col) as f32 + 1.0) * 2.0,
            })
        })
        .collect();
    let output = render_obj(&buildings);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_obj_tall_skyscraper() {
    let output = render_obj(&[ObjBuilding {
        name: "skyscraper".to_string(),
        x: 0.0,
        y: 0.0,
        w: 2.0,
        d: 2.0,
        h: 500.0,
    }]);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_obj_fractional_coords() {
    let output = render_obj(&[ObjBuilding {
        name: "precise".to_string(),
        x: 1.5,
        y: 2.75,
        w: 0.5,
        d: 0.25,
        h: 3.125,
    }]);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_obj_negative_origin() {
    let output = render_obj(&[ObjBuilding {
        name: "neg".to_string(),
        x: -5.0,
        y: -3.0,
        w: 1.0,
        d: 1.0,
        h: 4.0,
    }]);
    insta::assert_snapshot!(output);
}

// ===========================================================================
// MIDI snapshots — additional musical patterns
// ===========================================================================

fn midi_hex(notes: &[MidiNote], tempo: u16) -> String {
    let bytes = render_midi(notes, tempo).unwrap();
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn snapshot_midi_fast_tempo() {
    let notes = vec![MidiNote {
        key: 72,
        velocity: 110,
        start: 0,
        duration: 240,
        channel: 0,
    }];
    insta::assert_snapshot!(midi_hex(&notes, 200));
}

#[test]
fn snapshot_midi_multichannel() {
    let notes = vec![
        MidiNote {
            key: 60,
            velocity: 100,
            start: 0,
            duration: 480,
            channel: 0,
        },
        MidiNote {
            key: 48,
            velocity: 90,
            start: 0,
            duration: 960,
            channel: 1,
        },
        MidiNote {
            key: 36,
            velocity: 80,
            start: 0,
            duration: 1920,
            channel: 9,
        },
    ];
    insta::assert_snapshot!(midi_hex(&notes, 120));
}

#[test]
fn snapshot_midi_overlapping_notes() {
    let notes = vec![
        MidiNote {
            key: 60,
            velocity: 100,
            start: 0,
            duration: 960,
            channel: 0,
        },
        MidiNote {
            key: 62,
            velocity: 100,
            start: 240,
            duration: 960,
            channel: 0,
        },
    ];
    insta::assert_snapshot!(midi_hex(&notes, 120));
}
