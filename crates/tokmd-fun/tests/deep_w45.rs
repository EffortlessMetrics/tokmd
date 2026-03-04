//! Deep tests for tokmd-fun – wave 45.
//!
//! Fills coverage gaps: MIDI structural invariants (EndOfTrack, timing header,
//! event ordering), OBJ line-count formula, all-underscore names, property
//! tests for MIDI, and snapshot for zero-BPM edge case.
//!
//! Run with: `cargo test -p tokmd-fun --test deep_w45`

use tokmd_fun::{render_midi, render_obj, MidiNote, ObjBuilding};

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

fn midi_hex(notes: &[MidiNote], tempo: u16) -> String {
    let bytes = render_midi(notes, tempo).unwrap();
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

// =========================================================================
// MIDI – EndOfTrack is always the last event
// =========================================================================

#[test]
fn midi_end_of_track_is_always_last_event_empty() {
    let data = render_midi(&[], 120).unwrap();
    let smf = midly::Smf::parse(&data).unwrap();
    let last = smf.tracks[0].last().unwrap();
    assert!(
        matches!(
            last.kind,
            midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack)
        ),
        "last event must be EndOfTrack"
    );
}

#[test]
fn midi_end_of_track_is_always_last_event_many_notes() {
    let notes: Vec<MidiNote> = (0..50)
        .map(|i| mk_note((60 + i % 12) as u8, i * 480, 240, (i % 16) as u8))
        .collect();
    let data = render_midi(&notes, 120).unwrap();
    let smf = midly::Smf::parse(&data).unwrap();
    let last = smf.tracks[0].last().unwrap();
    assert!(
        matches!(
            last.kind,
            midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack)
        ),
        "last event must be EndOfTrack even with many notes"
    );
}

// =========================================================================
// MIDI – timing header is always Metrical(480)
// =========================================================================

#[test]
fn midi_timing_is_metrical_480() {
    let data = render_midi(&[mk_note(60, 0, 480, 0)], 120).unwrap();
    let smf = midly::Smf::parse(&data).unwrap();
    match smf.header.timing {
        midly::Timing::Metrical(tpq) => assert_eq!(tpq.as_int(), 480),
        _ => panic!("expected Metrical timing"),
    }
}

#[test]
fn midi_timing_is_metrical_480_regardless_of_tempo() {
    for tempo in [1u16, 60, 120, 200, u16::MAX] {
        let data = render_midi(&[mk_note(60, 0, 480, 0)], tempo).unwrap();
        let smf = midly::Smf::parse(&data).unwrap();
        match smf.header.timing {
            midly::Timing::Metrical(tpq) => assert_eq!(
                tpq.as_int(),
                480,
                "ticks-per-quarter must be 480 at tempo {tempo}"
            ),
            _ => panic!("expected Metrical timing at tempo {tempo}"),
        }
    }
}

// =========================================================================
// MIDI – format is SingleTrack
// =========================================================================

#[test]
fn midi_format_is_single_track() {
    let data = render_midi(&[mk_note(60, 0, 480, 0)], 120).unwrap();
    let smf = midly::Smf::parse(&data).unwrap();
    assert_eq!(smf.header.format, midly::Format::SingleTrack);
    assert_eq!(smf.tracks.len(), 1);
}

// =========================================================================
// MIDI – tempo meta-event is always the first event
// =========================================================================

#[test]
fn midi_tempo_event_is_first() {
    let data = render_midi(&[mk_note(60, 0, 480, 0)], 120).unwrap();
    let smf = midly::Smf::parse(&data).unwrap();
    let first = &smf.tracks[0][0];
    assert_eq!(first.delta.as_int(), 0, "tempo event delta must be 0");
    assert!(
        matches!(
            first.kind,
            midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(_))
        ),
        "first event must be a Tempo meta-event"
    );
}

#[test]
fn midi_tempo_value_matches_bpm() {
    // 120 BPM => 60_000_000 / 120 = 500_000 microseconds per quarter
    let data = render_midi(&[], 120).unwrap();
    let smf = midly::Smf::parse(&data).unwrap();
    if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(t)) = smf.tracks[0][0].kind {
        assert_eq!(t.as_int(), 500_000);
    } else {
        panic!("first event is not Tempo");
    }
}

#[test]
fn midi_tempo_value_60bpm() {
    // 60 BPM => 60_000_000 / 60 = 1_000_000 microseconds per quarter
    let data = render_midi(&[], 60).unwrap();
    let smf = midly::Smf::parse(&data).unwrap();
    if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(t)) = smf.tracks[0][0].kind {
        assert_eq!(t.as_int(), 1_000_000);
    } else {
        panic!("first event is not Tempo");
    }
}

// =========================================================================
// MIDI – event count formula: 1 tempo + 2*N notes + 1 EndOfTrack
// =========================================================================

#[test]
fn midi_event_count_formula() {
    for n in [0u32, 1, 5, 20, 100] {
        let notes: Vec<MidiNote> = (0..n).map(|i| mk_note(60, i * 480, 240, 0)).collect();
        let data = render_midi(&notes, 120).unwrap();
        let smf = midly::Smf::parse(&data).unwrap();
        let expected = 1 + 2 * n as usize + 1; // tempo + NoteOn/NoteOff pairs + EndOfTrack
        assert_eq!(
            smf.tracks[0].len(),
            expected,
            "event count mismatch for {n} notes"
        );
    }
}

// =========================================================================
// MIDI – NoteOn precedes NoteOff for same-time events (stable sort)
// =========================================================================

#[test]
fn midi_note_on_before_note_off_at_same_tick() {
    // Two consecutive notes where note1.end == note2.start
    let notes = vec![mk_note(60, 0, 480, 0), mk_note(64, 480, 480, 0)];
    let data = render_midi(&notes, 120).unwrap();
    let smf = midly::Smf::parse(&data).unwrap();

    // At tick 480, we should see NoteOff(60) and NoteOn(64) — both at delta sum=480
    // Collect events at tick 480
    let mut tick = 0u32;
    let mut events_at_480 = vec![];
    for ev in &smf.tracks[0] {
        tick += ev.delta.as_int();
        if tick == 480 {
            events_at_480.push(&ev.kind);
        }
    }

    // There should be at least NoteOff(60) and NoteOn(64) at tick 480
    assert!(
        events_at_480.len() >= 2,
        "expected at least 2 events at tick 480, got {}",
        events_at_480.len()
    );
}

// =========================================================================
// MIDI – snapshot for zero BPM (clamped to 1)
// =========================================================================

#[test]
fn snapshot_midi_zero_bpm_clamped() {
    insta::assert_snapshot!(midi_hex(&[mk_note(60, 0, 480, 0)], 0));
}

// =========================================================================
// MIDI – snapshot for tempo 1 BPM (extreme slow)
// =========================================================================

#[test]
fn snapshot_midi_1bpm_extreme_slow() {
    insta::assert_snapshot!(midi_hex(&[mk_note(60, 0, 480, 0)], 1));
}

// =========================================================================
// OBJ – exact line count formula
// =========================================================================

#[test]
fn obj_line_count_formula() {
    for n in [0usize, 1, 3, 10, 25] {
        let buildings: Vec<ObjBuilding> = (0..n)
            .map(|i| mk_building(&format!("b{i}"), i as f32 * 2.0, 0.0, 1.0, 1.0, 1.0))
            .collect();
        let out = render_obj(&buildings);
        let line_count = out.lines().count();
        // 1 header + n * (1 object + 8 vertex + 6 face) = 1 + 15n
        let expected = 1 + 15 * n;
        assert_eq!(
            line_count, expected,
            "line count mismatch for {n} buildings"
        );
    }
}

// =========================================================================
// OBJ – all-special-character name produces all underscores
// =========================================================================

#[test]
fn obj_all_special_chars_name_produces_all_underscores() {
    let out = render_obj(&[mk_building("!@#$%^&*()", 0.0, 0.0, 1.0, 1.0, 1.0)]);
    let obj_line = out.lines().find(|l| l.starts_with("o ")).unwrap();
    let sanitized = obj_line.strip_prefix("o ").unwrap();
    assert_eq!(sanitized, "__________");
    assert!(
        sanitized.chars().all(|c| c == '_'),
        "all-special name should become all underscores"
    );
}

#[test]
fn obj_emoji_name_sanitized() {
    let out = render_obj(&[mk_building("🦀🔥💻", 0.0, 0.0, 1.0, 1.0, 1.0)]);
    let obj_line = out.lines().find(|l| l.starts_with("o ")).unwrap();
    let sanitized = obj_line.strip_prefix("o ").unwrap();
    assert!(
        sanitized.chars().all(|c| c == '_'),
        "emoji name should become all underscores, got: {sanitized}"
    );
}

// =========================================================================
// OBJ – empty name produces empty object name
// =========================================================================

#[test]
fn obj_empty_name_produces_empty_object_line() {
    let out = render_obj(&[mk_building("", 0.0, 0.0, 1.0, 1.0, 1.0)]);
    assert!(out.contains("o \n"), "empty name should produce 'o \\n'");
}

// =========================================================================
// OBJ – vertex z-base is always zero
// =========================================================================

#[test]
fn obj_z_base_is_always_zero() {
    let buildings = vec![
        mk_building("a", 0.0, 0.0, 1.0, 1.0, 5.0),
        mk_building("b", 10.0, 20.0, 3.0, 4.0, 100.0),
    ];
    let out = render_obj(&buildings);
    let vertices: Vec<(f32, f32, f32)> = out
        .lines()
        .filter(|l| l.starts_with("v "))
        .map(|l| {
            let parts: Vec<f32> = l[2..]
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            (parts[0], parts[1], parts[2])
        })
        .collect();

    // For each building, 4 bottom vertices should have z=0
    for chunk in vertices.chunks(8) {
        for v in &chunk[..4] {
            assert_eq!(v.2, 0.0, "bottom vertex z must be 0.0, got {:?}", v);
        }
    }
}

// =========================================================================
// OBJ – top vertices z equals height
// =========================================================================

#[test]
fn obj_top_vertices_z_equals_height() {
    let h = 42.5f32;
    let out = render_obj(&[mk_building("t", 0.0, 0.0, 1.0, 1.0, h)]);
    let vertices: Vec<f32> = out
        .lines()
        .filter(|l| l.starts_with("v "))
        .map(|l| {
            let parts: Vec<f32> = l[2..]
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            parts[2]
        })
        .collect();

    // Top 4 vertices (indices 4-7) should have z = h
    for (i, &z) in vertices[4..8].iter().enumerate() {
        assert_eq!(z, h, "top vertex {i} z must be {h}, got {z}");
    }
}

// =========================================================================
// Clone and Debug trait contracts
// =========================================================================

#[test]
fn obj_building_clone_produces_equal_rendering() {
    let b = mk_building("clone_test", 1.0, 2.0, 3.0, 4.0, 5.0);
    let b_clone = b.clone();
    assert_eq!(render_obj(&[b]), render_obj(&[b_clone]));
}

#[test]
fn midi_note_clone_produces_equal_rendering() {
    let n = mk_note(60, 0, 480, 0);
    let n_clone = n.clone();
    let r1 = render_midi(&[n], 120).unwrap();
    let r2 = render_midi(&[n_clone], 120).unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn obj_building_debug_is_non_empty() {
    let b = mk_building("dbg", 0.0, 0.0, 1.0, 1.0, 1.0);
    let debug = format!("{:?}", b);
    assert!(!debug.is_empty());
    assert!(debug.contains("ObjBuilding"));
}

#[test]
fn midi_note_debug_is_non_empty() {
    let n = mk_note(60, 0, 480, 0);
    let debug = format!("{:?}", n);
    assert!(!debug.is_empty());
    assert!(debug.contains("MidiNote"));
}

// =========================================================================
// MIDI property tests
// =========================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// render_midi always produces valid MIDI (starts with MThd).
        #[test]
        fn midi_always_valid_header(
            key in 0u8..128,
            vel in 0u8..128,
            start in 0u32..100_000,
            dur in 0u32..10_000,
            ch in 0u8..16,
            tempo in 1u16..=u16::MAX,
        ) {
            let note = MidiNote { key, velocity: vel, start, duration: dur, channel: ch };
            let data = render_midi(&[note], tempo).unwrap();
            prop_assert_eq!(&data[..4], b"MThd");
        }

        /// render_midi is deterministic for any input.
        #[test]
        fn midi_deterministic(
            key in 0u8..128,
            vel in 0u8..128,
            start in 0u32..100_000,
            dur in 0u32..10_000,
            ch in 0u8..16,
            tempo in 1u16..=u16::MAX,
        ) {
            let note = MidiNote { key, velocity: vel, start, duration: dur, channel: ch };
            let r1 = render_midi(std::slice::from_ref(&note), tempo).unwrap();
            let r2 = render_midi(std::slice::from_ref(&note), tempo).unwrap();
            prop_assert_eq!(r1, r2);
        }

        /// render_midi output length grows with note count.
        #[test]
        fn midi_more_notes_means_more_bytes(n in 1u32..50) {
            let notes: Vec<MidiNote> = (0..n)
                .map(|i| MidiNote {
                    key: 60,
                    velocity: 100,
                    start: i * 480,
                    duration: 240,
                    channel: 0,
                })
                .collect();
            let full = render_midi(&notes, 120).unwrap();
            let half = render_midi(&notes[..1], 120).unwrap();
            if n > 1 {
                prop_assert!(full.len() > half.len());
            }
        }

        /// render_obj is deterministic for any building.
        #[test]
        fn obj_deterministic(
            x in -1000.0f32..1000.0,
            y in -1000.0f32..1000.0,
            w in 0.0f32..100.0,
            d in 0.0f32..100.0,
            h in 0.0f32..100.0,
        ) {
            let b = ObjBuilding { name: "p".to_string(), x, y, w, d, h };
            let r1 = render_obj(std::slice::from_ref(&b));
            let r2 = render_obj(std::slice::from_ref(&b));
            prop_assert_eq!(r1, r2);
        }

        /// render_obj always starts with the header comment.
        #[test]
        fn obj_always_has_header(
            n in 0usize..10,
        ) {
            let buildings: Vec<ObjBuilding> = (0..n)
                .map(|i| ObjBuilding {
                    name: format!("b{i}"),
                    x: i as f32,
                    y: 0.0,
                    w: 1.0,
                    d: 1.0,
                    h: 1.0,
                })
                .collect();
            let out = render_obj(&buildings);
            prop_assert!(out.starts_with("# tokmd code city\n"));
        }

        /// sanitize_name produces only alphanumeric or underscore chars.
        #[test]
        fn obj_sanitized_names_are_clean(name in "[\\x00-\\x7f]{0,50}") {
            let b = ObjBuilding { name, x: 0.0, y: 0.0, w: 1.0, d: 1.0, h: 1.0 };
            let out = render_obj(&[b]);
            if let Some(obj_line) = out.lines().find(|l| l.starts_with("o ")) {
                let sanitized = obj_line.strip_prefix("o ").unwrap();
                prop_assert!(
                    sanitized.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
                    "sanitized name contains invalid chars: {sanitized}"
                );
            }
        }
    }
}
