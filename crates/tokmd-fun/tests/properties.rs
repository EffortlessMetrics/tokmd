//! Property-based tests for tokmd-fun.
//!
//! These tests verify the correctness of OBJ and MIDI rendering functions
//! through mathematical invariants and format specifications.

use proptest::prelude::*;
use tokmd_fun::{render_midi, render_obj, MidiNote, ObjBuilding};

// ============================================================================
// Strategies
// ============================================================================

/// Strategy for generating arbitrary building names.
fn arb_name() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-zA-Z0-9_]{1,20}",          // Alphanumeric (should pass through unchanged)
        "[a-zA-Z0-9_/\\-.]{1,20}",     // With special chars (should be sanitized)
        ".*",                           // Arbitrary strings
    ]
}

/// Strategy for generating buildings with finite coordinates (no NaN/Inf).
fn arb_building_finite() -> impl Strategy<Value = ObjBuilding> {
    (
        arb_name(),
        -1e6f32..1e6f32, // x
        -1e6f32..1e6f32, // y
        0.0f32..1e6f32,  // w (width, non-negative)
        0.0f32..1e6f32,  // d (depth, non-negative)
        0.0f32..1e6f32,  // h (height, non-negative)
    )
        .prop_map(|(name, x, y, w, d, h)| ObjBuilding { name, x, y, w, d, h })
}

/// Strategy for generating arbitrary MIDI notes.
fn arb_midi_note() -> impl Strategy<Value = MidiNote> {
    (
        0u8..=127u8,    // key
        0u8..=127u8,    // velocity
        0u32..=100000,  // start
        0u32..=10000,   // duration
        0u8..=255u8,    // channel (will be clamped to 0-15)
    )
        .prop_map(|(key, velocity, start, duration, channel)| MidiNote {
            key,
            velocity,
            start,
            duration,
            channel,
        })
}

/// Strategy for tempo values (avoiding zero).
fn arb_tempo() -> impl Strategy<Value = u16> {
    1u16..=300u16
}

// ============================================================================
// sanitize_name properties (observable through render_obj)
// ============================================================================

proptest! {
    /// Sanitized names in output contain only alphanumeric and underscore characters.
    #[test]
    fn sanitize_name_output_is_alphanumeric_underscore(name in arb_name()) {
        let building = ObjBuilding {
            name: name.clone(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        };
        let result = render_obj(&[building]);

        // Extract the sanitized name from "o <name>\n"
        let obj_line = result.lines().find(|l| l.starts_with("o ")).unwrap();
        let sanitized = obj_line.strip_prefix("o ").unwrap();

        prop_assert!(
            sanitized.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
            "Sanitized name '{}' should only contain alphanumeric or underscore, original: '{}'",
            sanitized,
            name
        );
    }

    /// Sanitized name length equals input length.
    #[test]
    fn sanitize_name_preserves_length(name in "[\\x00-\\x7F]{1,50}") {
        let building = ObjBuilding {
            name: name.clone(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        };
        let result = render_obj(&[building]);

        let obj_line = result.lines().find(|l| l.starts_with("o ")).unwrap();
        let sanitized = obj_line.strip_prefix("o ").unwrap();

        prop_assert_eq!(
            sanitized.len(),
            name.len(),
            "Sanitized name length {} should equal input length {}: '{}' vs '{}'",
            sanitized.len(),
            name.len(),
            sanitized,
            name
        );
    }

    /// Alphanumeric input is unchanged after sanitization.
    #[test]
    fn sanitize_name_alphanumeric_unchanged(name in "[a-zA-Z0-9]{1,30}") {
        let building = ObjBuilding {
            name: name.clone(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        };
        let result = render_obj(&[building]);

        let obj_line = result.lines().find(|l| l.starts_with("o ")).unwrap();
        let sanitized = obj_line.strip_prefix("o ").unwrap();

        prop_assert_eq!(
            sanitized, name,
            "Alphanumeric name should be unchanged"
        );
    }

    /// Sanitization is deterministic (same input produces same output).
    #[test]
    fn sanitize_name_is_deterministic(name in arb_name()) {
        let building1 = ObjBuilding {
            name: name.clone(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        };
        let building2 = ObjBuilding {
            name: name.clone(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        };

        let result1 = render_obj(&[building1]);
        let result2 = render_obj(&[building2]);

        let obj_line1 = result1.lines().find(|l| l.starts_with("o ")).unwrap();
        let obj_line2 = result2.lines().find(|l| l.starts_with("o ")).unwrap();

        prop_assert_eq!(obj_line1, obj_line2, "Sanitization must be deterministic");
    }
}

// ============================================================================
// render_obj properties
// ============================================================================

proptest! {
    /// Output always starts with the header comment.
    #[test]
    fn render_obj_starts_with_header(buildings in prop::collection::vec(arb_building_finite(), 0..=5)) {
        let result = render_obj(&buildings);
        prop_assert!(
            result.starts_with("# tokmd code city\n"),
            "Output must start with header, got: {}",
            result.chars().take(50).collect::<String>()
        );
    }

    /// Each building produces exactly 8 vertices (v lines).
    #[test]
    fn render_obj_building_produces_8_vertices(building in arb_building_finite()) {
        let result = render_obj(&[building]);

        let vertex_count = result.lines().filter(|l| l.starts_with("v ")).count();
        prop_assert_eq!(
            vertex_count, 8,
            "Each building must produce exactly 8 vertices, got {}",
            vertex_count
        );
    }

    /// Each building produces exactly 6 faces (f lines).
    #[test]
    fn render_obj_building_produces_6_faces(building in arb_building_finite()) {
        let result = render_obj(&[building]);

        let face_count = result.lines().filter(|l| l.starts_with("f ")).count();
        prop_assert_eq!(
            face_count, 6,
            "Each building must produce exactly 6 faces, got {}",
            face_count
        );
    }

    /// Multiple buildings: vertices scale linearly (8 per building).
    #[test]
    fn render_obj_multiple_buildings_vertices(buildings in prop::collection::vec(arb_building_finite(), 1..=10)) {
        let result = render_obj(&buildings);
        let vertex_count = result.lines().filter(|l| l.starts_with("v ")).count();
        let expected = buildings.len() * 8;

        prop_assert_eq!(
            vertex_count, expected,
            "Expected {} vertices for {} buildings, got {}",
            expected,
            buildings.len(),
            vertex_count
        );
    }

    /// Multiple buildings: faces scale linearly (6 per building).
    #[test]
    fn render_obj_multiple_buildings_faces(buildings in prop::collection::vec(arb_building_finite(), 1..=10)) {
        let result = render_obj(&buildings);
        let face_count = result.lines().filter(|l| l.starts_with("f ")).count();
        let expected = buildings.len() * 6;

        prop_assert_eq!(
            face_count, expected,
            "Expected {} faces for {} buildings, got {}",
            expected,
            buildings.len(),
            face_count
        );
    }

    /// Empty buildings list produces just the header comment.
    #[test]
    fn render_obj_empty_list_just_header(_dummy in 0..1u8) {
        let result = render_obj(&[]);
        prop_assert_eq!(
            result, "# tokmd code city\n",
            "Empty list should produce only header"
        );
    }

    /// Vertex indices in faces are always valid (within range).
    #[test]
    fn render_obj_face_indices_valid(buildings in prop::collection::vec(arb_building_finite(), 1..=5)) {
        let result = render_obj(&buildings);

        let total_vertices = buildings.len() * 8;

        for line in result.lines() {
            if line.starts_with("f ") {
                let indices: Vec<usize> = line
                    .strip_prefix("f ")
                    .unwrap()
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();

                prop_assert_eq!(indices.len(), 4, "Each face should have 4 vertex indices");

                for idx in &indices {
                    prop_assert!(
                        *idx >= 1 && *idx <= total_vertices,
                        "Vertex index {} out of range [1, {}]",
                        idx,
                        total_vertices
                    );
                }
            }
        }
    }

    /// Building names are sanitized in output.
    #[test]
    fn render_obj_names_sanitized(name in "[a-z/\\\\.-]{5,20}") {
        let building = ObjBuilding {
            name: name.clone(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        };
        let result = render_obj(&[building]);

        // Should not contain raw special characters in object name
        for line in result.lines() {
            if line.starts_with("o ") {
                let obj_name = line.strip_prefix("o ").unwrap();
                prop_assert!(
                    !obj_name.contains('/') && !obj_name.contains('\\') && !obj_name.contains('-') && !obj_name.contains('.'),
                    "Object name should not contain special characters: '{}'",
                    obj_name
                );
            }
        }
    }
}

// ============================================================================
// ObjBuilding coordinate properties
// ============================================================================

proptest! {
    /// Negative coordinates are valid and produce output.
    #[test]
    fn render_obj_negative_coordinates_valid(
        x in -1000.0f32..0.0f32,
        y in -1000.0f32..0.0f32,
    ) {
        let building = ObjBuilding {
            name: "test".to_string(),
            x,
            y,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        };
        let result = render_obj(&[building]);

        // Should contain valid vertex and face definitions
        prop_assert!(result.contains("v "));
        prop_assert!(result.contains("f "));

        // Vertex count should still be 8
        let vertex_count = result.lines().filter(|l| l.starts_with("v ")).count();
        prop_assert_eq!(vertex_count, 8);
    }

    /// Zero dimensions produce valid (degenerate) geometry.
    #[test]
    fn render_obj_zero_dimensions_valid(
        w in prop_oneof![Just(0.0f32), 0.0f32..10.0f32],
        d in prop_oneof![Just(0.0f32), 0.0f32..10.0f32],
        h in prop_oneof![Just(0.0f32), 0.0f32..10.0f32],
    ) {
        let building = ObjBuilding {
            name: "test".to_string(),
            x: 0.0,
            y: 0.0,
            w,
            d,
            h,
        };
        let result = render_obj(&[building]);

        // Should still produce valid OBJ structure
        prop_assert!(result.starts_with("# tokmd code city\n"));
        let vertex_count = result.lines().filter(|l| l.starts_with("v ")).count();
        let face_count = result.lines().filter(|l| l.starts_with("f ")).count();

        prop_assert_eq!(vertex_count, 8, "Zero dimensions should still produce 8 vertices");
        prop_assert_eq!(face_count, 6, "Zero dimensions should still produce 6 faces");
    }

    /// Very large coordinates don't overflow (finite input produces valid output).
    #[test]
    fn render_obj_large_coordinates_no_overflow(
        x in 1e6f32..1e10f32,
        y in 1e6f32..1e10f32,
        w in 1.0f32..1e6f32,
        d in 1.0f32..1e6f32,
        h in 1.0f32..1e6f32,
    ) {
        let building = ObjBuilding {
            name: "test".to_string(),
            x,
            y,
            w,
            d,
            h,
        };
        let result = render_obj(&[building]);

        // Should produce valid output without panic
        prop_assert!(result.starts_with("# tokmd code city\n"));
        let vertex_count = result.lines().filter(|l| l.starts_with("v ")).count();
        prop_assert_eq!(vertex_count, 8);
    }
}

// ============================================================================
// render_midi properties
// ============================================================================

proptest! {
    /// Output is valid MIDI (doesn't panic, produces bytes starting with MThd).
    #[test]
    fn render_midi_produces_valid_header(notes in prop::collection::vec(arb_midi_note(), 0..=10), tempo in arb_tempo()) {
        let result = render_midi(&notes, tempo);
        prop_assert!(result.is_ok(), "render_midi should not return error");

        let bytes = result.unwrap();
        if !bytes.is_empty() {
            prop_assert_eq!(
                &bytes[0..4],
                b"MThd",
                "MIDI output must start with MThd header"
            );
        }
    }

    /// Tempo calculation never divides by zero (tempo_bpm.max(1)).
    #[test]
    fn render_midi_tempo_zero_safe(notes in prop::collection::vec(arb_midi_note(), 0..=5)) {
        // Explicitly test with tempo = 0
        let result = render_midi(&notes, 0);
        prop_assert!(result.is_ok(), "tempo=0 should not cause division by zero");

        let bytes = result.unwrap();
        prop_assert!(!bytes.is_empty(), "Should produce valid MIDI even with tempo=0");
    }

    /// Channel values are clamped to 0-15.
    #[test]
    fn render_midi_channel_clamped(channel in 0u8..=255u8) {
        let note = MidiNote {
            key: 60,
            velocity: 100,
            start: 0,
            duration: 480,
            channel,
        };
        let result = render_midi(&[note], 120);
        prop_assert!(result.is_ok(), "Should handle any channel value");

        let bytes = result.unwrap();
        // Parse and verify it's valid MIDI
        let smf = midly::Smf::parse(&bytes);
        prop_assert!(smf.is_ok(), "Output should be parseable MIDI");
    }

    /// Notes with same start time are both included.
    #[test]
    fn render_midi_simultaneous_notes_included(
        key1 in 0u8..=127u8,
        key2 in 0u8..=127u8,
        start in 0u32..=10000u32,
    ) {
        prop_assume!(key1 != key2); // Different notes at same time

        let notes = vec![
            MidiNote {
                key: key1,
                velocity: 100,
                start,
                duration: 480,
                channel: 0,
            },
            MidiNote {
                key: key2,
                velocity: 100,
                start, // Same start time
                duration: 480,
                channel: 0,
            },
        ];

        let result = render_midi(&notes, 120).unwrap();
        let smf = midly::Smf::parse(&result).unwrap();

        // Count note-on events
        let note_on_count = smf.tracks[0]
            .iter()
            .filter(|e| matches!(e.kind, midly::TrackEventKind::Midi { message: midly::MidiMessage::NoteOn { .. }, .. }))
            .count();

        prop_assert_eq!(
            note_on_count, 2,
            "Both simultaneous notes should be present"
        );
    }

    /// Empty notes list produces valid MIDI with just header/end.
    #[test]
    fn render_midi_empty_notes_valid(tempo in arb_tempo()) {
        let result = render_midi(&[], tempo);
        prop_assert!(result.is_ok(), "Empty notes should produce valid result");

        let bytes = result.unwrap();
        prop_assert!(!bytes.is_empty(), "Should produce non-empty output");
        prop_assert_eq!(&bytes[0..4], b"MThd", "Should start with MIDI header");

        // Should parse as valid MIDI
        let smf = midly::Smf::parse(&bytes);
        prop_assert!(smf.is_ok(), "Empty notes should produce parseable MIDI");
    }
}

// ============================================================================
// MidiNote properties
// ============================================================================

proptest! {
    /// Key values 0-127 are valid.
    #[test]
    fn render_midi_key_range_valid(key in 0u8..=127u8) {
        let note = MidiNote {
            key,
            velocity: 100,
            start: 0,
            duration: 480,
            channel: 0,
        };
        let result = render_midi(&[note], 120);
        prop_assert!(result.is_ok(), "Key {} should be valid", key);

        let bytes = result.unwrap();
        let smf = midly::Smf::parse(&bytes);
        prop_assert!(smf.is_ok(), "Output with key {} should be valid MIDI", key);
    }

    /// Velocity values 0-127 are valid.
    #[test]
    fn render_midi_velocity_range_valid(velocity in 0u8..=127u8) {
        let note = MidiNote {
            key: 60,
            velocity,
            start: 0,
            duration: 480,
            channel: 0,
        };
        let result = render_midi(&[note], 120);
        prop_assert!(result.is_ok(), "Velocity {} should be valid", velocity);

        let bytes = result.unwrap();
        let smf = midly::Smf::parse(&bytes);
        prop_assert!(smf.is_ok(), "Output with velocity {} should be valid MIDI", velocity);
    }

    /// Duration of 0 produces note on/off at same time.
    #[test]
    fn render_midi_zero_duration_valid(start in 0u32..=10000u32) {
        let note = MidiNote {
            key: 60,
            velocity: 100,
            start,
            duration: 0, // Zero duration
            channel: 0,
        };
        let result = render_midi(&[note], 120);
        prop_assert!(result.is_ok(), "Zero duration should be valid");

        let bytes = result.unwrap();
        let smf = midly::Smf::parse(&bytes).unwrap();

        // Find note on and note off times
        let mut current_time = 0u32;
        let mut note_on_time = None;
        let mut note_off_time = None;

        for event in &smf.tracks[0] {
            current_time += event.delta.as_int();
            match event.kind {
                midly::TrackEventKind::Midi {
                    message: midly::MidiMessage::NoteOn { .. },
                    ..
                } => {
                    note_on_time = Some(current_time);
                }
                midly::TrackEventKind::Midi {
                    message: midly::MidiMessage::NoteOff { .. },
                    ..
                } => {
                    note_off_time = Some(current_time);
                }
                _ => {}
            }
        }

        if let (Some(on), Some(off)) = (note_on_time, note_off_time) {
            prop_assert_eq!(on, off, "With zero duration, note on and off should be at same time");
        }
    }

    /// Notes are sorted by time in output.
    #[test]
    fn render_midi_notes_sorted_by_time(notes in prop::collection::vec(arb_midi_note(), 2..=10)) {
        let result = render_midi(&notes, 120).unwrap();
        let smf = midly::Smf::parse(&result).unwrap();

        // Verify events are in non-decreasing time order
        let mut current_time = 0u32;
        let mut times: Vec<u32> = Vec::new();

        for event in &smf.tracks[0] {
            current_time += event.delta.as_int();
            times.push(current_time);
        }

        // Times should be non-decreasing
        for i in 1..times.len() {
            prop_assert!(
                times[i] >= times[i - 1],
                "MIDI events should be sorted by time: {} < {}",
                times[i],
                times[i - 1]
            );
        }
    }

    /// MIDI output always ends with EndOfTrack.
    #[test]
    fn render_midi_ends_with_end_of_track(notes in prop::collection::vec(arb_midi_note(), 0..=5), tempo in arb_tempo()) {
        let result = render_midi(&notes, tempo).unwrap();
        let smf = midly::Smf::parse(&result).unwrap();

        let last_event = smf.tracks[0].last();
        prop_assert!(last_event.is_some(), "Track should have events");

        let last_kind = &last_event.unwrap().kind;
        prop_assert!(
            matches!(last_kind, midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack)),
            "Last event should be EndOfTrack, got {:?}",
            last_kind
        );
    }

    /// Tempo value in MIDI matches calculation: 60_000_000 / tempo_bpm.
    /// Note: MIDI tempo is stored in 24 bits (max 16,777,215), so very low BPM values
    /// will overflow. We test with BPM >= 4 where 60_000_000/4 = 15_000_000 fits.
    #[test]
    fn render_midi_tempo_calculation_correct(tempo_bpm in 4u16..=300u16) {
        let notes = vec![MidiNote {
            key: 60,
            velocity: 100,
            start: 0,
            duration: 480,
            channel: 0,
        }];

        let result = render_midi(&notes, tempo_bpm).unwrap();
        let smf = midly::Smf::parse(&result).unwrap();

        let expected_tempo = 60_000_000u32 / tempo_bpm as u32;

        // Find tempo event
        let mut found_tempo = None;
        for event in &smf.tracks[0] {
            if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(tempo)) = event.kind {
                found_tempo = Some(tempo.as_int());
                break;
            }
        }

        prop_assert!(found_tempo.is_some(), "Should have tempo event");
        prop_assert_eq!(
            found_tempo.unwrap(),
            expected_tempo,
            "Tempo should be 60_000_000 / {} = {}",
            tempo_bpm,
            expected_tempo
        );
    }
}

// ============================================================================
// Edge cases and regression tests
// ============================================================================

proptest! {
    /// Unicode names are handled (sanitized to underscores).
    #[test]
    fn render_obj_unicode_names_sanitized(name in "[\\u{00E0}-\\u{00FF}\\u{4E00}-\\u{4FFF}]{1,10}") {
        let building = ObjBuilding {
            name: name.clone(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        };
        let result = render_obj(&[building]);

        // Should have an object line
        let obj_line = result.lines().find(|l| l.starts_with("o "));
        prop_assert!(obj_line.is_some(), "Should have object definition");

        let sanitized = obj_line.unwrap().strip_prefix("o ").unwrap();
        prop_assert!(
            sanitized.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
            "Unicode chars should be replaced with underscores"
        );
    }

    /// Multiple buildings with same name produce valid output.
    #[test]
    fn render_obj_duplicate_names_valid(name in "[a-z]{3,10}") {
        let buildings = vec![
            ObjBuilding {
                name: name.clone(),
                x: 0.0,
                y: 0.0,
                w: 1.0,
                d: 1.0,
                h: 1.0,
            },
            ObjBuilding {
                name: name.clone(),
                x: 5.0,
                y: 5.0,
                w: 1.0,
                d: 1.0,
                h: 1.0,
            },
        ];
        let result = render_obj(&buildings);

        // Both objects should be present
        let obj_count = result.lines().filter(|l| l.starts_with("o ")).count();
        prop_assert_eq!(obj_count, 2, "Both buildings should have object definitions");

        // Vertex and face counts should be correct
        let vertex_count = result.lines().filter(|l| l.starts_with("v ")).count();
        let face_count = result.lines().filter(|l| l.starts_with("f ")).count();
        prop_assert_eq!(vertex_count, 16, "Should have 16 vertices for 2 buildings");
        prop_assert_eq!(face_count, 12, "Should have 12 faces for 2 buildings");
    }

    /// Very long note sequences don't cause issues.
    #[test]
    fn render_midi_many_notes_valid(notes in prop::collection::vec(arb_midi_note(), 50..=100)) {
        let result = render_midi(&notes, 120);
        prop_assert!(result.is_ok(), "Many notes should not cause errors");

        let bytes = result.unwrap();
        let smf = midly::Smf::parse(&bytes);
        prop_assert!(smf.is_ok(), "Output with many notes should be valid MIDI");

        // Should have 2 events per note (on + off) plus tempo and end
        let event_count = smf.unwrap().tracks[0].len();
        let expected_min = notes.len() * 2 + 2; // note_on + note_off for each, plus tempo + end
        prop_assert!(
            event_count >= expected_min,
            "Should have at least {} events, got {}",
            expected_min,
            event_count
        );
    }
}
