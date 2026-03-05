//! Deep tests for tokmd-fun – wave 69.
//!
//! Covers OBJ code-city rendering and MIDI generation with emphasis on
//! determinism, edge cases (zero/single/many buildings), structural
//! invariants, and eco-label-style inputs.

use tokmd_fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// ── helpers ─────────────────────────────────────────────────────────────

fn mk(name: &str, x: f32, y: f32, w: f32, d: f32, h: f32) -> ObjBuilding {
    ObjBuilding {
        name: name.into(),
        x,
        y,
        w,
        d,
        h,
    }
}

fn vertex_count(obj: &str) -> usize {
    obj.lines().filter(|l| l.starts_with("v ")).count()
}

fn face_count(obj: &str) -> usize {
    obj.lines().filter(|l| l.starts_with("f ")).count()
}

fn object_names(obj: &str) -> Vec<&str> {
    obj.lines().filter_map(|l| l.strip_prefix("o ")).collect()
}

// =========================================================================
// 1. OBJ determinism
// =========================================================================

#[test]
fn obj_deterministic_across_calls() {
    let buildings = vec![
        mk("alpha", 0.0, 0.0, 1.0, 1.0, 2.0),
        mk("beta", 2.0, 0.0, 1.0, 1.0, 3.0),
    ];
    let a = render_obj(&buildings);
    let b = render_obj(&buildings);
    assert_eq!(a, b, "render_obj must be deterministic");
}

#[test]
fn obj_deterministic_after_clone() {
    let buildings = vec![mk("x", 1.0, 2.0, 3.0, 4.0, 5.0)];
    let cloned = buildings.clone();
    assert_eq!(render_obj(&buildings), render_obj(&cloned));
}

// =========================================================================
// 2. OBJ structural invariants
// =========================================================================

#[test]
fn obj_header_always_present() {
    assert!(render_obj(&[]).starts_with("# tokmd code city\n"));
    assert!(render_obj(&[mk("a", 0.0, 0.0, 1.0, 1.0, 1.0)]).starts_with("# tokmd code city\n"));
}

#[test]
fn obj_vertices_and_faces_scale_linearly() {
    for n in 1..=5 {
        let buildings: Vec<_> = (0..n)
            .map(|i| mk(&format!("b{i}"), i as f32 * 2.0, 0.0, 1.0, 1.0, 1.0))
            .collect();
        let out = render_obj(&buildings);
        assert_eq!(vertex_count(&out), n * 8, "expected {n}*8 vertices");
        assert_eq!(face_count(&out), n * 6, "expected {n}*6 faces");
    }
}

#[test]
fn obj_face_indices_reference_valid_vertices() {
    let out = render_obj(&[
        mk("v", 0.0, 0.0, 1.0, 1.0, 1.0),
        mk("w", 3.0, 0.0, 1.0, 1.0, 1.0),
    ]);
    let vcount = vertex_count(&out);
    for line in out.lines().filter(|l| l.starts_with("f ")) {
        for tok in line.split_whitespace().skip(1) {
            let idx: usize = tok.parse().expect("face index must be numeric");
            assert!(
                idx >= 1 && idx <= vcount,
                "face index {idx} out of range 1..{vcount}"
            );
        }
    }
}

#[test]
fn obj_each_face_has_four_indices() {
    let out = render_obj(&[mk("q", 0.0, 0.0, 1.0, 1.0, 1.0)]);
    for line in out.lines().filter(|l| l.starts_with("f ")) {
        let parts: Vec<_> = line.split_whitespace().skip(1).collect();
        assert_eq!(parts.len(), 4, "each face must be a quad: {line}");
    }
}

// =========================================================================
// 3. OBJ name sanitization
// =========================================================================

#[test]
fn obj_names_sanitized_for_special_chars() {
    let out = render_obj(&[mk("src/main.rs", 0.0, 0.0, 1.0, 1.0, 1.0)]);
    let names = object_names(&out);
    assert_eq!(names, vec!["src_main_rs"]);
}

#[test]
fn obj_names_with_spaces_and_dashes() {
    let out = render_obj(&[mk("my file-name (copy).txt", 0.0, 0.0, 1.0, 1.0, 1.0)]);
    let names = object_names(&out);
    assert_eq!(names, vec!["my_file_name__copy__txt"]);
}

#[test]
fn obj_name_ordering_preserved() {
    let buildings = vec![
        mk("z_last", 0.0, 0.0, 1.0, 1.0, 1.0),
        mk("a_first", 2.0, 0.0, 1.0, 1.0, 1.0),
    ];
    let out = render_obj(&buildings);
    let names = object_names(&out);
    assert_eq!(
        names,
        vec!["z_last", "a_first"],
        "input order must be preserved"
    );
}

// =========================================================================
// 4. OBJ edge cases: zero-line / single-language style inputs
// =========================================================================

#[test]
fn obj_zero_height_building() {
    let out = render_obj(&[mk("empty_module", 0.0, 0.0, 1.0, 1.0, 0.0)]);
    assert_eq!(vertex_count(&out), 8);
    assert_eq!(face_count(&out), 6);
}

#[test]
fn obj_all_zero_dimensions() {
    let out = render_obj(&[mk("void", 0.0, 0.0, 0.0, 0.0, 0.0)]);
    assert_eq!(vertex_count(&out), 8);
    for line in out.lines().filter(|l| l.starts_with("v ")) {
        assert_eq!(line, "v 0 0 0");
    }
}

#[test]
fn obj_negative_coords_allowed() {
    let out = render_obj(&[mk("neg", -1.0, -2.0, 1.0, 1.0, 1.0)]);
    assert!(
        out.contains("v -1 -2 0"),
        "negative base coords should appear"
    );
}

#[test]
fn obj_single_building_eco_label_proxy() {
    let out = render_obj(&[mk("Rust", 0.0, 0.0, 10.0, 10.0, 500.0)]);
    assert_eq!(object_names(&out), vec!["Rust"]);
    assert_eq!(vertex_count(&out), 8);
}

// =========================================================================
// 5. MIDI determinism
// =========================================================================

#[test]
fn midi_deterministic_across_calls() {
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
    let a = render_midi(&notes, 120).unwrap();
    let b = render_midi(&notes, 120).unwrap();
    assert_eq!(a, b);
}

// =========================================================================
// 6. MIDI edge cases
// =========================================================================

#[test]
fn midi_empty_notes_produces_valid_header() {
    let bytes = render_midi(&[], 120).unwrap();
    assert_eq!(&bytes[..4], b"MThd");
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
    let bytes = render_midi(&notes, 0).unwrap();
    assert_eq!(&bytes[..4], b"MThd");
}

#[test]
fn midi_max_channel_clamped() {
    let notes = vec![MidiNote {
        key: 60,
        velocity: 100,
        start: 0,
        duration: 480,
        channel: 255,
    }];
    let bytes = render_midi(&notes, 120).unwrap();
    assert!(!bytes.is_empty());
}

#[test]
fn midi_simultaneous_notes() {
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
    let bytes = render_midi(&notes, 120).unwrap();
    assert_eq!(&bytes[..4], b"MThd");
    assert!(
        bytes.len() > 14,
        "chord should produce substantial MIDI data"
    );
}

#[test]
fn midi_high_tempo() {
    let notes = vec![MidiNote {
        key: 72,
        velocity: 90,
        start: 0,
        duration: 240,
        channel: 0,
    }];
    let bytes = render_midi(&notes, u16::MAX).unwrap();
    assert_eq!(&bytes[..4], b"MThd");
}

#[test]
fn midi_overlapping_notes_sorted_correctly() {
    let notes = vec![
        MidiNote {
            key: 60,
            velocity: 100,
            start: 0,
            duration: 960,
            channel: 0,
        },
        MidiNote {
            key: 72,
            velocity: 80,
            start: 480,
            duration: 480,
            channel: 1,
        },
    ];
    let bytes = render_midi(&notes, 120).unwrap();
    assert_eq!(&bytes[..4], b"MThd");
    assert!(bytes.len() > 14);
}
