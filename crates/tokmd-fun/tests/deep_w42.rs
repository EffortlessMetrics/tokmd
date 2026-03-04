//! Deep tests for tokmd-fun novelty outputs – wave 42.
//!
//! Tests eco-label (OBJ) rendering and building visualisation determinism,
//! plus MIDI output determinism and edge cases.
//!
//! Run with: `cargo test -p tokmd-fun --test deep_w42`

use tokmd_fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// =========================================================================
// Eco-label / OBJ rendering
// =========================================================================

#[test]
fn snapshot_obj_eco_label_three_languages() {
    let buildings = vec![
        ObjBuilding {
            name: "Rust".into(),
            x: 0.0,
            y: 0.0,
            w: 3.0,
            d: 2.0,
            h: 10.0,
        },
        ObjBuilding {
            name: "Python".into(),
            x: 4.0,
            y: 0.0,
            w: 2.0,
            d: 2.0,
            h: 5.0,
        },
        ObjBuilding {
            name: "Go".into(),
            x: 7.0,
            y: 0.0,
            w: 1.5,
            d: 1.5,
            h: 3.0,
        },
    ];
    let output = render_obj(&buildings);
    insta::assert_snapshot!(output);
}

#[test]
fn obj_eco_label_deterministic() {
    let buildings = vec![
        ObjBuilding {
            name: "Rust".into(),
            x: 0.0,
            y: 0.0,
            w: 2.0,
            d: 2.0,
            h: 5.0,
        },
        ObjBuilding {
            name: "TOML".into(),
            x: 3.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        },
    ];
    let a = render_obj(&buildings);
    let b = render_obj(&buildings);
    let c = render_obj(&buildings);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn obj_empty_city_deterministic() {
    let a = render_obj(&[]);
    let b = render_obj(&[]);
    assert_eq!(a, b);
    assert_eq!(a, "# tokmd code city\n");
}

#[test]
fn obj_building_name_sanitisation() {
    let buildings = vec![ObjBuilding {
        name: "src/main.rs".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 2.0,
    }];
    let output = render_obj(&buildings);
    assert!(output.contains("o src_main_rs\n"));
    assert!(!output.contains("o src/main.rs\n"));
}

#[test]
fn obj_special_chars_sanitised() {
    let buildings = vec![ObjBuilding {
        name: "C++ & C#".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    }];
    let output = render_obj(&buildings);
    assert!(output.contains("o C_____C_\n"));
}

// =========================================================================
// Building visualisation structure
// =========================================================================

#[test]
fn obj_vertex_count_per_building() {
    let buildings = vec![
        ObjBuilding {
            name: "a".into(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        },
        ObjBuilding {
            name: "b".into(),
            x: 2.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 2.0,
        },
        ObjBuilding {
            name: "c".into(),
            x: 4.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 3.0,
        },
    ];
    let output = render_obj(&buildings);
    // 8 vertices × 3 buildings = 24 "v " lines
    assert_eq!(output.matches("\nv ").count(), 24);
    // 6 faces × 3 buildings = 18 "f " lines
    assert_eq!(output.matches("\nf ").count(), 18);
}

#[test]
fn obj_face_indices_sequential() {
    let buildings = vec![
        ObjBuilding {
            name: "first".into(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        },
        ObjBuilding {
            name: "second".into(),
            x: 2.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        },
    ];
    let output = render_obj(&buildings);
    // Second building's faces should reference vertices 9-16
    assert!(output.contains("f 9 10 11 12\n"));
}

#[test]
fn snapshot_obj_tall_building() {
    let buildings = vec![ObjBuilding {
        name: "skyscraper".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 100.0,
    }];
    let output = render_obj(&buildings);
    insta::assert_snapshot!(output);
}

// =========================================================================
// MIDI novelty output
// =========================================================================

#[test]
fn midi_deterministic() {
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
    ];
    let a = render_midi(&notes, 120).unwrap();
    let b = render_midi(&notes, 120).unwrap();
    assert_eq!(a, b);
}

#[test]
fn midi_empty_deterministic() {
    let a = render_midi(&[], 120).unwrap();
    let b = render_midi(&[], 120).unwrap();
    assert_eq!(a, b);
    assert_eq!(&a[..4], b"MThd");
}

#[test]
fn midi_high_tempo_deterministic() {
    let notes = vec![MidiNote {
        key: 72,
        velocity: 127,
        start: 0,
        duration: 240,
        channel: 0,
    }];
    let a = render_midi(&notes, 300).unwrap();
    let b = render_midi(&notes, 300).unwrap();
    assert_eq!(a, b);
}
