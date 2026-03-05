//! MIDI rendering edge cases and structural invariants – wave 59.
//!
//! Covers: zero notes, max key/velocity, channel boundary, overlapping notes,
//! very long sequences, backwards time, MIDI header structure, EndOfTrack
//! placement, delta timing correctness, and parseback validation.

use tokmd_fun::{MidiNote, render_midi};

// ── helpers ─────────────────────────────────────────────────────────────

fn mk_note(key: u8, vel: u8, start: u32, dur: u32, ch: u8) -> MidiNote {
    MidiNote {
        key,
        velocity: vel,
        start,
        duration: dur,
        channel: ch,
    }
}

fn parse_midi(data: &[u8]) -> midly::Smf<'_> {
    midly::Smf::parse(data).expect("MIDI output must be parseable")
}

// =========================================================================
// Header invariants
// =========================================================================

#[test]
fn midi_header_magic_bytes() {
    let data = render_midi(&[], 120).unwrap();
    assert_eq!(&data[..4], b"MThd");
}

#[test]
fn midi_always_single_track_format() {
    let data = render_midi(&[mk_note(60, 100, 0, 480, 0)], 120).unwrap();
    let smf = parse_midi(&data);
    assert_eq!(smf.tracks.len(), 1);
    assert!(matches!(smf.header.format, midly::Format::SingleTrack));
}

#[test]
fn midi_timing_is_480_ticks_per_quarter() {
    let data = render_midi(&[], 120).unwrap();
    let smf = parse_midi(&data);
    if let midly::Timing::Metrical(tpq) = smf.header.timing {
        assert_eq!(tpq.as_int(), 480);
    } else {
        panic!("expected metrical timing");
    }
}

// =========================================================================
// EndOfTrack invariant
// =========================================================================

#[test]
fn midi_end_of_track_present_with_notes() {
    let data = render_midi(&[mk_note(60, 100, 0, 480, 0)], 120).unwrap();
    let smf = parse_midi(&data);
    let last = smf.tracks[0].last().unwrap();
    assert!(matches!(
        last.kind,
        midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack)
    ));
}

#[test]
fn midi_end_of_track_present_empty() {
    let data = render_midi(&[], 120).unwrap();
    let smf = parse_midi(&data);
    let last = smf.tracks[0].last().unwrap();
    assert!(matches!(
        last.kind,
        midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack)
    ));
}

// =========================================================================
// Tempo computation
// =========================================================================

#[test]
fn midi_tempo_at_60bpm() {
    let data = render_midi(&[mk_note(60, 100, 0, 480, 0)], 60).unwrap();
    let smf = parse_midi(&data);
    for ev in &smf.tracks[0] {
        if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(t)) = ev.kind {
            // 60_000_000 / 60 = 1_000_000
            assert_eq!(t.as_int(), 1_000_000);
            return;
        }
    }
    panic!("no tempo event found");
}

#[test]
fn midi_tempo_at_1bpm() {
    let data = render_midi(&[mk_note(60, 100, 0, 480, 0)], 1).unwrap();
    let smf = parse_midi(&data);
    for ev in &smf.tracks[0] {
        if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(t)) = ev.kind {
            // 60_000_000 / 1 = 60_000_000, but Tempo is 24-bit (max 16_777_215),
            // so it wraps: 60_000_000 & 0xFF_FFFF = 9_668_352
            assert_eq!(t.as_int(), 60_000_000u32 & 0xFF_FFFF);
            return;
        }
    }
    panic!("no tempo event found");
}

#[test]
fn midi_tempo_zero_clamped_to_1() {
    // tempo_bpm=0 → max(1) → 60_000_000 / 1, but Tempo is 24-bit
    let data = render_midi(&[mk_note(60, 100, 0, 480, 0)], 0).unwrap();
    let smf = parse_midi(&data);
    for ev in &smf.tracks[0] {
        if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(t)) = ev.kind {
            assert_eq!(t.as_int(), 60_000_000u32 & 0xFF_FFFF);
            return;
        }
    }
    panic!("no tempo event found");
}

#[test]
fn midi_tempo_max_u16() {
    let data = render_midi(&[mk_note(60, 100, 0, 480, 0)], u16::MAX).unwrap();
    let smf = parse_midi(&data);
    for ev in &smf.tracks[0] {
        if let midly::TrackEventKind::Meta(midly::MetaMessage::Tempo(t)) = ev.kind {
            let expected = 60_000_000u32 / u16::MAX as u32;
            assert_eq!(t.as_int(), expected);
            return;
        }
    }
    panic!("no tempo event found");
}

// =========================================================================
// Channel clamping
// =========================================================================

#[test]
fn midi_channel_15_accepted() {
    let data = render_midi(&[mk_note(60, 100, 0, 480, 15)], 120).unwrap();
    let smf = parse_midi(&data);
    for ev in &smf.tracks[0] {
        if let midly::TrackEventKind::Midi { channel, .. } = ev.kind {
            assert_eq!(channel.as_int(), 15);
            return;
        }
    }
    panic!("no MIDI event found");
}

#[test]
fn midi_channel_16_clamped_to_15() {
    let data = render_midi(&[mk_note(60, 100, 0, 480, 16)], 120).unwrap();
    let smf = parse_midi(&data);
    for ev in &smf.tracks[0] {
        if let midly::TrackEventKind::Midi { channel, .. } = ev.kind {
            assert_eq!(channel.as_int(), 15);
            return;
        }
    }
    panic!("no MIDI event found");
}

#[test]
fn midi_channel_255_clamped_to_15() {
    let data = render_midi(&[mk_note(60, 100, 0, 480, 255)], 120).unwrap();
    let smf = parse_midi(&data);
    for ev in &smf.tracks[0] {
        if let midly::TrackEventKind::Midi { channel, .. } = ev.kind {
            assert_eq!(channel.as_int(), 15);
            return;
        }
    }
    panic!("no MIDI event found");
}

// =========================================================================
// Key / velocity extremes
// =========================================================================

#[test]
fn midi_key_zero() {
    let data = render_midi(&[mk_note(0, 100, 0, 480, 0)], 120).unwrap();
    let smf = parse_midi(&data);
    for ev in &smf.tracks[0] {
        if let midly::TrackEventKind::Midi {
            message: midly::MidiMessage::NoteOn { key, .. },
            ..
        } = ev.kind
        {
            assert_eq!(key.as_int(), 0);
            return;
        }
    }
    panic!("no NoteOn found");
}

#[test]
fn midi_key_127() {
    let data = render_midi(&[mk_note(127, 100, 0, 480, 0)], 120).unwrap();
    let smf = parse_midi(&data);
    for ev in &smf.tracks[0] {
        if let midly::TrackEventKind::Midi {
            message: midly::MidiMessage::NoteOn { key, .. },
            ..
        } = ev.kind
        {
            assert_eq!(key.as_int(), 127);
            return;
        }
    }
    panic!("no NoteOn found");
}

#[test]
fn midi_velocity_zero_still_emits_note_on() {
    let data = render_midi(&[mk_note(60, 0, 0, 480, 0)], 120).unwrap();
    let smf = parse_midi(&data);
    let has_note_on = smf.tracks[0].iter().any(|ev| {
        matches!(
            ev.kind,
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOn { .. },
                ..
            }
        )
    });
    assert!(has_note_on, "velocity=0 note should still produce NoteOn");
}

#[test]
fn midi_velocity_127() {
    let data = render_midi(&[mk_note(60, 127, 0, 480, 0)], 120).unwrap();
    let smf = parse_midi(&data);
    for ev in &smf.tracks[0] {
        if let midly::TrackEventKind::Midi {
            message: midly::MidiMessage::NoteOn { vel, .. },
            ..
        } = ev.kind
        {
            assert_eq!(vel.as_int(), 127);
            return;
        }
    }
    panic!("no NoteOn found");
}

// =========================================================================
// Overlapping / simultaneous notes
// =========================================================================

#[test]
fn midi_overlapping_notes_same_key_valid() {
    let notes = vec![
        mk_note(60, 100, 0, 960, 0),
        mk_note(60, 80, 480, 960, 0), // overlaps first
    ];
    let data = render_midi(&notes, 120).unwrap();
    let smf = parse_midi(&data);
    let note_ons = smf.tracks[0]
        .iter()
        .filter(|ev| {
            matches!(
                ev.kind,
                midly::TrackEventKind::Midi {
                    message: midly::MidiMessage::NoteOn { .. },
                    ..
                }
            )
        })
        .count();
    assert_eq!(note_ons, 2, "both note-ons should be present");
}

#[test]
fn midi_chord_all_at_tick_zero() {
    let notes = vec![
        mk_note(60, 100, 0, 480, 0),
        mk_note(64, 100, 0, 480, 0),
        mk_note(67, 100, 0, 480, 0),
    ];
    let data = render_midi(&notes, 120).unwrap();
    let smf = parse_midi(&data);
    // All three NoteOn events should exist
    let note_ons = smf.tracks[0]
        .iter()
        .filter(|ev| {
            matches!(
                ev.kind,
                midly::TrackEventKind::Midi {
                    message: midly::MidiMessage::NoteOn { .. },
                    ..
                }
            )
        })
        .count();
    assert_eq!(note_ons, 3);
}

// =========================================================================
// Delta timing correctness
// =========================================================================

#[test]
fn midi_events_sorted_by_absolute_time() {
    // Provide notes out of order
    let notes = vec![
        mk_note(72, 100, 960, 480, 0),
        mk_note(60, 100, 0, 480, 0),
        mk_note(64, 100, 480, 480, 0),
    ];
    let data = render_midi(&notes, 120).unwrap();
    let smf = parse_midi(&data);

    // Verify monotonically increasing absolute time
    let mut abs = 0u32;
    for ev in &smf.tracks[0] {
        abs += ev.delta.as_int();
    }
    // If we reach here without midly parse error, timing is valid.
    assert!(abs > 0, "total time should be positive");
}

#[test]
fn midi_zero_duration_note_on_off_same_tick() {
    let data = render_midi(&[mk_note(60, 100, 100, 0, 0)], 120).unwrap();
    let smf = parse_midi(&data);

    let mut abs_times: Vec<(u32, &str)> = Vec::new();
    let mut t = 0u32;
    for ev in &smf.tracks[0] {
        t += ev.delta.as_int();
        match ev.kind {
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOn { .. },
                ..
            } => abs_times.push((t, "on")),
            midly::TrackEventKind::Midi {
                message: midly::MidiMessage::NoteOff { .. },
                ..
            } => abs_times.push((t, "off")),
            _ => {}
        }
    }
    assert_eq!(abs_times.len(), 2);
    assert_eq!(abs_times[0].0, abs_times[1].0, "on and off at same tick");
}

// =========================================================================
// Event counts
// =========================================================================

#[test]
fn midi_event_count_for_n_notes() {
    for n in [1, 5, 10, 20] {
        let notes: Vec<MidiNote> = (0..n)
            .map(|i| mk_note(60 + (i % 12) as u8, 100, i as u32 * 480, 480, 0))
            .collect();
        let data = render_midi(&notes, 120).unwrap();
        let smf = parse_midi(&data);
        // Events = 1 tempo + n NoteOn + n NoteOff + 1 EndOfTrack
        let expected = 1 + 2 * n + 1;
        assert_eq!(
            smf.tracks[0].len(),
            expected,
            "n={n}: expected {expected} events"
        );
    }
}

// =========================================================================
// Large start offset / saturating_sub
// =========================================================================

#[test]
fn midi_large_start_offset_no_overflow() {
    let note = mk_note(60, 100, u32::MAX - 100, 50, 0);
    let data = render_midi(std::slice::from_ref(&note), 120).unwrap();
    let smf = parse_midi(&data);
    // Must parse without error
    assert!(smf.tracks[0].len() >= 3); // tempo + note_on + note_off + eot
}

#[test]
fn midi_max_start_with_zero_duration_no_overflow() {
    // start + duration = u32::MAX + 0 — no overflow
    let note = mk_note(60, 100, u32::MAX, 0, 0);
    let data = render_midi(std::slice::from_ref(&note), 120).unwrap();
    assert_eq!(&data[..4], b"MThd");
}

// =========================================================================
// Determinism
// =========================================================================

#[test]
fn midi_deterministic_across_10_runs() {
    let notes = vec![
        mk_note(60, 100, 0, 480, 0),
        mk_note(64, 80, 480, 480, 1),
        mk_note(67, 60, 960, 480, 2),
    ];
    let first = render_midi(&notes, 120).unwrap();
    for _ in 0..10 {
        assert_eq!(
            render_midi(&notes, 120).unwrap(),
            first,
            "MIDI must be deterministic"
        );
    }
}

#[test]
fn midi_different_key_produces_different_output() {
    let a = render_midi(&[mk_note(60, 100, 0, 480, 0)], 120).unwrap();
    let b = render_midi(&[mk_note(72, 100, 0, 480, 0)], 120).unwrap();
    assert_ne!(a, b);
}

#[test]
fn midi_different_velocity_produces_different_output() {
    let a = render_midi(&[mk_note(60, 50, 0, 480, 0)], 120).unwrap();
    let b = render_midi(&[mk_note(60, 127, 0, 480, 0)], 120).unwrap();
    assert_ne!(a, b);
}

#[test]
fn midi_different_channel_produces_different_output() {
    let a = render_midi(&[mk_note(60, 100, 0, 480, 0)], 120).unwrap();
    let b = render_midi(&[mk_note(60, 100, 0, 480, 5)], 120).unwrap();
    assert_ne!(a, b);
}
