//! W75 tests for tokmd-fun.
//!
//! Covers eco-label-style inputs, novelty output formats (OBJ/MIDI),
//! various input sizes, and determinism guarantees.

use tokmd_fun::{MidiNote, ObjBuilding, render_midi, render_obj};

// ── Helpers ─────────────────────────────────────────────────────────

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

fn note(key: u8, start: u32, duration: u32) -> MidiNote {
    MidiNote {
        key,
        velocity: 100,
        start,
        duration,
        channel: 0,
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

// ═══════════════════════════════════════════════════════════════════
// § 1. Eco-label style generation (OBJ buildings as language proxies)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn eco_label_single_language_building() {
    let buildings = vec![mk("Rust", 0.0, 0.0, 10.0, 10.0, 1000.0)];
    let out = render_obj(&buildings);
    assert!(out.starts_with("# tokmd code city\n"));
    assert_eq!(object_names(&out), vec!["Rust"]);
    assert_eq!(vertex_count(&out), 8);
    assert_eq!(face_count(&out), 6);
}

#[test]
fn eco_label_multi_language_buildings() {
    let buildings = vec![
        mk("Rust", 0.0, 0.0, 10.0, 10.0, 5000.0),
        mk("Python", 12.0, 0.0, 8.0, 8.0, 2000.0),
        mk("JavaScript", 22.0, 0.0, 6.0, 6.0, 800.0),
    ];
    let out = render_obj(&buildings);
    assert_eq!(object_names(&out), vec!["Rust", "Python", "JavaScript"]);
    assert_eq!(vertex_count(&out), 24);
    assert_eq!(face_count(&out), 18);
}

#[test]
fn eco_label_proportional_heights() {
    let small = mk("Small", 0.0, 0.0, 5.0, 5.0, 100.0);
    let large = mk("Large", 6.0, 0.0, 5.0, 5.0, 10000.0);
    let out = render_obj(&[small, large]);
    // Both produce valid geometry
    assert_eq!(vertex_count(&out), 16);
    assert!(out.contains("o Small\n"));
    assert!(out.contains("o Large\n"));
}

// ═══════════════════════════════════════════════════════════════════
// § 2. Novelty output format validation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn obj_format_starts_with_header_comment() {
    let out = render_obj(&[mk("test", 0.0, 0.0, 1.0, 1.0, 1.0)]);
    let first_line = out.lines().next().unwrap();
    assert_eq!(first_line, "# tokmd code city");
}

#[test]
fn obj_format_contains_object_vertex_face_lines() {
    let out = render_obj(&[mk("mod", 0.0, 0.0, 2.0, 3.0, 4.0)]);
    let has_o = out.lines().any(|l| l.starts_with("o "));
    let has_v = out.lines().any(|l| l.starts_with("v "));
    let has_f = out.lines().any(|l| l.starts_with("f "));
    assert!(has_o, "OBJ must contain object lines");
    assert!(has_v, "OBJ must contain vertex lines");
    assert!(has_f, "OBJ must contain face lines");
}

#[test]
fn midi_format_has_mthd_header() {
    let bytes = render_midi(&[note(60, 0, 480)], 120).unwrap();
    assert_eq!(&bytes[..4], b"MThd");
}

#[test]
fn midi_format_has_mtrk_chunk() {
    let bytes = render_midi(&[note(60, 0, 480)], 120).unwrap();
    // MIDI track chunk starts with MTrk after the 14-byte header
    let mtrk_pos = bytes
        .windows(4)
        .position(|w| w == b"MTrk")
        .expect("MTrk chunk must be present");
    assert!(mtrk_pos >= 14, "MTrk after header");
}

// ═══════════════════════════════════════════════════════════════════
// § 3. Various input sizes
// ═══════════════════════════════════════════════════════════════════

#[test]
fn obj_empty_input_produces_header_only() {
    let out = render_obj(&[]);
    assert_eq!(out, "# tokmd code city\n");
    assert_eq!(vertex_count(&out), 0);
    assert_eq!(face_count(&out), 0);
}

#[test]
fn obj_many_buildings_scales_correctly() {
    let n = 50;
    let buildings: Vec<_> = (0..n)
        .map(|i| mk(&format!("lang{i}"), i as f32 * 2.0, 0.0, 1.0, 1.0, (i + 1) as f32))
        .collect();
    let out = render_obj(&buildings);
    assert_eq!(vertex_count(&out), n * 8);
    assert_eq!(face_count(&out), n * 6);
    assert_eq!(object_names(&out).len(), n);
}

#[test]
fn midi_many_notes_produces_valid_output() {
    let notes: Vec<MidiNote> = (0..100)
        .map(|i| MidiNote {
            key: 40 + (i % 48) as u8,
            velocity: 80,
            start: i * 120,
            duration: 100,
            channel: (i % 16) as u8,
        })
        .collect();
    let bytes = render_midi(&notes, 120).unwrap();
    assert_eq!(&bytes[..4], b"MThd");
    assert!(bytes.len() > 100, "many notes should produce substantial data");
}

#[test]
fn midi_single_note_minimal_output() {
    let bytes = render_midi(&[note(60, 0, 480)], 120).unwrap();
    assert_eq!(&bytes[..4], b"MThd");
    assert!(bytes.len() > 14);
}

// ═══════════════════════════════════════════════════════════════════
// § 4. Determinism guarantees
// ═══════════════════════════════════════════════════════════════════

#[test]
fn obj_deterministic_same_input() {
    let buildings = vec![
        mk("alpha", 0.0, 0.0, 3.0, 3.0, 10.0),
        mk("beta", 5.0, 0.0, 2.0, 2.0, 20.0),
        mk("gamma", 9.0, 0.0, 4.0, 4.0, 5.0),
    ];
    let a = render_obj(&buildings);
    let b = render_obj(&buildings);
    assert_eq!(a, b, "OBJ output must be deterministic");
}

#[test]
fn midi_deterministic_same_input() {
    let notes = vec![note(60, 0, 480), note(64, 480, 480), note(67, 960, 480)];
    let a = render_midi(&notes, 120).unwrap();
    let b = render_midi(&notes, 120).unwrap();
    assert_eq!(a, b, "MIDI output must be deterministic");
}

#[test]
fn obj_deterministic_after_clone() {
    let buildings = vec![mk("x", 1.0, 2.0, 3.0, 4.0, 5.0)];
    let cloned = buildings.clone();
    assert_eq!(
        render_obj(&buildings),
        render_obj(&cloned),
        "cloned input must produce identical output"
    );
}

#[test]
fn midi_different_tempos_produce_different_output() {
    let notes = vec![note(60, 0, 480)];
    let slow = render_midi(&notes, 60).unwrap();
    let fast = render_midi(&notes, 240).unwrap();
    assert_ne!(slow, fast, "different tempos must produce different MIDI");
}
