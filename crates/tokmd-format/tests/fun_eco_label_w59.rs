#![cfg(feature = "fun")]
//! Eco-label edge cases and OBJ rendering boundary conditions – wave 59.
//!
//! Covers: fractional coordinates, NaN/Inf graceful handling, very small
//! dimensions, building ordering stability, vertex winding invariants,
//! and structural format compliance.

use tokmd_format::fun::{ObjBuilding, render_obj};

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

fn vertex_lines(obj: &str) -> Vec<&str> {
    obj.lines().filter(|l| l.starts_with("v ")).collect()
}

fn face_lines(obj: &str) -> Vec<&str> {
    obj.lines().filter(|l| l.starts_with("f ")).collect()
}

fn object_names(obj: &str) -> Vec<&str> {
    obj.lines().filter_map(|l| l.strip_prefix("o ")).collect()
}

// =========================================================================
// Fractional / sub-pixel coordinates
// =========================================================================

#[test]
fn obj_fractional_coords_rendered_precisely() {
    let b = mk("frac", 0.125, 0.25, 0.5, 0.75, 1.5);
    let out = render_obj(std::slice::from_ref(&b));
    // x+w = 0.625, y+d = 1.0, z+h = 1.5
    assert!(out.contains("v 0.625 0.25 0"), "x+w vertex");
    assert!(out.contains("v 0.125 1 0"), "y+d vertex");
    assert!(out.contains("v 0.125 0.25 1.5"), "z+h vertex");
}

#[test]
fn obj_very_small_dimensions_not_lost() {
    let b = mk("tiny", 0.0, 0.0, 1e-6, 1e-6, 1e-6);
    let out = render_obj(std::slice::from_ref(&b));
    // All 8 vertices present even for microscopic building
    assert_eq!(vertex_lines(&out).len(), 8);
    assert_eq!(face_lines(&out).len(), 6);
}

#[test]
fn obj_negative_dimensions_do_not_panic() {
    let b = mk("negdim", 0.0, 0.0, -1.0, -2.0, -3.0);
    let out = render_obj(std::slice::from_ref(&b));
    assert_eq!(vertex_lines(&out).len(), 8);
    assert_eq!(face_lines(&out).len(), 6);
}

// =========================================================================
// NaN / Infinity (graceful — should not panic)
// =========================================================================

#[test]
fn obj_nan_coords_produce_valid_structure() {
    let b = mk("nan", f32::NAN, 0.0, 1.0, 1.0, 1.0);
    let out = render_obj(std::slice::from_ref(&b));
    // Structure is intact even if values are NaN
    assert_eq!(vertex_lines(&out).len(), 8);
    assert_eq!(face_lines(&out).len(), 6);
}

#[test]
fn obj_infinity_coords_produce_valid_structure() {
    let b = mk("inf", f32::INFINITY, f32::NEG_INFINITY, 1.0, 1.0, 1.0);
    let out = render_obj(std::slice::from_ref(&b));
    assert_eq!(vertex_lines(&out).len(), 8);
    assert_eq!(face_lines(&out).len(), 6);
}

#[test]
fn obj_nan_height_does_not_crash() {
    let b = mk("nanh", 0.0, 0.0, 1.0, 1.0, f32::NAN);
    let out = render_obj(std::slice::from_ref(&b));
    assert!(out.contains("o nanh"));
}

// =========================================================================
// Large building count (stress)
// =========================================================================

#[test]
fn obj_100_buildings_correct_counts() {
    let buildings: Vec<ObjBuilding> = (0..100)
        .map(|i| mk(&format!("b{i}"), i as f32, 0.0, 1.0, 1.0, 1.0))
        .collect();
    let out = render_obj(&buildings);
    assert_eq!(vertex_lines(&out).len(), 800);
    assert_eq!(face_lines(&out).len(), 600);
    assert_eq!(object_names(&out).len(), 100);
}

#[test]
fn obj_last_face_indices_for_many_buildings() {
    let n = 50;
    let buildings: Vec<ObjBuilding> = (0..n)
        .map(|i| mk(&format!("m{i}"), i as f32, 0.0, 1.0, 1.0, 1.0))
        .collect();
    let out = render_obj(&buildings);
    // Last building starts at vertex (n-1)*8+1 = 393
    let base = (n - 1) * 8 + 1;
    let expected_first_face = format!("f {} {} {} {}", base, base + 1, base + 2, base + 3);
    assert!(
        out.contains(&expected_first_face),
        "expected {expected_first_face}"
    );
}

// =========================================================================
// Name edge cases
// =========================================================================

#[test]
fn obj_numeric_only_name_preserved() {
    let b = mk("12345", 0.0, 0.0, 1.0, 1.0, 1.0);
    let out = render_obj(std::slice::from_ref(&b));
    assert!(out.contains("o 12345\n"));
}

#[test]
fn obj_single_char_name() {
    let b = mk("x", 0.0, 0.0, 1.0, 1.0, 1.0);
    let out = render_obj(std::slice::from_ref(&b));
    assert!(out.contains("o x\n"));
}

#[test]
fn obj_very_long_name_not_truncated() {
    let long_name = "a".repeat(1000);
    let b = mk(&long_name, 0.0, 0.0, 1.0, 1.0, 1.0);
    let out = render_obj(std::slice::from_ref(&b));
    let obj_name = object_names(&out)[0];
    assert_eq!(obj_name.len(), 1000);
}

#[test]
fn obj_mixed_unicode_and_ascii_sanitized() {
    let b = mk("café_42", 0.0, 0.0, 1.0, 1.0, 1.0);
    let out = render_obj(std::slice::from_ref(&b));
    // 'c' ok, 'a' ok, 'f' ok, 'é' -> '_', '_' -> '_', '4' ok, '2' ok
    assert!(out.contains("o caf__42\n"));
}

#[test]
fn obj_path_separators_sanitized() {
    let b = mk("src\\main.rs", 0.0, 0.0, 1.0, 1.0, 1.0);
    let out = render_obj(std::slice::from_ref(&b));
    assert!(out.contains("o src_main_rs\n"));
}

// =========================================================================
// Duplicate names (allowed — OBJ tolerates them)
// =========================================================================

#[test]
fn obj_duplicate_names_produce_duplicate_objects() {
    let buildings = vec![
        mk("dup", 0.0, 0.0, 1.0, 1.0, 1.0),
        mk("dup", 2.0, 0.0, 1.0, 1.0, 1.0),
    ];
    let out = render_obj(&buildings);
    let names = object_names(&out);
    assert_eq!(names, vec!["dup", "dup"]);
    assert_eq!(vertex_lines(&out).len(), 16);
}

// =========================================================================
// Output formatting invariants
// =========================================================================

#[test]
fn obj_no_trailing_whitespace_on_vertex_lines() {
    let b = mk("ws", 1.0, 2.0, 3.0, 4.0, 5.0);
    let out = render_obj(std::slice::from_ref(&b));
    for line in vertex_lines(&out) {
        assert_eq!(line, line.trim_end(), "vertex line has trailing whitespace");
    }
}

#[test]
fn obj_no_trailing_whitespace_on_face_lines() {
    let b = mk("ws", 1.0, 2.0, 3.0, 4.0, 5.0);
    let out = render_obj(std::slice::from_ref(&b));
    for line in face_lines(&out) {
        assert_eq!(line, line.trim_end(), "face line has trailing whitespace");
    }
}

#[test]
fn obj_every_line_type_is_known() {
    let buildings = vec![
        mk("a", 0.0, 0.0, 1.0, 1.0, 1.0),
        mk("b", 2.0, 0.0, 1.0, 1.0, 1.0),
    ];
    let out = render_obj(&buildings);
    for line in out.lines() {
        assert!(
            line.starts_with("# ")
                || line.starts_with("o ")
                || line.starts_with("v ")
                || line.starts_with("f "),
            "unexpected line type: {line:?}"
        );
    }
}

#[test]
fn obj_faces_always_have_four_indices() {
    let buildings: Vec<ObjBuilding> = (0..5)
        .map(|i| mk(&format!("q{i}"), i as f32 * 2.0, 0.0, 1.0, 1.0, 1.0))
        .collect();
    let out = render_obj(&buildings);
    for line in face_lines(&out) {
        let indices: Vec<&str> = line
            .strip_prefix("f ")
            .unwrap()
            .split_whitespace()
            .collect();
        assert_eq!(indices.len(), 4, "face must be a quad: {line:?}");
    }
}

// =========================================================================
// Vertex geometry: bottom vs top plane
// =========================================================================

#[test]
fn obj_bottom_vertices_have_z_zero() {
    let b = mk("box", 1.0, 2.0, 3.0, 4.0, 5.0);
    let out = render_obj(std::slice::from_ref(&b));
    let verts = vertex_lines(&out);
    // First 4 vertices are the bottom plane (z=0)
    for v in &verts[..4] {
        let z: f32 = v.split_whitespace().nth(3).unwrap().parse().unwrap();
        assert_eq!(z, 0.0, "bottom vertex z must be 0");
    }
}

#[test]
fn obj_top_vertices_have_z_equal_to_height() {
    let b = mk("box", 1.0, 2.0, 3.0, 4.0, 7.5);
    let out = render_obj(std::slice::from_ref(&b));
    let verts = vertex_lines(&out);
    // Last 4 vertices are the top plane (z=h)
    for v in &verts[4..8] {
        let z: f32 = v.split_whitespace().nth(3).unwrap().parse().unwrap();
        assert_eq!(z, 7.5, "top vertex z must equal height");
    }
}

// =========================================================================
// Determinism with varied ordering
// =========================================================================

#[test]
fn obj_deterministic_across_10_runs() {
    let b = mk("det", 3.15, 2.72, 1.42, 1.74, 2.24);
    let first = render_obj(std::slice::from_ref(&b));
    for _ in 0..10 {
        assert_eq!(
            render_obj(std::slice::from_ref(&b)),
            first,
            "OBJ output must be deterministic"
        );
    }
}

#[test]
fn obj_input_order_matches_output_order() {
    let names = ["z", "m", "a", "q", "b"];
    let buildings: Vec<ObjBuilding> = names
        .iter()
        .enumerate()
        .map(|(i, &n)| mk(n, i as f32, 0.0, 1.0, 1.0, 1.0))
        .collect();
    let out = render_obj(&buildings);
    assert_eq!(object_names(&out), names);
}
