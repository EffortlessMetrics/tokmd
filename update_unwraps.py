import os
import re

def update_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # We need to replace .unwrap() with .expect("Static regex must compile") or similar
    # In tokmd-core/src/lib.rs
    if 'tokmd-core/src/lib.rs' in filepath:
        content = re.sub(
            r'parse_analysis_preset\((.*?)\)\.unwrap\(\)',
            r'parse_analysis_preset(\1).expect("Test inputs to parse_analysis_preset must be statically valid presets")',
            content
        )
        content = re.sub(
            r'fs::create_dir_all\((.*?)\)\.unwrap\(\)',
            r'fs::create_dir_all(\1).expect("Test directory creation must succeed")',
            content
        )
        content = re.sub(
            r'fs::write\((.*?)\)\.unwrap\(\)',
            r'fs::write(\1).expect("Test file write must succeed")',
            content
        )
        # Handle the ffi doctest unwrap, although it's a doctest, the memory rule doesn't exclude them
        content = re.sub(
            r'serde_json::from_str\(&result\)\.unwrap\(\)',
            r'serde_json::from_str(&result).expect("JSON response from FFI run_json must be well-formed")',
            content
        )

    # In tokmd-core/src/ffi.rs
    if 'tokmd-core/src/ffi.rs' in filepath:
        content = re.sub(
            r'serde_json::from_str\(&result\)\.unwrap\(\)',
            r'serde_json::from_str(&result).expect("JSON response from FFI run_json must be well-formed")',
            content
        )

    # In tokmd-config/src/lib.rs
    if 'tokmd-config/src/lib.rs' in filepath:
        content = re.sub(
            r'serde_json::to_string\(&(.*?)\)\.unwrap\(\)',
            r'serde_json::to_string(&\1).expect("\1 serialization cannot fail as it contains no complex nested state")',
            content
        )
        content = re.sub(
            r'serde_json::from_str\(&json\)\.unwrap\(\)',
            r'serde_json::from_str(&json).expect("Deserialization of just-serialized enum must succeed")',
            content
        )

    with open(filepath, 'w') as f:
        f.write(content)

update_file('crates/tokmd-core/src/lib.rs')
update_file('crates/tokmd-core/src/ffi.rs')
update_file('crates/tokmd-config/src/lib.rs')
