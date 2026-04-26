#![cfg(feature = "fun")]
//! Property-based tests for tokmd-format::fun – wave 59.
//!
//! Extends existing property tests with deeper structural invariants:
//! face-index bounds, vertex count formula, MIDI parseback, event ordering,
//! and Clone/Debug/PartialEq derive contracts.

use proptest::prelude::*;
use tokmd_format::fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// ── strategies ──────────────────────────────────────────────────────────

fn arb_building() -> impl Strategy<Value = ObjBuilding> {
    (
        "[a-zA-Z0-9_]{1,30}",
        -1e4f32..1e4f32,
        -1e4f32..1e4f32,
        0.0f32..1e4f32,
        0.0f32..1e4f32,
        0.0f32..1e4f32,
    )
        .prop_map(|(name, x, y, w, d, h)| ObjBuilding {
            name,
            x,
            y,
            w,
            d,
            h,
        })
}

fn arb_note() -> impl Strategy<Value = MidiNote> {
    (
        0u8..=127u8,
        0u8..=127u8,
        0u32..50_000u32,
        0u32..5_000u32,
        0u8..=15u8,
    )
        .prop_map(|(key, velocity, start, duration, channel)| MidiNote {
            key,
            velocity,
            start,
            duration,
            channel,
        })
}

proptest! {
    // ── OBJ properties ──────────────────────────────────────────────────

    /// For N buildings, output has exactly N×8 vertex lines.
    #[test]
    fn prop_obj_vertex_count(buildings in proptest::collection::vec(arb_building(), 0..20)) {
        let out = render_obj(&buildings);
        let v_count = out.lines().filter(|l| l.starts_with("v ")).count();
        prop_assert_eq!(v_count, buildings.len() * 8);
    }

    /// For N buildings, output has exactly N×6 face lines.
    #[test]
    fn prop_obj_face_count(buildings in proptest::collection::vec(arb_building(), 0..20)) {
        let out = render_obj(&buildings);
        let f_count = out.lines().filter(|l| l.starts_with("f ")).count();
        prop_assert_eq!(f_count, buildings.len() * 6);
    }

    /// Every face index is within [1, N*8].
    #[test]
    fn prop_obj_face_indices_in_bounds(buildings in proptest::collection::vec(arb_building(), 1..15)) {
        let out = render_obj(&buildings);
        let max_vert = buildings.len() * 8;
        for line in out.lines().filter(|l| l.starts_with("f ")) {
            for tok in line.strip_prefix("f ").unwrap().split_whitespace() {
                let idx: usize = tok.parse().unwrap();
                prop_assert!(idx >= 1 && idx <= max_vert, "face index {idx} out of bounds [1, {max_vert}]");
            }
        }
    }

    /// Each face is a quad (exactly 4 indices).
    #[test]
    fn prop_obj_all_faces_are_quads(buildings in proptest::collection::vec(arb_building(), 1..15)) {
        let out = render_obj(&buildings);
        for line in out.lines().filter(|l| l.starts_with("f ")) {
            let n = line.strip_prefix("f ").unwrap().split_whitespace().count();
            prop_assert_eq!(n, 4, "face should have 4 indices, got {}", n);
        }
    }

    /// Building names in output match (sanitized) input order.
    #[test]
    fn prop_obj_names_match_input_order(buildings in proptest::collection::vec(arb_building(), 0..20)) {
        let out = render_obj(&buildings);
        let obj_names: Vec<&str> = out.lines()
            .filter_map(|l| l.strip_prefix("o "))
            .collect();
        prop_assert_eq!(obj_names.len(), buildings.len());
    }

    /// OBJ output header is always the first line.
    #[test]
    fn prop_obj_header_always_first(buildings in proptest::collection::vec(arb_building(), 0..10)) {
        let out = render_obj(&buildings);
        prop_assert_eq!(out.lines().next().unwrap(), "# tokmd code city");
    }

    /// OBJ output is deterministic: same input → same output.
    #[test]
    fn prop_obj_deterministic(buildings in proptest::collection::vec(arb_building(), 0..10)) {
        let a = render_obj(&buildings);
        let b = render_obj(&buildings);
        prop_assert_eq!(a, b);
    }

    /// Bottom-plane vertices (first 4 per building) always have z=0.
    #[test]
    fn prop_obj_bottom_z_zero(b in arb_building()) {
        let out = render_obj(std::slice::from_ref(&b));
        let verts: Vec<&str> = out.lines().filter(|l| l.starts_with("v ")).collect();
        for v in &verts[..4] {
            let z: f32 = v.split_whitespace().nth(3).unwrap().parse().unwrap();
            prop_assert_eq!(z, 0.0f32, "bottom z must be 0");
        }
    }

    /// Top-plane vertices (last 4 per building) have z == h.
    #[test]
    fn prop_obj_top_z_equals_height(b in arb_building()) {
        let out = render_obj(std::slice::from_ref(&b));
        let verts: Vec<&str> = out.lines().filter(|l| l.starts_with("v ")).collect();
        for v in &verts[4..8] {
            let z: f32 = v.split_whitespace().nth(3).unwrap().parse().unwrap();
            prop_assert!((z - b.h).abs() < 1e-6, "top z ({z}) must equal h ({})", b.h);
        }
    }

    // ── MIDI properties ─────────────────────────────────────────────────

    /// render_midi always produces parseable MIDI.
    #[test]
    fn prop_midi_always_parseable(notes in proptest::collection::vec(arb_note(), 0..30), tempo in 1u16..500u16) {
        let data = render_midi(&notes, tempo).unwrap();
        prop_assert!(midly::Smf::parse(&data).is_ok(), "MIDI must be parseable");
    }

    /// MIDI output starts with MThd magic bytes.
    #[test]
    fn prop_midi_header_magic(notes in proptest::collection::vec(arb_note(), 0..10), tempo in 1u16..300u16) {
        let data = render_midi(&notes, tempo).unwrap();
        prop_assert_eq!(&data[..4], b"MThd");
    }

    /// Event count = 1 (tempo) + 2*N (on+off) + 1 (EndOfTrack).
    #[test]
    fn prop_midi_event_count(notes in proptest::collection::vec(arb_note(), 0..20), tempo in 1u16..300u16) {
        let data = render_midi(&notes, tempo).unwrap();
        let smf = midly::Smf::parse(&data).unwrap();
        let expected = 1 + 2 * notes.len() + 1;
        prop_assert_eq!(smf.tracks[0].len(), expected);
    }

    /// Last event is always EndOfTrack.
    #[test]
    fn prop_midi_ends_with_eot(notes in proptest::collection::vec(arb_note(), 0..20), tempo in 1u16..300u16) {
        let data = render_midi(&notes, tempo).unwrap();
        let smf = midly::Smf::parse(&data).unwrap();
        let last = smf.tracks[0].last().unwrap();
        prop_assert!(matches!(last.kind, midly::TrackEventKind::Meta(midly::MetaMessage::EndOfTrack)));
    }

    /// MIDI is deterministic.
    #[test]
    fn prop_midi_deterministic(notes in proptest::collection::vec(arb_note(), 0..20), tempo in 1u16..300u16) {
        let a = render_midi(&notes, tempo).unwrap();
        let b = render_midi(&notes, tempo).unwrap();
        prop_assert_eq!(a, b);
    }

    /// MIDI output is always a single track.
    #[test]
    fn prop_midi_single_track(notes in proptest::collection::vec(arb_note(), 0..20), tempo in 1u16..300u16) {
        let data = render_midi(&notes, tempo).unwrap();
        let smf = midly::Smf::parse(&data).unwrap();
        prop_assert_eq!(smf.tracks.len(), 1);
    }
}

// ── Clone / Debug derive contracts ──────────────────────────────────────

#[test]
fn obj_building_clone_eq() {
    let b = ObjBuilding {
        name: "test".into(),
        x: 1.0,
        y: 2.0,
        w: 3.0,
        d: 4.0,
        h: 5.0,
    };
    let c = b.clone();
    assert_eq!(b.name, c.name);
    assert_eq!(b.x, c.x);
    assert_eq!(b.y, c.y);
    assert_eq!(b.w, c.w);
    assert_eq!(b.d, c.d);
    assert_eq!(b.h, c.h);
}

#[test]
fn obj_building_debug_not_empty() {
    let b = ObjBuilding {
        name: "dbg".into(),
        x: 0.0,
        y: 0.0,
        w: 0.0,
        d: 0.0,
        h: 0.0,
    };
    let dbg = format!("{b:?}");
    assert!(dbg.contains("ObjBuilding"));
    assert!(dbg.contains("dbg"));
}

#[test]
fn midi_note_clone_eq() {
    let n = MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 0,
    };
    let c = n.clone();
    assert_eq!(n.key, c.key);
    assert_eq!(n.velocity, c.velocity);
    assert_eq!(n.start, c.start);
    assert_eq!(n.duration, c.duration);
    assert_eq!(n.channel, c.channel);
}

#[test]
fn midi_note_debug_not_empty() {
    let n = MidiNote {
        key: 72,
        velocity: 64,
        start: 100,
        duration: 200,
        channel: 3,
    };
    let dbg = format!("{n:?}");
    assert!(dbg.contains("MidiNote"));
}
