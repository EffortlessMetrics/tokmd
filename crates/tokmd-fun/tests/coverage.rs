//! Additional coverage tests for `tokmd-fun`.
//!
//! Targets boundary values, negative coordinates, multi-channel MIDI,
//! large inputs, and determinism gaps not covered by existing suites.

use tokmd_fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// ── OBJ: negative coordinates ───────────────────────────────────────────

#[test]
fn given_negative_position_when_rendered_then_vertices_reflect_negatives() {
    let building = ObjBuilding {
        name: "neg".to_string(),
        x: -5.0,
        y: -3.0,
        w: 2.0,
        d: 2.0,
        h: 4.0,
    };
    let output = render_obj(&[building]);
    assert!(output.contains("v -5 -3 0"));
    assert!(output.contains("v -3 -1 4"));
}

#[test]
fn given_negative_dimensions_when_rendered_then_still_valid_obj() {
    let building = ObjBuilding {
        name: "inv".to_string(),
        x: 0.0,
        y: 0.0,
        w: -1.0,
        d: -1.0,
        h: -1.0,
    };
    let output = render_obj(&[building]);
    let vcount = output.lines().filter(|l| l.starts_with("v ")).count();
    let fcount = output.lines().filter(|l| l.starts_with("f ")).count();
    assert_eq!(vcount, 8);
    assert_eq!(fcount, 6);
}

// ── OBJ: large number of buildings ──────────────────────────────────────

#[test]
fn given_100_buildings_when_rendered_then_correct_vertex_and_face_counts() {
    let buildings: Vec<ObjBuilding> = (0..100)
        .map(|i| ObjBuilding {
            name: format!("b{i}"),
            x: i as f32 * 2.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: (i + 1) as f32,
        })
        .collect();
    let output = render_obj(&buildings);
    let vcount = output.lines().filter(|l| l.starts_with("v ")).count();
    let fcount = output.lines().filter(|l| l.starts_with("f ")).count();
    assert_eq!(vcount, 800, "100 buildings × 8 vertices");
    assert_eq!(fcount, 600, "100 buildings × 6 faces");
}

// ── OBJ: floating point precision ───────────────────────────────────────

#[test]
fn given_fractional_coords_when_rendered_then_values_present() {
    let building = ObjBuilding {
        name: "frac".to_string(),
        x: 0.5,
        y: 0.25,
        w: 1.5,
        d: 0.75,
        h: 2.5,
    };
    let output = render_obj(&[building]);
    assert!(output.contains("v 0.5 0.25 0"));
    assert!(output.contains("v 2 1 2.5"));
}

// ── OBJ: determinism with fractional coords ─────────────────────────────

#[test]
fn given_fractional_buildings_when_rendered_twice_then_identical() {
    let buildings = vec![
        ObjBuilding {
            name: "a".to_string(),
            x: 0.1,
            y: 0.2,
            w: 0.3,
            d: 0.4,
            h: 0.5,
        },
        ObjBuilding {
            name: "b".to_string(),
            x: 10.5,
            y: 20.5,
            w: 5.5,
            d: 3.5,
            h: 7.5,
        },
    ];
    let r1 = render_obj(&buildings);
    let r2 = render_obj(&buildings);
    assert_eq!(r1, r2);
}

// ── MIDI: multi-channel notes ───────────────────────────────────────────

#[test]
fn given_notes_on_different_channels_when_rendered_then_all_channels_present() {
    let notes: Vec<MidiNote> = (0..4)
        .map(|ch| MidiNote {
            key: 60 + ch,
            velocity: 100,
            start: ch as u32 * 480,
            duration: 480,
            channel: ch,
        })
        .collect();
    let bytes = render_midi(&notes, 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    let channels: std::collections::BTreeSet<u8> = smf.tracks[0]
        .iter()
        .filter_map(|e| match e.kind {
            midly::TrackEventKind::Midi { channel, .. } => Some(channel.as_int()),
            _ => None,
        })
        .collect();
    assert!(channels.contains(&0));
    assert!(channels.contains(&1));
    assert!(channels.contains(&2));
    assert!(channels.contains(&3));
}

// ── MIDI: boundary key values ───────────────────────────────────────────

#[test]
fn given_key_zero_when_rendered_then_note_on_key_is_zero() {
    let note = MidiNote {
        key: 0,
        velocity: 64,
        start: 0,
        duration: 480,
        channel: 0,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let on = smf.tracks[0].iter().find(|e| {
        matches!(
            e.kind,
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOn { .. },
                ..
            }
        )
    });
    if let midly::TrackEventKind::Midi {
        message: midly::MidiMessage::NoteOn { key, .. },
        ..
    } = on.unwrap().kind
    {
        assert_eq!(key.as_int(), 0);
    }
}

#[test]
fn given_key_127_when_rendered_then_note_on_key_is_127() {
    let note = MidiNote {
        key: 127,
        velocity: 64,
        start: 0,
        duration: 480,
        channel: 0,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let on = smf.tracks[0].iter().find(|e| {
        matches!(
            e.kind,
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOn { .. },
                ..
            }
        )
    });
    if let midly::TrackEventKind::Midi {
        message: midly::MidiMessage::NoteOn { key, .. },
        ..
    } = on.unwrap().kind
    {
        assert_eq!(key.as_int(), 127);
    }
}

// ── MIDI: velocity boundary values ──────────────────────────────────────

#[test]
fn given_velocity_zero_when_rendered_then_valid_midi() {
    let note = MidiNote {
        key: 60,
        velocity: 0,
        start: 0,
        duration: 480,
        channel: 0,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    assert_eq!(&bytes[..4], b"MThd");
}

#[test]
fn given_velocity_127_when_rendered_then_velocity_preserved() {
    let note = MidiNote {
        key: 60,
        velocity: 127,
        start: 0,
        duration: 480,
        channel: 0,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let on = smf.tracks[0].iter().find(|e| {
        matches!(
            e.kind,
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOn { .. },
                ..
            }
        )
    });
    if let midly::TrackEventKind::Midi {
        message: midly::MidiMessage::NoteOn { vel, .. },
        ..
    } = on.unwrap().kind
    {
        assert_eq!(vel.as_int(), 127);
    }
}

// ── MIDI: max BPM ───────────────────────────────────────────────────────

#[test]
fn given_max_bpm_when_rendered_then_valid_midi() {
    let note = MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    };
    let bytes = render_midi(&[note], u16::MAX).unwrap();
    assert_eq!(&bytes[..4], b"MThd");
    let smf = midly::Smf::parse(&bytes).unwrap();
    assert_eq!(smf.tracks.len(), 1);
}

// ── MIDI: large note count ──────────────────────────────────────────────

#[test]
fn given_200_notes_when_rendered_then_event_count_is_402() {
    let notes: Vec<MidiNote> = (0..200)
        .map(|i| MidiNote {
            key: (i % 128) as u8,
            velocity: 100,
            start: i as u32 * 240,
            duration: 120,
            channel: (i % 16) as u8,
        })
        .collect();
    let bytes = render_midi(&notes, 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    // 1 tempo + 200 NoteOn + 200 NoteOff + 1 EndOfTrack = 402
    assert_eq!(smf.tracks[0].len(), 402);
}

// ── MIDI: determinism with multi-channel ────────────────────────────────

#[test]
fn given_multi_channel_notes_when_rendered_twice_then_identical() {
    let notes: Vec<MidiNote> = (0..10)
        .map(|i| MidiNote {
            key: 60 + (i % 12) as u8,
            velocity: 80,
            start: i as u32 * 480,
            duration: 240,
            channel: (i % 4) as u8,
        })
        .collect();
    let r1 = render_midi(&notes, 120).unwrap();
    let r2 = render_midi(&notes, 120).unwrap();
    assert_eq!(r1, r2);
}
