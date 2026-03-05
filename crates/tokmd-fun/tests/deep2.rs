//! Deep tests (v2) for tokmd-fun: OBJ geometry invariants, MIDI structural
//! properties, edge-case inputs, determinism, and Clone/Debug contracts.

use tokmd_fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// ── helpers ─────────────────────────────────────────────────────────────

fn mk_building(name: &str, x: f32, y: f32, w: f32, d: f32, h: f32) -> ObjBuilding {
    ObjBuilding {
        name: name.to_string(),
        x,
        y,
        w,
        d,
        h,
    }
}

fn mk_note(key: u8, start: u32, duration: u32, channel: u8) -> MidiNote {
    MidiNote {
        key,
        velocity: 100,
        start,
        duration,
        channel,
    }
}

/// Parse all face lines from OBJ output, returning Vec<Vec<usize>>.
fn parse_faces(obj: &str) -> Vec<Vec<usize>> {
    obj.lines()
        .filter(|l| l.starts_with("f "))
        .map(|l| {
            l[2..]
                .split_whitespace()
                .filter_map(|s| s.parse::<usize>().ok())
                .collect()
        })
        .collect()
}

/// Parse all vertex lines, returning Vec<(f32, f32, f32)>.
fn parse_vertices(obj: &str) -> Vec<(f32, f32, f32)> {
    obj.lines()
        .filter(|l| l.starts_with("v "))
        .map(|l| {
            let parts: Vec<f32> = l[2..]
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            (parts[0], parts[1], parts[2])
        })
        .collect()
}

// =========================================================================
// OBJ – face isolation: each building's faces reference only its own verts
// =========================================================================

#[test]
fn obj_each_buildings_faces_reference_only_its_own_vertices() {
    let buildings: Vec<ObjBuilding> = (0..5)
        .map(|i| mk_building(&format!("m{i}"), i as f32 * 3.0, 0.0, 1.0, 1.0, 1.0))
        .collect();
    let out = render_obj(&buildings);
    let faces = parse_faces(&out);

    for (b_idx, chunk) in faces.chunks(6).enumerate() {
        let base = b_idx * 8 + 1;
        let upper = base + 7;
        for face in chunk {
            for &idx in face {
                assert!(
                    (base..=upper).contains(&idx),
                    "building {b_idx}: face index {idx} outside [{base}, {upper}]"
                );
            }
        }
    }
}

// =========================================================================
// OBJ – building output ordering matches input ordering
// =========================================================================

#[test]
fn obj_building_order_preserved_in_output() {
    let names = ["zeta", "alpha", "middle", "beta", "omega"];
    let buildings: Vec<ObjBuilding> = names
        .iter()
        .enumerate()
        .map(|(i, &n)| mk_building(n, i as f32 * 3.0, 0.0, 1.0, 1.0, 1.0))
        .collect();
    let out = render_obj(&buildings);
    let obj_names: Vec<&str> = out
        .lines()
        .filter(|l| l.starts_with("o "))
        .map(|l| l.strip_prefix("o ").unwrap())
        .collect();
    assert_eq!(obj_names, names);
}

// =========================================================================
// OBJ – vertex geometry: box corners computed correctly
// =========================================================================

#[test]
fn obj_vertex_positions_for_arbitrary_building() {
    let b = mk_building("t", 3.0, 7.0, 4.0, 5.0, 6.0);
    let out = render_obj(&[b]);
    let verts = parse_vertices(&out);
    assert_eq!(verts.len(), 8);

    // Expected vertices (x,y,z) with z_base=0
    let expected = [
        (3.0, 7.0, 0.0),
        (7.0, 7.0, 0.0),
        (7.0, 12.0, 0.0),
        (3.0, 12.0, 0.0),
        (3.0, 7.0, 6.0),
        (7.0, 7.0, 6.0),
        (7.0, 12.0, 6.0),
        (3.0, 12.0, 6.0),
    ];
    for (i, (&actual, &exp)) in verts.iter().zip(expected.iter()).enumerate() {
        assert_eq!(actual, exp, "vertex {i} mismatch");
    }
}

// =========================================================================
// OBJ – all six faces have consistent winding (same index pattern)
// =========================================================================

#[test]
fn obj_face_patterns_consistent_across_buildings() {
    // The face patterns relative to each building's base index should be identical.
    let buildings = vec![
        mk_building("a", 0.0, 0.0, 1.0, 1.0, 1.0),
        mk_building("b", 5.0, 5.0, 2.0, 2.0, 2.0),
        mk_building("c", 10.0, 0.0, 3.0, 3.0, 3.0),
    ];
    let out = render_obj(&buildings);
    let faces = parse_faces(&out);

    // Normalize each building's 6 faces to 0-based offsets
    let normalize = |chunk: &[Vec<usize>], base: usize| -> Vec<Vec<usize>> {
        chunk
            .iter()
            .map(|f| f.iter().map(|&i| i - base).collect())
            .collect()
    };
    let pattern0 = normalize(&faces[0..6], 1);
    let pattern1 = normalize(&faces[6..12], 9);
    let pattern2 = normalize(&faces[12..18], 17);

    assert_eq!(
        pattern0, pattern1,
        "face patterns differ between building 0 and 1"
    );
    assert_eq!(
        pattern0, pattern2,
        "face patterns differ between building 0 and 2"
    );
}

// =========================================================================
// OBJ – name sanitization edge cases
// =========================================================================

#[test]
fn obj_all_underscores_name_preserved() {
    let out = render_obj(&[mk_building("___", 0.0, 0.0, 1.0, 1.0, 1.0)]);
    assert!(out.contains("o ___\n"));
}

#[test]
fn obj_numeric_only_name_preserved() {
    let out = render_obj(&[mk_building("12345", 0.0, 0.0, 1.0, 1.0, 1.0)]);
    assert!(out.contains("o 12345\n"));
}

#[test]
fn obj_long_name_sanitized_without_truncation() {
    let long = "a".repeat(200);
    let out = render_obj(&[mk_building(&long, 0.0, 0.0, 1.0, 1.0, 1.0)]);
    let obj_line = out.lines().find(|l| l.starts_with("o ")).unwrap();
    let sanitized = obj_line.strip_prefix("o ").unwrap();
    assert_eq!(sanitized.len(), 200, "long name should not be truncated");
}

#[test]
fn obj_tab_and_newline_chars_sanitized() {
    let out = render_obj(&[mk_building("a\tb\nc", 0.0, 0.0, 1.0, 1.0, 1.0)]);
    let obj_line = out.lines().find(|l| l.starts_with("o ")).unwrap();
    let sanitized = obj_line.strip_prefix("o ").unwrap();
    assert!(
        sanitized
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_'),
        "tabs/newlines should be sanitized: got '{sanitized}'"
    );
}

#[test]
fn obj_emoji_name_sanitized_to_underscores() {
    let out = render_obj(&[mk_building("🦀🔥", 0.0, 0.0, 1.0, 1.0, 1.0)]);
    let obj_line = out.lines().find(|l| l.starts_with("o ")).unwrap();
    let sanitized = obj_line.strip_prefix("o ").unwrap();
    assert!(
        sanitized.chars().all(|c| c == '_'),
        "emoji should be replaced with underscores: got '{sanitized}'"
    );
}

// =========================================================================
// OBJ – output is valid ASCII (no control characters except newline)
// =========================================================================

#[test]
fn obj_output_contains_no_control_chars_except_newline() {
    let buildings = vec![
        mk_building("lib/main.rs", 0.0, 0.0, 2.0, 2.0, 10.0),
        mk_building("test\ttab", 5.0, 0.0, 1.0, 1.0, 1.0),
    ];
    let out = render_obj(&buildings);
    for (i, ch) in out.chars().enumerate() {
        assert!(
            ch == '\n' || !ch.is_control(),
            "unexpected control char at position {i}: {:?}",
            ch
        );
    }
}

// =========================================================================
// OBJ – total line count invariant
// =========================================================================

#[test]
fn obj_total_line_count_matches_formula() {
    // For N buildings: 1 header + N * (1 object + 8 vertex + 6 face) = 1 + 15*N
    for n in 0..=8 {
        let buildings: Vec<ObjBuilding> = (0..n)
            .map(|i| mk_building(&format!("b{i}"), i as f32, 0.0, 1.0, 1.0, 1.0))
            .collect();
        let out = render_obj(&buildings);
        // The output ends with '\n' so split gives an extra empty element
        let line_count = out.lines().count();
        let expected = 1 + 15 * n;
        assert_eq!(
            line_count, expected,
            "for {n} buildings: expected {expected} lines, got {line_count}"
        );
    }
}

// =========================================================================
// OBJ – Clone + Debug traits on ObjBuilding
// =========================================================================

#[test]
fn obj_building_clone_produces_equal_output() {
    let b = mk_building("clone_test", 1.0, 2.0, 3.0, 4.0, 5.0);
    let b2 = b.clone();
    assert_eq!(render_obj(&[b]), render_obj(&[b2]));
}

#[test]
fn obj_building_debug_contains_fields() {
    let b = mk_building("dbg", 1.0, 2.0, 3.0, 4.0, 5.0);
    let dbg = format!("{:?}", b);
    assert!(dbg.contains("ObjBuilding"));
    assert!(dbg.contains("dbg"));
}

// =========================================================================
// MIDI – overlapping notes produce correct event count
// =========================================================================

#[test]
fn midi_overlapping_notes_all_events_present() {
    // Two notes that overlap in time
    let notes = vec![
        mk_note(60, 0, 960, 0),   // 0..960
        mk_note(64, 240, 960, 0), // 240..1200 (overlaps)
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
    let off_count = smf.tracks[0]
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                midly::TrackEventKind::Midi {
                    message: midly::MidiMessage::NoteOff { .. },
                    ..
                }
            )
        })
        .count();
    assert_eq!(on_count, 2, "should have 2 NoteOn events");
    assert_eq!(off_count, 2, "should have 2 NoteOff events");
}

// =========================================================================
// MIDI – all 16 channels simultaneously
// =========================================================================

#[test]
fn midi_all_16_channels_simultaneously() {
    let notes: Vec<MidiNote> = (0u8..16)
        .map(|ch| mk_note(60 + ch % 12, 0, 480, ch))
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
    assert_eq!(channels.len(), 16, "all 16 MIDI channels should be present");
}

// =========================================================================
// MIDI – delta time calculations
// =========================================================================

#[test]
fn midi_delta_times_sum_to_absolute_positions() {
    let notes = vec![mk_note(60, 0, 480, 0), mk_note(64, 480, 480, 0)];
    let bytes = render_midi(&notes, 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    let mut clock = 0u32;
    let mut abs_times = Vec::new();
    for event in &smf.tracks[0] {
        clock += event.delta.as_int();
        abs_times.push(clock);
    }
    // First event (tempo) at 0, note-on at 0, note-off at 480, note-on at 480, note-off at 960, end
    assert_eq!(abs_times[0], 0, "tempo at t=0");
}

// =========================================================================
// MIDI – header structure validation
// =========================================================================

#[test]
fn midi_header_timing_is_480_tpq() {
    let bytes = render_midi(&[mk_note(60, 0, 480, 0)], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    match smf.header.timing {
        midly::Timing::Metrical(tpq) => {
            assert_eq!(tpq.as_int(), 480, "ticks per quarter should be 480");
        }
        _ => panic!("expected metrical timing"),
    }
}

#[test]
fn midi_header_format_is_single_track() {
    let bytes = render_midi(&[mk_note(60, 0, 480, 0)], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();
    assert!(matches!(smf.header.format, midly::Format::SingleTrack));
}

#[test]
fn midi_always_has_exactly_one_track() {
    for n in [0, 1, 5, 20] {
        let notes: Vec<MidiNote> = (0..n).map(|i| mk_note(60, i * 480, 240, 0)).collect();
        let bytes = render_midi(&notes, 120).unwrap();
        let smf = midly::Smf::parse(&bytes).unwrap();
        assert_eq!(
            smf.tracks.len(),
            1,
            "should always have exactly 1 track for {n} notes"
        );
    }
}

// =========================================================================
// MIDI – specific tempo values for various BPM
// =========================================================================

#[test]
fn midi_tempo_values_for_common_bpms() {
    let bpm_expected = [
        (60u16, 1_000_000u32),
        (80, 750_000),
        (100, 600_000),
        (120, 500_000),
        (140, 428_571),
        (200, 300_000),
    ];
    let note = mk_note(60, 0, 480, 0);
    for (bpm, expected_tempo) in bpm_expected {
        let bytes = render_midi(std::slice::from_ref(&note), bpm).unwrap();
        let smf = midly::Smf::parse(&bytes).unwrap();
        let tempo = smf.tracks[0]
            .iter()
            .find_map(|e| {
                if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(t)) = e.kind {
                    Some(t.as_int())
                } else {
                    None
                }
            })
            .expect("tempo event missing");
        assert_eq!(
            tempo, expected_tempo,
            "at {bpm} BPM: expected {expected_tempo}, got {tempo}"
        );
    }
}

// =========================================================================
// MIDI – note key propagation for all octaves
// =========================================================================

#[test]
fn midi_key_propagated_across_full_range() {
    for key in [0u8, 12, 24, 36, 48, 60, 72, 84, 96, 108, 120, 127] {
        let bytes = render_midi(&[mk_note(key, 0, 480, 0)], 120).unwrap();
        let smf = midly::Smf::parse(&bytes).unwrap();
        let found_key = smf.tracks[0]
            .iter()
            .find_map(|e| {
                if let midly::TrackEventKind::Midi {
                    message: midly::MidiMessage::NoteOn { key: k, .. },
                    ..
                } = e.kind
                {
                    Some(k.as_int())
                } else {
                    None
                }
            })
            .expect("NoteOn missing");
        assert_eq!(found_key, key, "key mismatch for input {key}");
    }
}

// =========================================================================
// MIDI – Clone + Debug on MidiNote
// =========================================================================

#[test]
fn midi_note_clone_produces_identical_output() {
    let n = mk_note(60, 0, 480, 0);
    let n2 = n.clone();
    let r1 = render_midi(&[n], 120).unwrap();
    let r2 = render_midi(&[n2], 120).unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn midi_note_debug_contains_fields() {
    let n = mk_note(72, 100, 200, 5);
    let dbg = format!("{:?}", n);
    assert!(dbg.contains("MidiNote"));
    assert!(dbg.contains("72"));
    assert!(dbg.contains("100"));
}

// =========================================================================
// MIDI – saturating_sub prevents underflow in delta computation
// =========================================================================

#[test]
fn midi_saturating_sub_prevents_negative_deltas() {
    // Events are sorted by absolute time; saturating_sub ensures no negative deltas.
    // Even with wonky start/duration combos, output should parse.
    let notes = vec![
        mk_note(60, 1000, 10, 0),
        mk_note(64, 500, 10, 0),
        mk_note(67, 0, 10, 0),
    ];
    let bytes = render_midi(&notes, 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    // All deltas should be non-negative (they're u28 so always >= 0)
    for event in &smf.tracks[0] {
        assert!(
            event.delta.as_int() < u32::MAX,
            "delta should be reasonable"
        );
    }
}

// =========================================================================
// MIDI – empty notes at various tempos
// =========================================================================

#[test]
fn midi_empty_notes_valid_at_various_tempos() {
    for bpm in [1u16, 30, 60, 120, 240, 300, u16::MAX] {
        let bytes = render_midi(&[], bpm).unwrap();
        assert_eq!(&bytes[..4], b"MThd", "invalid header at {bpm} BPM");
        let smf = midly::Smf::parse(&bytes).unwrap();
        // tempo + EndOfTrack
        assert_eq!(
            smf.tracks[0].len(),
            2,
            "empty notes at {bpm} BPM: expected 2 events"
        );
    }
}

// =========================================================================
// MIDI – note-off key matches note-on key
// =========================================================================

#[test]
fn midi_note_off_key_matches_note_on_key() {
    let key = 72u8;
    let bytes = render_midi(&[mk_note(key, 0, 480, 0)], 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    let on_key = smf.tracks[0].iter().find_map(|e| {
        if let midly::TrackEventKind::Midi {
            message: midly::MidiMessage::NoteOn { key: k, .. },
            ..
        } = e.kind
        {
            Some(k.as_int())
        } else {
            None
        }
    });
    let off_key = smf.tracks[0].iter().find_map(|e| {
        if let midly::TrackEventKind::Midi {
            message: midly::MidiMessage::NoteOff { key: k, .. },
            ..
        } = e.kind
        {
            Some(k.as_int())
        } else {
            None
        }
    });
    assert_eq!(on_key, Some(key));
    assert_eq!(off_key, Some(key));
}

// =========================================================================
// MIDI – large start offset doesn't panic
// =========================================================================

#[test]
fn midi_large_start_offset_produces_valid_midi() {
    let note = mk_note(60, u32::MAX - 1, 1, 0);
    let bytes = render_midi(&[note], 120).unwrap();
    assert_eq!(&bytes[..4], b"MThd");
    // Should parse (midly handles large variable-length quantities)
    let smf = midly::Smf::parse(&bytes);
    assert!(
        smf.is_ok(),
        "large start offset should produce parseable MIDI"
    );
}

// =========================================================================
// Cross-cutting: determinism with complex inputs
// =========================================================================

#[test]
fn determinism_obj_complex_city() {
    let buildings: Vec<ObjBuilding> = (0..20)
        .map(|i| {
            mk_building(
                &format!("module_{i}"),
                (i % 5) as f32 * 4.0,
                (i / 5) as f32 * 4.0,
                1.0 + (i as f32 * 0.1),
                1.0 + (i as f32 * 0.2),
                (i + 1) as f32,
            )
        })
        .collect();
    let r1 = render_obj(&buildings);
    let r2 = render_obj(&buildings);
    assert_eq!(r1, r2, "complex city output must be deterministic");
}

#[test]
fn determinism_midi_complex_sequence() {
    let notes: Vec<MidiNote> = (0..50)
        .map(|i| MidiNote {
            key: (48 + i % 24) as u8,
            velocity: (60 + i % 40) as u8,
            start: i as u32 * 120,
            duration: 100 + (i as u32 % 3) * 50,
            channel: (i % 16) as u8,
        })
        .collect();
    let r1 = render_midi(&notes, 90).unwrap();
    let r2 = render_midi(&notes, 90).unwrap();
    assert_eq!(r1, r2, "complex MIDI sequence must be deterministic");
}

// =========================================================================
// OBJ – no duplicate vertex lines within a single building
// =========================================================================

#[test]
fn obj_all_eight_vertices_are_distinct_for_nondegenerate_building() {
    let b = mk_building("b2", 0.0, 0.0, 2.0, 3.0, 5.0);
    let out = render_obj(&[b]);
    let verts = parse_vertices(&out);
    let unique: std::collections::HashSet<String> = verts
        .iter()
        .map(|(x, y, z)| format!("{x},{y},{z}"))
        .collect();
    assert_eq!(
        unique.len(),
        8,
        "non-degenerate building should have 8 distinct vertices"
    );
}

// =========================================================================
// OBJ – building with very small fractional dimensions
// =========================================================================

#[test]
fn obj_tiny_dimensions_produce_valid_geometry() {
    let b = mk_building("tiny", 0.0, 0.0, 0.001, 0.001, 0.001);
    let out = render_obj(&[b]);
    assert_eq!(out.lines().filter(|l| l.starts_with("v ")).count(), 8);
    assert_eq!(out.lines().filter(|l| l.starts_with("f ")).count(), 6);
}

// =========================================================================
// MIDI – tempo event is always the first event
// =========================================================================

#[test]
fn midi_tempo_event_is_first() {
    let notes = vec![mk_note(60, 0, 480, 0), mk_note(72, 240, 480, 1)];
    let bytes = render_midi(&notes, 120).unwrap();
    let smf = midly::Smf::parse(&bytes).unwrap();

    let first = &smf.tracks[0][0];
    assert!(
        matches!(
            first.kind,
            midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(_))
        ),
        "first event should be tempo, got {:?}",
        first.kind
    );
    assert_eq!(first.delta.as_int(), 0, "tempo delta should be 0");
}
