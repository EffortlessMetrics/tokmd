//! OBJ code-city rendering for fun analysis outputs.

#[derive(Debug, Clone)]
pub struct ObjBuilding {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub d: f32,
    pub h: f32,
}

pub fn render_obj(buildings: &[ObjBuilding]) -> String {
    let mut out = String::new();
    out.push_str("# tokmd code city\n");
    let mut vertex_index = 1usize;

    for b in buildings {
        out.push_str(&format!("o {}\n", sanitize_name(&b.name)));
        let (x, y, z) = (b.x, b.y, 0.0f32);
        let (w, d, h) = (b.w, b.d, b.h);

        let v = [
            (x, y, z),
            (x + w, y, z),
            (x + w, y + d, z),
            (x, y + d, z),
            (x, y, z + h),
            (x + w, y, z + h),
            (x + w, y + d, z + h),
            (x, y + d, z + h),
        ];
        for (vx, vy, vz) in v {
            out.push_str(&format!("v {} {} {}\n", vx, vy, vz));
        }

        let faces = [
            [1, 2, 3, 4],
            [5, 6, 7, 8],
            [1, 2, 6, 5],
            [2, 3, 7, 6],
            [3, 4, 8, 7],
            [4, 1, 5, 8],
        ];
        for face in faces {
            out.push_str(&format!(
                "f {} {} {} {}\n",
                vertex_index + face[0] - 1,
                vertex_index + face[1] - 1,
                vertex_index + face[2] - 1,
                vertex_index + face[3] - 1,
            ));
        }

        vertex_index += 8;
    }

    out
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_name_replaces_non_alphanumeric() {
        assert_eq!(sanitize_name("hello world"), "hello_world");
        assert_eq!(sanitize_name("src/main.rs"), "src_main_rs");
        assert_eq!(sanitize_name("foo-bar_baz"), "foo_bar_baz");
    }

    #[test]
    fn sanitize_name_preserves_alphanumeric() {
        assert_eq!(sanitize_name("abc123"), "abc123");
    }

    #[test]
    fn sanitize_name_empty() {
        assert_eq!(sanitize_name(""), "");
    }

    #[test]
    fn render_obj_empty_input() {
        let result = render_obj(&[]);
        assert_eq!(result, "# tokmd code city\n");
    }

    #[test]
    fn render_obj_single_building() {
        let buildings = vec![ObjBuilding {
            name: "main".into(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 2.0,
        }];
        let result = render_obj(&buildings);
        assert!(result.starts_with("# tokmd code city\n"));
        assert!(result.contains("o main\n"));
        assert_eq!(result.matches("\nv ").count(), 8);
        assert_eq!(result.matches("\nf ").count(), 6);
    }

    #[test]
    fn render_obj_multiple_buildings() {
        let buildings = vec![
            ObjBuilding {
                name: "a".into(),
                x: 0.0,
                y: 0.0,
                w: 1.0,
                d: 1.0,
                h: 1.0,
            },
            ObjBuilding {
                name: "b".into(),
                x: 2.0,
                y: 0.0,
                w: 1.0,
                d: 1.0,
                h: 3.0,
            },
        ];
        let result = render_obj(&buildings);
        assert!(result.contains("o a\n"));
        assert!(result.contains("o b\n"));
        assert_eq!(result.matches("\nv ").count(), 16);
        assert_eq!(result.matches("\nf ").count(), 12);
    }

    #[test]
    fn render_obj_sanitizes_names() {
        let buildings = vec![ObjBuilding {
            name: "src/main.rs".into(),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            d: 1.0,
            h: 1.0,
        }];
        let result = render_obj(&buildings);
        assert!(result.contains("o src_main_rs\n"));
        assert!(!result.contains("o src/main.rs\n"));
    }
}
