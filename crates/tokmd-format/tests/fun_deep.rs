#![cfg(feature = "fun")]
//! Deep tests for tokmd-format::fun: OBJ rendering, MIDI generation, and edge cases.

use tokmd_format::fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// ── OBJ rendering ──────────────────────────────────────────────────────

#[test]
fn obj_empty_produces_header_only() {
    let out = render_obj(&[]);
    assert_eq!(out, "# tokmd code city\n");
}

#[test]
fn obj_single_building_geometry() {
    let b = ObjBuilding {
        name: "lib".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 5.0,
    };
    let out = render_obj(&[b]);
    assert!(out.starts_with("# tokmd code city\n"));
    assert!(out.contains("o lib\n"));
    // 8 vertices, 6 faces per building
    assert_eq!(out.matches("\nv ").count(), 8);
    assert_eq!(out.matches("\nf ").count(), 6);
}

#[test]
fn obj_deterministic_same_input_same_output() {
    let buildings = vec![
        ObjBuilding {
            name: "a".into(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 2.0,
        },
        ObjBuilding {
            name: "b".into(),
            x: 3.0,
            y: 0.0,
            w: 2.0,
            d: 2.0,
            h: 4.0,
        },
    ];
    let out1 = render_obj(&buildings);
    let out2 = render_obj(&buildings);
    assert_eq!(out1, out2, "OBJ output must be deterministic");
}

#[test]
fn obj_many_buildings_vertex_indexing() {
    let buildings: Vec<ObjBuilding> = (0..10)
        .map(|i| ObjBuilding {
            name: format!("mod{i}"),
            x: i as f32 * 2.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: (i + 1) as f32,
        })
        .collect();
    let out = render_obj(&buildings);
    // 10 buildings × 8 vertices = 80
    assert_eq!(out.matches("\nv ").count(), 80);
    // 10 buildings × 6 faces = 60
    assert_eq!(out.matches("\nf ").count(), 60);
    // Last face references should be within valid range (73..80)
    // The last building starts at vertex_index = 9*8+1 = 73
    assert!(out.contains("f 73 74 75 76\n"));
}

#[test]
fn obj_name_sanitization_special_chars() {
    let b = ObjBuilding {
        name: "src/lib.rs (main)".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    };
    let out = render_obj(&[b]);
    assert!(out.contains("o src_lib_rs__main_\n"));
    // Must not contain original unsanitized name as object name
    assert!(!out.contains("o src/lib.rs"));
}

#[test]
fn obj_zero_dimension_building() {
    let b = ObjBuilding {
        name: "zero".into(),
        x: 0.0,
        y: 0.0,
        w: 0.0,
        d: 0.0,
        h: 0.0,
    };
    let out = render_obj(&[b]);
    // Should produce valid OBJ even for degenerate geometry
    assert!(out.contains("o zero\n"));
    assert_eq!(out.matches("\nv ").count(), 8);
    assert_eq!(out.matches("\nf ").count(), 6);
}

#[test]
fn obj_negative_coords() {
    let b = ObjBuilding {
        name: "neg".into(),
        x: -5.0,
        y: -3.0,
        w: 2.0,
        d: 2.0,
        h: 1.0,
    };
    let out = render_obj(&[b]);
    assert!(out.contains("o neg\n"));
    assert!(out.contains("v -5 -3 0"));
}

#[test]
fn obj_unicode_name_sanitized() {
    let b = ObjBuilding {
        name: "módulo_café".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    };
    let out = render_obj(&[b]);
    // Non-ASCII chars should be replaced with underscores
    assert!(out.contains("o m_dulo_caf_\n"));
}

// ── MIDI rendering ─────────────────────────────────────────────────────

#[test]
fn midi_empty_notes_valid_header() {
    let data = render_midi(&[], 120).unwrap();
    assert!(!data.is_empty());
    assert_eq!(&data[..4], b"MThd", "MIDI must start with MThd header");
}

#[test]
fn midi_single_note_valid() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    }];
    let data = render_midi(&notes, 120).unwrap();
    assert_eq!(&data[..4], b"MThd");
    assert!(data.len() > 14, "must have track data beyond header");
}

#[test]
fn midi_deterministic_same_input_same_output() {
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
    let data1 = render_midi(&notes, 120).unwrap();
    let data2 = render_midi(&notes, 120).unwrap();
    assert_eq!(data1, data2, "MIDI output must be deterministic");
}

#[test]
fn midi_multiple_channels() {
    let notes: Vec<MidiNote> = (0..16)
        .map(|ch| MidiNote {
            key: 60 + ch,
            velocity: 100,
            start: ch as u32 * 480,
            duration: 480,
            channel: ch,
        })
        .collect();
    let data = render_midi(&notes, 120).unwrap();
    assert_eq!(&data[..4], b"MThd");
}

#[test]
fn midi_channel_clamped_to_15() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 255,
    }];
    // Must not panic; channel is clamped to 15
    let data = render_midi(&notes, 120).unwrap();
    assert!(!data.is_empty());
}

#[test]
fn midi_zero_tempo_clamped() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    }];
    let data = render_midi(&notes, 0).unwrap();
    assert!(!data.is_empty());
}

#[test]
fn midi_high_tempo() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    }];
    let data = render_midi(&notes, u16::MAX).unwrap();
    assert_eq!(&data[..4], b"MThd");
}

#[test]
fn midi_simultaneous_notes() {
    // All notes at the same start time (chord)
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
            velocity: 100,
            start: 0,
            duration: 480,
            channel: 0,
        },
        MidiNote {
            key: 67,
            velocity: 100,
            start: 0,
            duration: 480,
            channel: 0,
        },
    ];
    let data = render_midi(&notes, 120).unwrap();
    assert_eq!(&data[..4], b"MThd");
}

#[test]
fn midi_zero_duration_note() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 0,
        channel: 0,
    }];
    let data = render_midi(&notes, 120).unwrap();
    assert!(!data.is_empty());
}

#[test]
fn midi_zero_velocity_note() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 0,
        start: 0,
        duration: 480,
        channel: 0,
    }];
    let data = render_midi(&notes, 120).unwrap();
    assert!(!data.is_empty());
}

#[test]
fn midi_large_start_offset() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: u32::MAX / 2,
        duration: 480,
        channel: 0,
    }];
    let data = render_midi(&notes, 120).unwrap();
    assert_eq!(&data[..4], b"MThd");
}

#[test]
fn midi_many_notes() {
    let notes: Vec<MidiNote> = (0..100)
        .map(|i| MidiNote {
            key: (60 + (i % 12)) as u8,
            velocity: 80,
            start: i * 240,
            duration: 240,
            channel: 0,
        })
        .collect();
    let data = render_midi(&notes, 140).unwrap();
    assert_eq!(&data[..4], b"MThd");
    // More notes → more data
    let single = render_midi(&notes[..1], 140).unwrap();
    assert!(data.len() > single.len());
}

#[test]
fn midi_notes_out_of_order_still_valid() {
    // Notes given in reverse order; the function should sort them
    let notes = vec![
        MidiNote {
            key: 67,
            velocity: 80,
            start: 960,
            duration: 480,
            channel: 0,
        },
        MidiNote {
            key: 60,
            velocity: 100,
            start: 0,
            duration: 480,
            channel: 0,
        },
        MidiNote {
            key: 64,
            velocity: 90,
            start: 480,
            duration: 480,
            channel: 0,
        },
    ];
    let data = render_midi(&notes, 120).unwrap();
    assert_eq!(&data[..4], b"MThd");
}

#[test]
fn midi_different_tempos_produce_different_output() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    }];
    let data_slow = render_midi(&notes, 60).unwrap();
    let data_fast = render_midi(&notes, 240).unwrap();
    assert_ne!(
        data_slow, data_fast,
        "different tempos should produce different MIDI"
    );
}

// ── OBJ + MIDI integration ────────────────────────────────────────────

#[test]
fn obj_and_midi_both_handle_empty_input() {
    let obj = render_obj(&[]);
    let midi = render_midi(&[], 120).unwrap();
    assert!(!obj.is_empty());
    assert!(!midi.is_empty());
}
