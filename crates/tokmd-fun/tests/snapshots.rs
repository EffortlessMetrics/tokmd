//! Snapshot tests for tokmd-fun using `insta`.
//!
//! These capture representative outputs so any rendering change
//! is caught and reviewed via `cargo insta review`.

use tokmd_fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// ===========================================================================
// OBJ snapshots
// ===========================================================================

#[test]
fn snapshot_obj_empty() {
    let output = render_obj(&[]);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_obj_single_unit_cube() {
    let output = render_obj(&[ObjBuilding {
        name: "cube".to_string(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    }]);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_obj_offset_building() {
    let output = render_obj(&[ObjBuilding {
        name: "tower".to_string(),
        x: 10.0,
        y: 20.0,
        w: 5.0,
        d: 3.0,
        h: 50.0,
    }]);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_obj_two_buildings() {
    let output = render_obj(&[
        ObjBuilding {
            name: "lib".to_string(),
            x: 0.0,
            y: 0.0,
            w: 2.0,
            d: 2.0,
            h: 10.0,
        },
        ObjBuilding {
            name: "main".to_string(),
            x: 5.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 3.0,
        },
    ]);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_obj_special_chars_name() {
    let output = render_obj(&[ObjBuilding {
        name: "src/my-lib.rs".to_string(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    }]);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_obj_zero_dimensions() {
    let output = render_obj(&[ObjBuilding {
        name: "flat".to_string(),
        x: 0.0,
        y: 0.0,
        w: 0.0,
        d: 0.0,
        h: 0.0,
    }]);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_obj_three_buildings_city() {
    let output = render_obj(&[
        ObjBuilding {
            name: "core".to_string(),
            x: 0.0,
            y: 0.0,
            w: 3.0,
            d: 3.0,
            h: 20.0,
        },
        ObjBuilding {
            name: "utils".to_string(),
            x: 5.0,
            y: 0.0,
            w: 2.0,
            d: 2.0,
            h: 5.0,
        },
        ObjBuilding {
            name: "tests".to_string(),
            x: 0.0,
            y: 5.0,
            w: 4.0,
            d: 1.0,
            h: 2.0,
        },
    ]);
    insta::assert_snapshot!(output);
}

// ===========================================================================
// MIDI snapshots (hex-encoded for readability)
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
fn snapshot_midi_empty_120bpm() {
    insta::assert_snapshot!(midi_hex(&[], 120));
}

#[test]
fn snapshot_midi_single_middle_c() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    }];
    insta::assert_snapshot!(midi_hex(&notes, 120));
}

#[test]
fn snapshot_midi_chord() {
    let notes = vec![
        MidiNote {
            key: 60,
            velocity: 100,
            start: 0,
            duration: 960,
            channel: 0,
        },
        MidiNote {
            key: 64,
            velocity: 100,
            start: 0,
            duration: 960,
            channel: 0,
        },
        MidiNote {
            key: 67,
            velocity: 100,
            start: 0,
            duration: 960,
            channel: 0,
        },
    ];
    insta::assert_snapshot!(midi_hex(&notes, 120));
}

#[test]
fn snapshot_midi_scale_fragment() {
    let notes: Vec<MidiNote> = [60, 62, 64, 65, 67]
        .iter()
        .enumerate()
        .map(|(i, &key)| MidiNote {
            key,
            velocity: 80,
            start: i as u32 * 480,
            duration: 400,
            channel: 0,
        })
        .collect();
    insta::assert_snapshot!(midi_hex(&notes, 100));
}

#[test]
fn snapshot_midi_60bpm() {
    let notes = vec![MidiNote {
        key: 48,
        velocity: 127,
        start: 0,
        duration: 960,
        channel: 1,
    }];
    insta::assert_snapshot!(midi_hex(&notes, 60));
}
