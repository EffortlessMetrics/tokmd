//! Golden snapshot tests for fun outputs (W70).
//!
//! Covers OBJ rendering (3D code city) and MIDI rendering (sonification).
//! MIDI output is hex-encoded since insta snapshots are text-based.

use tokmd_fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// ===========================================================================
// OBJ – 3D code city
// ===========================================================================

#[test]
fn w70_obj_empty_city() {
    insta::assert_snapshot!("w70_obj_empty_city", render_obj(&[]));
}

#[test]
fn w70_obj_single_building() {
    let buildings = vec![ObjBuilding {
        name: "core".to_string(),
        x: 0.0,
        y: 0.0,
        w: 4.0,
        d: 4.0,
        h: 15.0,
    }];
    insta::assert_snapshot!("w70_obj_single_building", render_obj(&buildings));
}

#[test]
fn w70_obj_three_buildings_grid() {
    let buildings = vec![
        ObjBuilding {
            name: "Rust".to_string(),
            x: 0.0,
            y: 0.0,
            w: 5.0,
            d: 5.0,
            h: 20.0,
        },
        ObjBuilding {
            name: "Python".to_string(),
            x: 6.0,
            y: 0.0,
            w: 3.0,
            d: 3.0,
            h: 8.0,
        },
        ObjBuilding {
            name: "TOML".to_string(),
            x: 10.0,
            y: 0.0,
            w: 2.0,
            d: 2.0,
            h: 1.5,
        },
    ];
    insta::assert_snapshot!("w70_obj_three_buildings_grid", render_obj(&buildings));
}

#[test]
fn w70_obj_special_chars() {
    let buildings = vec![ObjBuilding {
        name: "C++ (embedded)".to_string(),
        x: 1.0,
        y: 2.0,
        w: 2.5,
        d: 2.5,
        h: 5.0,
    }];
    insta::assert_snapshot!("w70_obj_special_chars", render_obj(&buildings));
}

#[test]
fn w70_obj_zero_height() {
    let buildings = vec![ObjBuilding {
        name: "empty".to_string(),
        x: 0.0,
        y: 0.0,
        w: 3.0,
        d: 3.0,
        h: 0.0,
    }];
    insta::assert_snapshot!("w70_obj_zero_height", render_obj(&buildings));
}

#[test]
fn w70_obj_fractional_coords() {
    let buildings = vec![ObjBuilding {
        name: "precise".to_string(),
        x: 1.5,
        y: 2.5,
        w: 0.75,
        d: 0.75,
        h: 3.25,
    }];
    insta::assert_snapshot!("w70_obj_fractional_coords", render_obj(&buildings));
}

// ===========================================================================
// MIDI – sonification (hex-encoded)
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
fn w70_midi_single_note() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    }];
    insta::assert_snapshot!("w70_midi_single_note", midi_hex(&notes, 120));
}

#[test]
fn w70_midi_chord() {
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
    insta::assert_snapshot!("w70_midi_chord", midi_hex(&notes, 100));
}

#[test]
fn w70_midi_scale_fragment() {
    let notes: Vec<MidiNote> = [60, 62, 64, 65, 67]
        .iter()
        .enumerate()
        .map(|(i, &key)| MidiNote {
            key,
            velocity: 90,
            start: (i as u32) * 240,
            duration: 240,
            channel: 0,
        })
        .collect();
    insta::assert_snapshot!("w70_midi_scale_fragment", midi_hex(&notes, 140));
}

#[test]
fn w70_midi_empty_notes() {
    insta::assert_snapshot!("w70_midi_empty_notes", midi_hex(&[], 120));
}
