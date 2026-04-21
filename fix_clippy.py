import re

with open("crates/tokmd-python/src/lib.rs", "r") as f:
    content = f.read()

# Fix single_match instances (ignoring errors in tests)
content = content.replace("""            match result {
                Ok(_) => (),  // Handled gracefully
                Err(_) => (), // Error is also fine
            }""", """            let _ = result;""")

content = content.replace("""            assert!(true, "Permission error contract documented");""", """""")

content = content.replace("""            match lang(
                py,
                Some(vec![temp_path.clone()]),
                0,
                false,
                None,
                None,
                None,
                false,
            ) {
                Ok(_) => (),
                Err(_) => (),
            }""", """            let _ = lang(
                py,
                Some(vec![temp_path.clone()]),
                0,
                false,
                None,
                None,
                None,
                false,
            );""")

content = content.replace("""            match module(
                py,
                Some(vec![temp_path.clone()]),
                0,
                None,
                1,
                None,
                None,
                None,
                false,
            ) {
                Ok(_) => (),
                Err(_) => (),
            }""", """            let _ = module(
                py,
                Some(vec![temp_path.clone()]),
                0,
                None,
                1,
                None,
                None,
                None,
                false,
            );""")

content = content.replace("""            match export(
                py,
                Some(vec![temp_path.clone()]),
                None,
                0,
                0,
                None,
                2,
                None,
                None,
                None,
                false,
            ) {
                Ok(_) => (),
                Err(_) => (),
            }""", """            let _ = export(
                py,
                Some(vec![temp_path.clone()]),
                None,
                0,
                0,
                None,
                2,
                None,
                None,
                None,
                false,
            );""")

content = content.replace("""            match analyze(
                py,
                Some(vec![temp_path.clone()]),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                false,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ) {
                Ok(_) => (),
                Err(_) => (),
            }""", """            let _ = analyze(
                py,
                Some(vec![temp_path.clone()]),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                false,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            );""")

content = content.replace("""            match diff(py, Some(&temp_path), Some(&temp_path)) {
                Ok(_) => (),
                Err(_) => (),
            }""", """            let _ = diff(py, Some(&temp_path), Some(&temp_path));""")

content = content.replace("""            match cockpit(py, None, None, None, None) {
                Ok(_) => (),
                Err(_) => (),
            }""", """            let _ = cockpit(py, None, None, None, None);""")

content = content.replace("""                match result {
                    Ok(_) => (),
                    Err(_) => (),
                }""", """                let _ = result;""")

content = content.replace("""            assert!(true, "GIL remained valid after run()");""", """""")

content = content.replace("""            match result {
                Ok(_) => (),
                Err(_) => (),
            }""", """            let _ = result;""")

with open("crates/tokmd-python/src/lib.rs", "w") as f:
    f.write(content)
