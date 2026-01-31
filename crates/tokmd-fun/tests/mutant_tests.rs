//! Tests designed to catch all mutants in the tokmd-fun crate.

use tokmd_fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// ============================================================================
// render_obj tests
// ============================================================================

/// Test that render_obj returns non-empty output (kills String::new() mutant)
#[test]
fn test_render_obj_non_empty() {
    let buildings = vec![ObjBuilding {
        name: "test".to_string(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    }];
    let result = render_obj(&buildings);
    assert!(!result.is_empty());
    assert!(result.contains("# tokmd code city"));
}

/// Test that render_obj returns something other than "xyzzy" (kills "xyzzy" mutant)
#[test]
fn test_render_obj_not_xyzzy() {
    let buildings = vec![ObjBuilding {
        name: "test".to_string(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    }];
    let result = render_obj(&buildings);
    assert_ne!(result, "xyzzy");
    assert!(result.contains("v ")); // Should contain vertex definitions
    assert!(result.contains("f ")); // Should contain face definitions
}

/// Test that vertex coordinates are computed correctly using + not - or *
/// This tests line 30: x + w
#[test]
fn test_render_obj_vertex_x_plus_w() {
    let buildings = vec![ObjBuilding {
        name: "test".to_string(),
        x: 2.0,
        y: 0.0,
        w: 3.0, // x + w should be 5.0, x - w would be -1.0, x * w would be 6.0
        d: 1.0,
        h: 1.0,
    }];
    let result = render_obj(&buildings);
    // v2 = (x + w, y, z) = (5, 0, 0)
    assert!(
        result.contains("v 5 0 0"),
        "Expected 'v 5 0 0' but got:\n{}",
        result
    );
}

/// Test line 31: x + w, y + d
#[test]
fn test_render_obj_vertex_y_plus_d() {
    let buildings = vec![ObjBuilding {
        name: "test".to_string(),
        x: 0.0,
        y: 2.0,
        w: 1.0,
        d: 3.0, // y + d should be 5.0
        h: 1.0,
    }];
    let result = render_obj(&buildings);
    // v3 = (x + w, y + d, z) = (1, 5, 0)
    assert!(
        result.contains("v 1 5 0"),
        "Expected 'v 1 5 0' but got:\n{}",
        result
    );
}

/// Test line 32: x, y + d
#[test]
fn test_render_obj_vertex_4_y_plus_d() {
    let buildings = vec![ObjBuilding {
        name: "test".to_string(),
        x: 1.0,
        y: 2.0,
        w: 1.0,
        d: 4.0, // y + d should be 6.0
        h: 1.0,
    }];
    let result = render_obj(&buildings);
    // v4 = (x, y + d, z) = (1, 6, 0)
    assert!(
        result.contains("v 1 6 0"),
        "Expected 'v 1 6 0' but got:\n{}",
        result
    );
}

/// Test line 33: x, y, z + h
#[test]
fn test_render_obj_vertex_z_plus_h() {
    let buildings = vec![ObjBuilding {
        name: "test".to_string(),
        x: 1.0,
        y: 2.0,
        w: 1.0,
        d: 1.0,
        h: 5.0, // z + h should be 5.0
    }];
    let result = render_obj(&buildings);
    // v5 = (x, y, z + h) = (1, 2, 5)
    assert!(
        result.contains("v 1 2 5"),
        "Expected 'v 1 2 5' but got:\n{}",
        result
    );
}

/// Test line 34: x + w, y, z + h
#[test]
fn test_render_obj_vertex_6_coords() {
    let buildings = vec![ObjBuilding {
        name: "test".to_string(),
        x: 2.0,
        y: 3.0,
        w: 4.0, // x + w = 6
        d: 1.0,
        h: 5.0, // z + h = 5
    }];
    let result = render_obj(&buildings);
    // v6 = (x + w, y, z + h) = (6, 3, 5)
    assert!(
        result.contains("v 6 3 5"),
        "Expected 'v 6 3 5' but got:\n{}",
        result
    );
}

/// Test line 35: x + w, y + d, z + h
#[test]
fn test_render_obj_vertex_7_coords() {
    let buildings = vec![ObjBuilding {
        name: "test".to_string(),
        x: 1.0,
        y: 2.0,
        w: 3.0, // x + w = 4
        d: 4.0, // y + d = 6
        h: 5.0, // z + h = 5
    }];
    let result = render_obj(&buildings);
    // v7 = (x + w, y + d, z + h) = (4, 6, 5)
    assert!(
        result.contains("v 4 6 5"),
        "Expected 'v 4 6 5' but got:\n{}",
        result
    );
}

/// Test line 36: x, y + d, z + h
#[test]
fn test_render_obj_vertex_8_coords() {
    let buildings = vec![ObjBuilding {
        name: "test".to_string(),
        x: 1.0,
        y: 2.0,
        w: 1.0,
        d: 3.0, // y + d = 5
        h: 4.0, // z + h = 4
    }];
    let result = render_obj(&buildings);
    // v8 = (x, y + d, z + h) = (1, 5, 4)
    assert!(
        result.contains("v 1 5 4"),
        "Expected 'v 1 5 4' but got:\n{}",
        result
    );
}

/// Test that vertex_index increments by 8 for each building (catches += mutants)
#[test]
fn test_render_obj_vertex_index_increment() {
    let buildings = vec![
        ObjBuilding {
            name: "first".to_string(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        },
        ObjBuilding {
            name: "second".to_string(),
            x: 10.0,
            y: 10.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        },
    ];
    let result = render_obj(&buildings);

    // First building uses vertices 1-8
    // Face indices start at vertex_index (1) and go up by 1-4
    // First building first face should be "f 1 2 3 4"
    assert!(
        result.contains("f 1 2 3 4"),
        "First building should have face '1 2 3 4'"
    );

    // Second building uses vertices 9-16 (after vertex_index += 8)
    // If += was replaced with -= or *=, we'd get wrong indices
    // Second building first face should be "f 9 10 11 12"
    assert!(
        result.contains("f 9 10 11 12"),
        "Second building should have face '9 10 11 12'"
    );
}

// ============================================================================
// sanitize_name tests (called indirectly through render_obj)
// ============================================================================

/// Test that sanitize_name returns non-empty output
#[test]
fn test_sanitize_name_non_empty() {
    let buildings = vec![ObjBuilding {
        name: "test".to_string(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    }];
    let result = render_obj(&buildings);
    // Should contain "o test" where test is the sanitized name
    assert!(result.contains("o test"), "Expected sanitized name 'test'");
}

/// Test that sanitize_name doesn't return "xyzzy"
#[test]
fn test_sanitize_name_not_xyzzy() {
    let buildings = vec![ObjBuilding {
        name: "mybuilding".to_string(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    }];
    let result = render_obj(&buildings);
    assert!(
        result.contains("o mybuilding"),
        "Expected sanitized name 'mybuilding', not 'xyzzy'"
    );
    assert!(!result.contains("o xyzzy"), "Should not contain 'o xyzzy'");
}

/// Test that sanitize_name replaces non-alphanumeric chars with underscore
#[test]
fn test_sanitize_name_special_chars() {
    let buildings = vec![ObjBuilding {
        name: "my-building/path".to_string(),
        x: 0.0,
        y: 0.0,
        w: 1.0,
        d: 1.0,
        h: 1.0,
    }];
    let result = render_obj(&buildings);
    // Should replace - and / with _
    assert!(
        result.contains("o my_building_path"),
        "Expected 'my_building_path'"
    );
}

// ============================================================================
// render_midi tests
// ============================================================================

/// Test that render_midi returns non-empty output (kills Ok(vec![]) mutant)
#[test]
fn test_render_midi_non_empty() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    }];
    let result = render_midi(&notes, 120).unwrap();
    assert!(!result.is_empty(), "MIDI output should not be empty");
    // MIDI files start with "MThd" header
    assert_eq!(&result[0..4], b"MThd", "Should start with MIDI header");
}

/// Test that render_midi doesn't return vec![0] or vec![1]
#[test]
fn test_render_midi_not_trivial() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    }];
    let result = render_midi(&notes, 120).unwrap();
    assert_ne!(result, vec![0u8], "Should not be vec![0]");
    assert_ne!(result, vec![1u8], "Should not be vec![1]");
    // Valid MIDI is much longer than 1 byte
    assert!(result.len() > 10, "MIDI output should be substantial");
}

/// Test that tempo calculation uses division correctly (line 85)
/// tempo = 60_000_000 / tempo_bpm
/// At 120 BPM: tempo = 500000 microseconds per beat
/// At 60 BPM: tempo = 1000000 microseconds per beat
#[test]
fn test_render_midi_tempo_division() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    }];

    // With different BPMs, the MIDI output should differ
    let result_120 = render_midi(&notes, 120).unwrap();
    let result_60 = render_midi(&notes, 60).unwrap();

    // The outputs should be different because tempo metadata differs
    assert_ne!(
        result_120, result_60,
        "Different tempos should produce different MIDI data"
    );
}

/// Test that note off time uses start + duration correctly (line 101)
#[test]
fn test_render_midi_note_duration() {
    // Two notes with same start but different durations
    let notes_short = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 100,
        channel: 0,
    }];
    let notes_long = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 1000,
        channel: 0,
    }];

    let result_short = render_midi(&notes_short, 120).unwrap();
    let result_long = render_midi(&notes_long, 120).unwrap();

    // Different durations should produce different MIDI data
    assert_ne!(
        result_short, result_long,
        "Different durations should produce different MIDI data"
    );
}

/// Test MIDI with multiple notes to verify timing is correct
#[test]
fn test_render_midi_multiple_notes() {
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
            start: 480,
            duration: 480,
            channel: 0,
        },
    ];
    let result = render_midi(&notes, 120).unwrap();

    // Should be valid MIDI
    assert_eq!(&result[0..4], b"MThd");

    // Verify it parses as valid MIDI
    let smf = midly::Smf::parse(&result);
    assert!(smf.is_ok(), "Output should be valid MIDI");
}

/// Test that note off time is computed as start + duration, not start - duration or start * duration
#[test]
fn test_render_midi_note_off_timing() {
    // If + was replaced with -, note off would be at start - duration = -480 (which wraps or saturates)
    // If + was replaced with *, note off would be at start * duration = 0
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 480,    // Start at tick 480
        duration: 240, // Duration 240 ticks
        channel: 0,
    }];

    let result = render_midi(&notes, 120).unwrap();

    // Parse the MIDI and verify note-off is at the right place
    let smf = midly::Smf::parse(&result).unwrap();
    assert_eq!(smf.tracks.len(), 1);

    // Find note on and note off events
    let mut note_on_time = 0u32;
    let mut note_off_time = 0u32;
    let mut current_time = 0u32;

    for event in &smf.tracks[0] {
        current_time += event.delta.as_int();
        match event.kind {
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOn { .. },
                ..
            } => {
                note_on_time = current_time;
            }
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOff { .. },
                ..
            } => {
                note_off_time = current_time;
            }
            _ => {}
        }
    }

    // Note on should be at 480, note off should be at 480 + 240 = 720
    assert_eq!(note_on_time, 480, "Note on should be at tick 480");
    assert_eq!(
        note_off_time, 720,
        "Note off should be at tick 720 (480 + 240)"
    );
}

/// Additional test to specifically catch the tempo % and * mutants
#[test]
fn test_render_midi_tempo_value() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    }];

    // At 120 BPM, tempo = 60_000_000 / 120 = 500_000 microseconds per beat
    // With %, it would be 60_000_000 % 120 = 0
    // With *, it would be 60_000_000 * 120 = overflow or very large
    let result = render_midi(&notes, 120).unwrap();
    let smf = midly::Smf::parse(&result).unwrap();

    // Find tempo event
    for event in &smf.tracks[0] {
        if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo)) = event.kind {
            let tempo_val = tempo.as_int();
            assert_eq!(
                tempo_val, 500_000,
                "Tempo should be 500000 microseconds (120 BPM)"
            );
            return;
        }
    }
    panic!("No tempo event found in MIDI");
}
