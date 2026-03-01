//! BDD-style scenario tests for tokmd-fun novelty outputs.
//!
//! Each test follows the pattern:
//! `given_<precondition>_when_<action>_then_<expected_result>`

use tokmd_fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// ===========================================================================
// Helpers
// ===========================================================================

fn unit_building(name: &str) -> ObjBuilding {
    ObjBuilding {
        name: name.to_string(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    }
}

fn building_at(name: &str, x: f32, y: f32, w: f32, d: f32, h: f32) -> ObjBuilding {
    ObjBuilding {
        name: name.to_string(),
        x,
        y,
        w,
        d,
        h,
    }
}

fn middle_c(start: u32, duration: u32) -> MidiNote {
    MidiNote {
        key: 60,
        velocity: 100,
        start,
        duration,
        channel: 0,
    }
}

// ===========================================================================
// OBJ – empty input
// ===========================================================================

#[test]
fn given_no_buildings_when_rendered_then_output_contains_only_header() {
    let output = render_obj(&[]);
    assert_eq!(output, "# tokmd code city\n");
}

#[test]
fn given_no_buildings_when_rendered_then_no_vertex_or_face_lines() {
    let output = render_obj(&[]);
    assert!(!output.contains("v "));
    assert!(!output.contains("f "));
    assert!(!output.contains("o "));
}

// ===========================================================================
// OBJ – single building structure
// ===========================================================================

#[test]
fn given_single_building_when_rendered_then_header_present() {
    let output = render_obj(&[unit_building("room")]);
    assert!(output.starts_with("# tokmd code city\n"));
}

#[test]
fn given_single_building_when_rendered_then_object_named() {
    let output = render_obj(&[unit_building("room")]);
    assert!(
        output.contains("o room\n"),
        "Expected 'o room' line in output"
    );
}

#[test]
fn given_single_building_when_rendered_then_eight_vertices() {
    let output = render_obj(&[unit_building("b")]);
    let count = output.lines().filter(|l| l.starts_with("v ")).count();
    assert_eq!(count, 8);
}

#[test]
fn given_single_building_when_rendered_then_six_faces() {
    let output = render_obj(&[unit_building("b")]);
    let count = output.lines().filter(|l| l.starts_with("f ")).count();
    assert_eq!(count, 6);
}

#[test]
fn given_single_building_when_rendered_then_faces_reference_vertices_1_through_8() {
    let output = render_obj(&[unit_building("b")]);
    for line in output.lines().filter(|l| l.starts_with("f ")) {
        let indices: Vec<usize> = line[2..]
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        assert_eq!(indices.len(), 4, "each face must have 4 indices");
        for idx in &indices {
            assert!(
                (1..=8).contains(idx),
                "index {idx} out of range for single building"
            );
        }
    }
}

// ===========================================================================
// OBJ – vertex coordinate correctness
// ===========================================================================

#[test]
fn given_origin_unit_cube_when_rendered_then_base_vertices_at_z_zero() {
    let output = render_obj(&[building_at("c", 0.0, 0.0, 1.0, 1.0, 1.0)]);
    // First four vertices are the base (z = 0)
    assert!(output.contains("v 0 0 0"));
    assert!(output.contains("v 1 0 0"));
    assert!(output.contains("v 1 1 0"));
    assert!(output.contains("v 0 1 0"));
}

#[test]
fn given_origin_unit_cube_when_rendered_then_top_vertices_at_z_one() {
    let output = render_obj(&[building_at("c", 0.0, 0.0, 1.0, 1.0, 1.0)]);
    assert!(output.contains("v 0 0 1"));
    assert!(output.contains("v 1 0 1"));
    assert!(output.contains("v 1 1 1"));
    assert!(output.contains("v 0 1 1"));
}

#[test]
fn given_offset_building_when_rendered_then_position_reflected_in_vertices() {
    let output = render_obj(&[building_at("off", 10.0, 20.0, 5.0, 3.0, 7.0)]);
    // v1 = (x, y, 0)  = (10, 20, 0)
    assert!(output.contains("v 10 20 0"));
    // v3 = (x+w, y+d, 0) = (15, 23, 0)
    assert!(output.contains("v 15 23 0"));
    // v7 = (x+w, y+d, h) = (15, 23, 7)
    assert!(output.contains("v 15 23 7"));
}

// ===========================================================================
// OBJ – name sanitization
// ===========================================================================

#[test]
fn given_alphanumeric_name_when_rendered_then_name_unchanged() {
    let output = render_obj(&[unit_building("Hello123")]);
    assert!(output.contains("o Hello123\n"));
}

#[test]
fn given_name_with_slashes_when_rendered_then_replaced_with_underscores() {
    let output = render_obj(&[unit_building("src/lib.rs")]);
    assert!(output.contains("o src_lib_rs\n"));
}

#[test]
fn given_name_with_spaces_when_rendered_then_replaced_with_underscores() {
    let output = render_obj(&[unit_building("my file")]);
    assert!(output.contains("o my_file\n"));
}

#[test]
fn given_name_with_dots_dashes_when_rendered_then_replaced_with_underscores() {
    let output = render_obj(&[unit_building("my-lib.rs")]);
    assert!(output.contains("o my_lib_rs\n"));
}

#[test]
fn given_empty_name_when_rendered_then_object_line_has_empty_name() {
    let output = render_obj(&[unit_building("")]);
    assert!(output.contains("o \n"));
}

// ===========================================================================
// OBJ – multiple buildings
// ===========================================================================

#[test]
fn given_two_buildings_when_rendered_then_two_object_lines() {
    let output = render_obj(&[unit_building("a"), unit_building("b")]);
    let count = output.lines().filter(|l| l.starts_with("o ")).count();
    assert_eq!(count, 2);
}

#[test]
fn given_two_buildings_when_rendered_then_sixteen_vertices() {
    let output = render_obj(&[unit_building("a"), unit_building("b")]);
    let count = output.lines().filter(|l| l.starts_with("v ")).count();
    assert_eq!(count, 16);
}

#[test]
fn given_two_buildings_when_rendered_then_twelve_faces() {
    let output = render_obj(&[unit_building("a"), unit_building("b")]);
    let count = output.lines().filter(|l| l.starts_with("f ")).count();
    assert_eq!(count, 12);
}

#[test]
fn given_two_buildings_when_rendered_then_second_building_faces_start_at_9() {
    let output = render_obj(&[unit_building("a"), unit_building("b")]);
    let faces: Vec<&str> = output.lines().filter(|l| l.starts_with("f ")).collect();
    // First building: faces with indices 1-8
    assert!(faces[0].starts_with("f 1 "));
    // Second building: faces with indices 9-16
    assert!(faces[6].starts_with("f 9 "));
}

#[test]
fn given_three_buildings_when_rendered_then_third_faces_start_at_17() {
    let buildings = vec![unit_building("a"), unit_building("b"), unit_building("c")];
    let output = render_obj(&buildings);
    let faces: Vec<&str> = output.lines().filter(|l| l.starts_with("f ")).collect();
    // Third building's first face should start at vertex 17
    assert!(
        faces[12].starts_with("f 17 "),
        "expected face starting at 17, got: {}",
        faces[12]
    );
}

// ===========================================================================
// OBJ – zero dimensions (degenerate geometry)
// ===========================================================================

#[test]
fn given_zero_width_building_when_rendered_then_valid_obj_produced() {
    let output = render_obj(&[building_at("flat", 0.0, 0.0, 0.0, 1.0, 1.0)]);
    let vcount = output.lines().filter(|l| l.starts_with("v ")).count();
    let fcount = output.lines().filter(|l| l.starts_with("f ")).count();
    assert_eq!(vcount, 8);
    assert_eq!(fcount, 6);
}

#[test]
fn given_zero_height_building_when_rendered_then_base_equals_top() {
    let output = render_obj(&[building_at("flat", 0.0, 0.0, 1.0, 1.0, 0.0)]);
    // With h=0, z+h == z == 0, so top and bottom vertices coincide
    // All z coordinates should be 0
    for line in output.lines().filter(|l| l.starts_with("v ")) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let z: f32 = parts[3].parse().unwrap();
        assert_eq!(z, 0.0, "z should be 0 for zero-height building");
    }
}

// ===========================================================================
// OBJ – output line ordering
// ===========================================================================

#[test]
fn given_single_building_when_rendered_then_lines_follow_header_obj_verts_faces_order() {
    let output = render_obj(&[unit_building("b")]);
    let lines: Vec<&str> = output.lines().collect();

    assert!(lines[0].starts_with('#'), "line 0 should be header comment");
    assert!(lines[1].starts_with("o "), "line 1 should be object name");
    // Next 8 lines are vertices
    for (i, line) in lines.iter().enumerate().take(10).skip(2) {
        assert!(
            line.starts_with("v "),
            "line {i} should be vertex, got: {line}",
        );
    }
    // Next 6 lines are faces
    for (i, line) in lines.iter().enumerate().take(16).skip(10) {
        assert!(
            line.starts_with("f "),
            "line {i} should be face, got: {line}",
        );
    }
}

// ===========================================================================
// MIDI – empty input
// ===========================================================================

#[test]
fn given_no_notes_when_rendered_then_valid_midi_produced() {
    let bytes = render_midi(&[], 120).unwrap();
    assert_eq!(&bytes[0..4], b"MThd");
    let smf = midly::Smf::parse(&bytes).unwrap();
    assert_eq!(smf.tracks.len(), 1);
}

#[test]
fn given_no_notes_when_rendered_then_track_has_tempo_and_end() {
    let bytes = render_midi(&[], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    // Should have exactly 2 events: tempo + EndOfTrack
    assert_eq!(smf.tracks[0].len(), 2);
}

// ===========================================================================
// MIDI – single note
// ===========================================================================

#[test]
fn given_single_note_when_rendered_then_midi_header_correct() {
    let bytes = render_midi(&[middle_c(0, 480)], 120).unwrap();
    assert_eq!(&bytes[0..4], b"MThd");
    let smf = midly::Smf::parse(&bytes).unwrap();
    assert_eq!(smf.tracks.len(), 1);
    assert!(matches!(
        smf.header.timing,
        midly::Timing::Metrical(tpq) if tpq.as_int() == 480
    ));
}

#[test]
fn given_single_note_when_rendered_then_note_on_and_off_present() {
    let bytes = render_midi(&[middle_c(0, 480)], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    let has_on = smf.tracks[0].iter().any(|e| {
        matches!(
            e.kind,
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOn { .. },
                ..
            }
        )
    });
    let has_off = smf.tracks[0].iter().any(|e| {
        matches!(
            e.kind,
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOff { .. },
                ..
            }
        )
    });
    assert!(has_on, "should have NoteOn");
    assert!(has_off, "should have NoteOff");
}

#[test]
fn given_single_note_at_start_when_rendered_then_note_on_delta_is_zero() {
    let bytes = render_midi(&[middle_c(0, 480)], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    let note_on = smf.tracks[0].iter().find(|e| {
        matches!(
            e.kind,
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOn { .. },
                ..
            }
        )
    });
    assert_eq!(note_on.unwrap().delta.as_int(), 0);
}

// ===========================================================================
// MIDI – tempo calculation
// ===========================================================================

#[test]
fn given_120_bpm_when_rendered_then_tempo_is_500000() {
    let bytes = render_midi(&[middle_c(0, 480)], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let tempo = find_tempo(&smf);
    assert_eq!(tempo, 500_000);
}

#[test]
fn given_60_bpm_when_rendered_then_tempo_is_1000000() {
    let bytes = render_midi(&[middle_c(0, 480)], 60).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let tempo = find_tempo(&smf);
    assert_eq!(tempo, 1_000_000);
}

#[test]
fn given_1_bpm_when_rendered_then_tempo_wraps_in_24_bits() {
    // 60_000_000 / 1 = 60_000_000 which exceeds MIDI's 24-bit tempo field (max 16_777_215).
    // midly truncates to lower 24 bits: 60_000_000 & 0xFF_FFFF = 9_668_352
    let bytes = render_midi(&[middle_c(0, 480)], 1).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let tempo = find_tempo(&smf);
    assert_eq!(tempo, 60_000_000u32 & 0xFF_FFFF);
}

#[test]
fn given_zero_bpm_when_rendered_then_no_division_by_zero() {
    // tempo_bpm.max(1) prevents division by zero; result is same as 1 BPM
    let result = render_midi(&[middle_c(0, 480)], 0);
    assert!(result.is_ok());
    let bytes = result.unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let tempo = find_tempo(&smf);
    // 60_000_000 / 1 exceeds 24-bit; same truncation as 1 BPM
    assert_eq!(tempo, 60_000_000u32 & 0xFF_FFFF, "0 BPM clamps to 1 BPM");
}

fn find_tempo(smf: &midly::Smf) -> u32 {
    for event in &smf.tracks[0] {
        if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(t)) = event.kind {
            return t.as_int();
        }
    }
    panic!("no tempo event found");
}

// ===========================================================================
// MIDI – note timing
// ===========================================================================

#[test]
fn given_note_starting_at_480_with_duration_240_when_rendered_then_off_at_720() {
    let note = MidiNote {
        key: 60,
        velocity: 100,
        start: 480,
        duration: 240,
        channel: 0,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    let (on, off) = find_note_times(&bytes);
    assert_eq!(on, 480);
    assert_eq!(off, 720);
}

#[test]
fn given_zero_duration_note_when_rendered_then_on_and_off_at_same_tick() {
    let note = MidiNote {
        key: 72,
        velocity: 80,
        start: 100,
        duration: 0,
        channel: 0,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    let (on, off) = find_note_times(&bytes);
    assert_eq!(on, off, "zero-duration note: on and off must coincide");
}

fn find_note_times(midi_bytes: &[u8]) -> (u32, u32) {
    let smf = midly::Smf::parse(midi_bytes).unwrap();
    let mut on_time = 0u32;
    let mut off_time = 0u32;
    let mut clock = 0u32;
    for event in &smf.tracks[0] {
        clock += event.delta.as_int();
        match event.kind {
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOn { .. },
                ..
            } => on_time = clock,
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOff { .. },
                ..
            } => off_time = clock,
            _ => {}
        }
    }
    (on_time, off_time)
}

// ===========================================================================
// MIDI – channel clamping
// ===========================================================================

#[test]
fn given_channel_255_when_rendered_then_clamped_to_15() {
    let note = MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 255,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    for event in &smf.tracks[0] {
        if let midly::TrackEventKind::Midi { channel, .. } = event.kind {
            assert!(
                channel.as_int() <= 15,
                "channel must be ≤15, got {}",
                channel.as_int()
            );
        }
    }
}

#[test]
fn given_channel_0_when_rendered_then_channel_is_0() {
    let note = MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    for event in &smf.tracks[0] {
        if let midly::TrackEventKind::Midi { channel, .. } = event.kind {
            assert_eq!(channel.as_int(), 0);
        }
    }
}

// ===========================================================================
// MIDI – multiple notes ordering
// ===========================================================================

#[test]
fn given_notes_out_of_order_when_rendered_then_events_sorted_by_time() {
    let notes = vec![
        MidiNote {
            key: 60,
            velocity: 100,
            start: 960,
            duration: 240,
            channel: 0,
        },
        MidiNote {
            key: 64,
            velocity: 100,
            start: 0,
            duration: 240,
            channel: 0,
        },
        MidiNote {
            key: 67,
            velocity: 100,
            start: 480,
            duration: 240,
            channel: 0,
        },
    ];
    let bytes = render_midi(&notes, 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    let mut clock = 0u32;
    let mut prev = 0u32;
    for event in &smf.tracks[0] {
        clock += event.delta.as_int();
        assert!(clock >= prev, "events must be in non-decreasing time order");
        prev = clock;
    }
}

#[test]
fn given_simultaneous_notes_when_rendered_then_both_note_ons_emitted() {
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
    ];
    let bytes = render_midi(&notes, 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    let on_count = smf.tracks[0]
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                midly::TrackEventKind::Midi {
                    message: midly::MidiMessage::NoteOn { .. },
                    ..
                }
            )
        })
        .count();
    assert_eq!(on_count, 2);
}

// ===========================================================================
// MIDI – end of track
// ===========================================================================

#[test]
fn given_any_notes_when_rendered_then_track_ends_with_end_of_track() {
    let bytes = render_midi(&[middle_c(0, 480)], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let last = smf.tracks[0].last().unwrap();
    assert!(matches!(
        last.kind,
        midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack)
    ));
}

#[test]
fn given_empty_notes_when_rendered_then_track_ends_with_end_of_track() {
    let bytes = render_midi(&[], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    let last = smf.tracks[0].last().unwrap();
    assert!(matches!(
        last.kind,
        midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack)
    ));
}

// ===========================================================================
// MIDI – note-off velocity
// ===========================================================================

#[test]
fn given_any_note_when_rendered_then_note_off_velocity_is_zero() {
    let bytes = render_midi(&[middle_c(0, 480)], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    for event in &smf.tracks[0] {
        if let midly::TrackEventKind::Midi {
            message: midly::MidiMessage::NoteOff { vel, .. },
            ..
        } = event.kind
        {
            assert_eq!(vel.as_int(), 0, "note-off velocity must be 0");
        }
    }
}

// ===========================================================================
// MIDI – event count invariant
// ===========================================================================

#[test]
fn given_n_notes_when_rendered_then_event_count_is_2n_plus_2() {
    for n in 0..=5 {
        let notes: Vec<MidiNote> = (0..n)
            .map(|i| MidiNote {
                key: 60 + i as u8,
                velocity: 100,
                start: i as u32 * 480,
                duration: 240,
                channel: 0,
            })
            .collect();
        let bytes = render_midi(&notes, 120).unwrap();
        let smf = midly::Smf::parse(&bytes).unwrap();
        // 1 tempo + n NoteOn + n NoteOff + 1 EndOfTrack = 2n + 2
        assert_eq!(
            smf.tracks[0].len(),
            2 * n + 2,
            "expected 2*{n}+2 events, got {}",
            smf.tracks[0].len()
        );
    }
}

// ===========================================================================
// MIDI – key and velocity propagation
// ===========================================================================

#[test]
fn given_specific_key_when_rendered_then_note_on_carries_that_key() {
    let note = MidiNote {
        key: 72,
        velocity: 110,
        start: 0,
        duration: 480,
        channel: 0,
    };
    let bytes = render_midi(&[note], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    let on_event = smf.tracks[0].iter().find(|e| {
        matches!(
            e.kind,
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOn { .. },
                ..
            }
        )
    });
    if let midly::TrackEventKind::Midi {
        message: midly::MidiMessage::NoteOn { key, vel },
        ..
    } = on_event.unwrap().kind
    {
        assert_eq!(key.as_int(), 72);
        assert_eq!(vel.as_int(), 110);
    } else {
        panic!("expected NoteOn");
    }
}

// ===========================================================================
// MIDI – single track format
// ===========================================================================

#[test]
fn given_any_input_when_rendered_then_format_is_single_track() {
    let bytes = render_midi(&[middle_c(0, 480)], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    assert!(matches!(smf.header.format, midly::Format::SingleTrack));
}

// ===========================================================================
// Determinism
// ===========================================================================

#[test]
fn given_same_buildings_when_rendered_twice_then_output_identical() {
    let buildings = vec![
        building_at("a", 0.0, 0.0, 1.0, 2.0, 3.0),
        building_at("b", 5.0, 5.0, 2.0, 2.0, 2.0),
    ];
    let r1 = render_obj(&buildings);
    let r2 = render_obj(&buildings);
    assert_eq!(r1, r2);
}

#[test]
fn given_same_notes_when_rendered_twice_then_output_identical() {
    let notes = vec![middle_c(0, 480), middle_c(480, 480)];
    let r1 = render_midi(&notes, 120).unwrap();
    let r2 = render_midi(&notes, 120).unwrap();
    assert_eq!(r1, r2);
}
