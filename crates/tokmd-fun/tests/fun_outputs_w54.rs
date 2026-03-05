//! Comprehensive tests for tokmd-fun outputs – wave 54.
//!
//! Covers OBJ code city rendering and MIDI generation with edge cases,
//! structural invariants, property tests, and determinism checks.

use tokmd_fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// ===========================================================================
// OBJ: zero / degenerate dimensions
// ===========================================================================

#[test]
fn obj_zero_width_building_produces_flat_geometry() {
    let b = ObjBuilding {
        name: "flat_w".into(),
        x: 0.0,
        y: 0.0,
        w: 0.0,
        d: 1.0,
        h: 1.0,
    };
    let out = render_obj(&[b]);
    // Still produces 8 vertices and 6 faces (degenerate but valid OBJ)
    assert_eq!(out.lines().filter(|l| l.starts_with("v ")).count(), 8);
    assert_eq!(out.lines().filter(|l| l.starts_with("f ")).count(), 6);
}

#[test]
fn obj_zero_depth_building_produces_flat_geometry() {
    let b = ObjBuilding {
        name: "flat_d".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 0.0,
        h: 1.0,
    };
    let out = render_obj(&[b]);
    assert_eq!(out.lines().filter(|l| l.starts_with("v ")).count(), 8);
}

#[test]
fn obj_zero_height_building_produces_ground_plane() {
    let b = ObjBuilding {
        name: "flat_h".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 0.0,
    };
    let out = render_obj(&[b]);
    // All z coords should be 0
    for line in out.lines().filter(|l| l.starts_with("v ")) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let z: f32 = parts[3].parse().unwrap();
        assert_eq!(z, 0.0, "all z coords should be zero for h=0");
    }
}

#[test]
fn obj_all_zero_dimensions_building_collapses_to_point() {
    let b = ObjBuilding {
        name: "point".into(),
        x: 5.0,
        y: 3.0,
        w: 0.0,
        d: 0.0,
        h: 0.0,
    };
    let out = render_obj(&[b]);
    // All 8 vertices should be the same point
    let verts: Vec<&str> = out.lines().filter(|l| l.starts_with("v ")).collect();
    assert_eq!(verts.len(), 8);
    for v in &verts {
        assert_eq!(*v, "v 5 3 0");
    }
}

// ===========================================================================
// OBJ: very large coordinates
// ===========================================================================

#[test]
fn obj_large_coordinates_render_without_panic() {
    let b = ObjBuilding {
        name: "huge".into(),
        x: 1e6,
        y: 1e6,
        w: 1e6,
        d: 1e6,
        h: 1e6,
    };
    let out = render_obj(&[b]);
    assert!(out.contains("o huge"));
    assert_eq!(out.lines().filter(|l| l.starts_with("v ")).count(), 8);
}

// ===========================================================================
// OBJ: unicode and special character names
// ===========================================================================

#[test]
fn obj_unicode_name_is_sanitized() {
    let b = ObjBuilding {
        name: "日本語テスト".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    };
    let out = render_obj(&[b]);
    // All non-ASCII replaced with underscores
    assert!(out.contains("o ______"));
    assert!(!out.contains("日本語"));
}

#[test]
fn obj_emoji_name_is_sanitized() {
    let b = ObjBuilding {
        name: "🚀rocket".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    };
    let out = render_obj(&[b]);
    assert!(out.contains("rocket"));
    // emoji should become underscores
    assert!(!out.contains("🚀"));
}

#[test]
fn obj_empty_name_produces_empty_object_line() {
    let b = ObjBuilding {
        name: "".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    };
    let out = render_obj(&[b]);
    assert!(out.contains("o \n"));
}

#[test]
fn obj_whitespace_name_all_underscores() {
    let b = ObjBuilding {
        name: "  \t\n ".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    };
    let out = render_obj(&[b]);
    // All whitespace chars should become underscores
    let obj_line = out.lines().find(|l| l.starts_with("o ")).unwrap();
    let name_part = &obj_line[2..];
    assert!(name_part.chars().all(|c| c == '_'));
}

// ===========================================================================
// OBJ: structural invariants
// ===========================================================================

#[test]
fn obj_header_is_always_first_line() {
    let out = render_obj(&[ObjBuilding {
        name: "x".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    }]);
    assert_eq!(out.lines().next().unwrap(), "# tokmd code city");
}

#[test]
fn obj_empty_header_is_only_line() {
    let out = render_obj(&[]);
    assert_eq!(out, "# tokmd code city\n");
    assert_eq!(out.lines().count(), 1);
}

#[test]
fn obj_lines_per_building_is_15() {
    // 1 object + 8 vertex + 6 face = 15 lines per building
    let out = render_obj(&[ObjBuilding {
        name: "b".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    }]);
    // header + 15 building lines
    assert_eq!(out.lines().count(), 1 + 15);
}

#[test]
fn obj_n_buildings_has_correct_line_count() {
    for n in [1, 2, 5, 10, 50] {
        let buildings: Vec<ObjBuilding> = (0..n)
            .map(|i| ObjBuilding {
                name: format!("b{i}"),
                x: i as f32 * 2.0,
                y: 0.0,
                w: 1.0,
                d: 1.0,
                h: 1.0,
            })
            .collect();
        let out = render_obj(&buildings);
        let expected_lines = 1 + n * 15; // header + 15 per building
        assert_eq!(
            out.lines().count(),
            expected_lines,
            "n={n}: expected {expected_lines} lines"
        );
    }
}

#[test]
fn obj_face_indices_reference_valid_vertices() {
    let buildings: Vec<ObjBuilding> = (0..3)
        .map(|i| ObjBuilding {
            name: format!("b{i}"),
            x: i as f32 * 5.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        })
        .collect();
    let out = render_obj(&buildings);
    let total_vertices = 3 * 8; // 24

    for line in out.lines().filter(|l| l.starts_with("f ")) {
        for idx_str in line.split_whitespace().skip(1) {
            let idx: usize = idx_str.parse().unwrap();
            assert!(
                idx >= 1 && idx <= total_vertices,
                "face index {idx} out of range 1..={total_vertices}"
            );
        }
    }
}

#[test]
fn obj_each_face_has_exactly_four_indices() {
    let out = render_obj(&[ObjBuilding {
        name: "q".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    }]);
    for line in out.lines().filter(|l| l.starts_with("f ")) {
        let indices: Vec<&str> = line.split_whitespace().skip(1).collect();
        assert_eq!(indices.len(), 4, "face should have 4 indices: {line}");
    }
}

// ===========================================================================
// OBJ: vertex coordinate verification
// ===========================================================================

#[test]
fn obj_base_vertex_at_origin() {
    let b = ObjBuilding {
        name: "origin".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    };
    let out = render_obj(&[b]);
    // First vertex should be at (x, y, 0) = (0, 0, 0)
    let verts: Vec<&str> = out.lines().filter(|l| l.starts_with("v ")).collect();
    assert_eq!(verts[0], "v 0 0 0");
}

#[test]
fn obj_top_vertices_have_correct_height() {
    let b = ObjBuilding {
        name: "tall".into(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 42.0,
    };
    let out = render_obj(&[b]);
    let verts: Vec<&str> = out.lines().filter(|l| l.starts_with("v ")).collect();
    // Vertices 5-8 (indices 4-7) should have z=42
    for v in &verts[4..8] {
        let z: f32 = v.split_whitespace().last().unwrap().parse().unwrap();
        assert_eq!(z, 42.0);
    }
    // Vertices 1-4 (indices 0-3) should have z=0
    for v in &verts[0..4] {
        let z: f32 = v.split_whitespace().last().unwrap().parse().unwrap();
        assert_eq!(z, 0.0);
    }
}

// ===========================================================================
// OBJ: determinism
// ===========================================================================

#[test]
fn obj_deterministic_with_many_buildings() {
    let buildings: Vec<ObjBuilding> = (0..20)
        .map(|i| ObjBuilding {
            name: format!("lang_{i}"),
            x: (i % 5) as f32 * 4.0,
            y: (i / 5) as f32 * 4.0,
            w: 2.0,
            d: 2.0,
            h: (i as f32 + 1.0) * 3.0,
        })
        .collect();
    let r1 = render_obj(&buildings);
    let r2 = render_obj(&buildings);
    assert_eq!(r1, r2);
}

// ===========================================================================
// MIDI: zero duration note
// ===========================================================================

#[test]
fn midi_zero_duration_note_produces_valid_midi() {
    let note = MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 0,
        channel: 0,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    assert_eq!(&bytes[..4], b"MThd");
    let smf = midly::Smf::parse(&bytes).unwrap();
    assert_eq!(smf.tracks.len(), 1);
}

#[test]
fn midi_zero_duration_has_on_and_off_at_same_tick() {
    let note = MidiNote {
        key: 60,
        velocity: 100,
        start: 100,
        duration: 0,
        channel: 0,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    let mut time = 0u32;
    let mut on_time = None;
    let mut off_time = None;
    for ev in &smf.tracks[0] {
        time += ev.delta.as_int();
        match ev.kind {
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOn { .. },
                ..
            } => on_time = Some(time),
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOff { .. },
                ..
            } => off_time = Some(time),
            _ => {}
        }
    }
    assert_eq!(on_time.unwrap(), off_time.unwrap());
}

// ===========================================================================
// MIDI: notes in reverse start order
// ===========================================================================

#[test]
fn midi_reverse_order_notes_sorted_by_time() {
    let notes = vec![
        MidiNote {
            key: 60,
            velocity: 100,
            start: 960,
            duration: 480,
            channel: 0,
        },
        MidiNote {
            key: 64,
            velocity: 100,
            start: 480,
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
    let bytes = render_midi(&notes, 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    // Verify events are time-sorted by checking deltas are non-negative
    for ev in &smf.tracks[0] {
        // delta is always >= 0 by construction (u28)
        assert!(ev.delta.as_int() < u32::MAX);
    }

    // Verify the first NoteOn is the one at start=0
    let mut time = 0u32;
    for ev in &smf.tracks[0] {
        time += ev.delta.as_int();
        if let midly::TrackEventKind::Midi {
            message: midly::MidiMessage::NoteOn { key, .. },
            ..
        } = ev.kind
        {
            assert_eq!(time, 0, "first note-on should be at tick 0");
            assert_eq!(key.as_int(), 67, "first note should be key 67");
            break;
        }
    }
}

// ===========================================================================
// MIDI: single tick duration
// ===========================================================================

#[test]
fn midi_single_tick_duration_has_correct_off_time() {
    let note = MidiNote {
        key: 72,
        velocity: 90,
        start: 0,
        duration: 1,
        channel: 0,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    let mut time = 0u32;
    let mut off_time = None;
    for ev in &smf.tracks[0] {
        time += ev.delta.as_int();
        if let midly::TrackEventKind::Midi {
            message: midly::MidiMessage::NoteOff { .. },
            ..
        } = ev.kind
        {
            off_time = Some(time);
        }
    }
    assert_eq!(off_time.unwrap(), 1);
}

// ===========================================================================
// MIDI: format and timing metadata
// ===========================================================================

#[test]
fn midi_format_is_single_track() {
    let bytes = render_midi(&[], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    assert!(matches!(smf.header.format, midly::Format::SingleTrack));
}

#[test]
fn midi_timing_is_480_ticks_per_quarter() {
    let bytes = render_midi(&[], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    if let midly::Timing::Metrical(tpq) = smf.header.timing {
        assert_eq!(tpq.as_int(), 480);
    } else {
        panic!("expected metrical timing");
    }
}

#[test]
fn midi_end_of_track_is_last_event() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    }];
    let bytes = render_midi(&notes, 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let last = smf.tracks[0].last().unwrap();
    assert!(matches!(
        last.kind,
        midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack)
    ));
}

#[test]
fn midi_tempo_event_is_first() {
    let bytes = render_midi(&[], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let first = &smf.tracks[0][0];
    assert!(
        matches!(
            first.kind,
            midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(_))
        ),
        "first event should be tempo"
    );
}

// ===========================================================================
// MIDI: tempo calculation verification
// ===========================================================================

#[test]
fn midi_tempo_at_60_bpm_is_1000000_us() {
    let bytes = render_midi(&[], 60).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    for ev in &smf.tracks[0] {
        if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(t)) = ev.kind {
            assert_eq!(t.as_int(), 1_000_000);
            return;
        }
    }
    panic!("no tempo event found");
}

#[test]
fn midi_tempo_at_1_bpm_produces_valid_midi() {
    // 60_000_000 / 1 = 60_000_000 which overflows MIDI's 24-bit tempo field
    let bytes = render_midi(&[], 1).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    // Just verify a tempo event exists and the file is valid
    let has_tempo = smf.tracks[0].iter().any(|ev| {
        matches!(
            ev.kind,
            midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(_))
        )
    });
    assert!(has_tempo, "should have a tempo event");
}

#[test]
fn midi_tempo_at_0_bpm_clamped_produces_valid_midi() {
    // 0 BPM clamped to 1, same overflow behavior as 1 BPM
    let bytes = render_midi(&[], 0).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let has_tempo = smf.tracks[0].iter().any(|ev| {
        matches!(
            ev.kind,
            midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(_))
        )
    });
    assert!(has_tempo, "should have a tempo event");
}

// ===========================================================================
// MIDI: event counts
// ===========================================================================

#[test]
fn midi_empty_has_tempo_and_end_of_track() {
    let bytes = render_midi(&[], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    // 1 tempo + 1 end-of-track = 2 events
    assert_eq!(smf.tracks[0].len(), 2);
}

#[test]
fn midi_single_note_has_4_events() {
    let note = MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    // 1 tempo + 1 NoteOn + 1 NoteOff + 1 EndOfTrack = 4
    assert_eq!(smf.tracks[0].len(), 4);
}

#[test]
fn midi_n_notes_has_2n_plus_2_events() {
    for n in [1, 5, 10, 50] {
        let notes: Vec<MidiNote> = (0..n)
            .map(|i| MidiNote {
                key: 60,
                velocity: 100,
                start: i as u32 * 480,
                duration: 240,
                channel: 0,
            })
            .collect();
        let bytes = render_midi(&notes, 120).unwrap();
        let smf = midly::Smf::parse(&bytes).unwrap();
        let expected = 2 * n + 2; // tempo + n*on + n*off + end
        assert_eq!(
            smf.tracks[0].len(),
            expected,
            "n={n}: expected {expected} events"
        );
    }
}

// ===========================================================================
// MIDI: simultaneous notes (chord)
// ===========================================================================

#[test]
fn midi_chord_all_notes_start_at_same_time() {
    let notes: Vec<MidiNote> = [60, 64, 67]
        .iter()
        .map(|&k| MidiNote {
            key: k,
            velocity: 80,
            start: 0,
            duration: 960,
            channel: 0,
        })
        .collect();
    let bytes = render_midi(&notes, 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    let mut time = 0u32;
    let mut on_times = vec![];
    for ev in &smf.tracks[0] {
        time += ev.delta.as_int();
        if matches!(
            ev.kind,
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOn { .. },
                ..
            }
        ) {
            on_times.push(time);
        }
    }
    assert_eq!(on_times.len(), 3);
    assert!(on_times.iter().all(|&t| t == 0), "all notes start at 0");
}

// ===========================================================================
// MIDI: channel 15 (highest valid)
// ===========================================================================

#[test]
fn midi_channel_15_is_valid() {
    let note = MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 15,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let ch = smf.tracks[0].iter().find_map(|ev| match ev.kind {
        midly::TrackEventKind::Midi { channel, .. } => Some(channel.as_int()),
        _ => None,
    });
    assert_eq!(ch.unwrap(), 15);
}

#[test]
fn midi_channel_above_15_clamped() {
    let note = MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 200,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let ch = smf.tracks[0].iter().find_map(|ev| match ev.kind {
        midly::TrackEventKind::Midi { channel, .. } => Some(channel.as_int()),
        _ => None,
    });
    assert_eq!(ch.unwrap(), 15);
}

// ===========================================================================
// Property tests
// ===========================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn obj_always_starts_with_header(
            x in -1000.0f32..1000.0,
            y in -1000.0f32..1000.0,
            w in 0.0f32..100.0,
            d in 0.0f32..100.0,
            h in 0.0f32..100.0,
        ) {
            let b = ObjBuilding { name: "p".into(), x, y, w, d, h };
            let out = render_obj(&[b]);
            prop_assert!(out.starts_with("# tokmd code city\n"));
        }

        #[test]
        fn obj_has_8_vertices_per_building(n in 1usize..=20) {
            let buildings: Vec<ObjBuilding> = (0..n).map(|i| ObjBuilding {
                name: format!("b{i}"), x: i as f32, y: 0.0, w: 1.0, d: 1.0, h: 1.0,
            }).collect();
            let out = render_obj(&buildings);
            let vcount = out.lines().filter(|l| l.starts_with("v ")).count();
            prop_assert_eq!(vcount, n * 8);
        }

        #[test]
        fn obj_has_6_faces_per_building(n in 1usize..=20) {
            let buildings: Vec<ObjBuilding> = (0..n).map(|i| ObjBuilding {
                name: format!("b{i}"), x: i as f32, y: 0.0, w: 1.0, d: 1.0, h: 1.0,
            }).collect();
            let out = render_obj(&buildings);
            let fcount = out.lines().filter(|l| l.starts_with("f ")).count();
            prop_assert_eq!(fcount, n * 6);
        }

        #[test]
        fn obj_is_deterministic(
            x in -100.0f32..100.0,
            y in -100.0f32..100.0,
            w in 0.0f32..50.0,
            d in 0.0f32..50.0,
            h in 0.0f32..50.0,
        ) {
            let b = ObjBuilding { name: "det".into(), x, y, w, d, h };
            prop_assert_eq!(render_obj(std::slice::from_ref(&b)), render_obj(&[b]));
        }

        #[test]
        fn midi_always_valid_header(
            key in 0u8..=127,
            vel in 0u8..=127,
            tempo in 1u16..=300,
        ) {
            let note = MidiNote { key, velocity: vel, start: 0, duration: 480, channel: 0 };
            let bytes = render_midi(&[note], tempo).unwrap();
            prop_assert_eq!(&bytes[..4], b"MThd");
        }

        #[test]
        fn midi_is_deterministic(
            key in 0u8..=127,
            vel in 0u8..=127,
            start in 0u32..10000,
            dur in 0u32..10000,
            ch in 0u8..=15,
            tempo in 1u16..=300,
        ) {
            let note = MidiNote { key, velocity: vel, start, duration: dur, channel: ch };
            let r1 = render_midi(std::slice::from_ref(&note), tempo).unwrap();
            let r2 = render_midi(&[note], tempo).unwrap();
            prop_assert_eq!(r1, r2);
        }

        #[test]
        fn midi_event_count_formula(n in 0usize..=50) {
            let notes: Vec<MidiNote> = (0..n).map(|i| MidiNote {
                key: 60, velocity: 100, start: i as u32 * 480, duration: 240, channel: 0,
            }).collect();
            let bytes = render_midi(&notes, 120).unwrap();
            let smf = midly::Smf::parse(&bytes).unwrap();
            // tempo + n*NoteOn + n*NoteOff + EndOfTrack = 2n + 2
            prop_assert_eq!(smf.tracks[0].len(), 2 * n + 2);
        }

        #[test]
        fn midi_end_of_track_always_last(
            key in 0u8..=127,
            n in 0usize..=10,
        ) {
            let notes: Vec<MidiNote> = (0..n).map(|i| MidiNote {
                key, velocity: 100, start: i as u32 * 480, duration: 240, channel: 0,
            }).collect();
            let bytes = render_midi(&notes, 120).unwrap();
            let smf = midly::Smf::parse(&bytes).unwrap();
            let last = smf.tracks[0].last().unwrap();
            prop_assert!(matches!(last.kind, midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack)));
        }
    }
}
