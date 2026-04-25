with open("crates/tokmd-format/tests/format_snapshot_w58.rs", "r") as f:
    content = f.read()

old = """            if total_files > 0 {
                total_lines / total_files
            } else {
                0
            }"""

new = """            total_lines.checked_div(total_files).unwrap_or(0)"""

content = content.replace(old, new)
with open("crates/tokmd-format/tests/format_snapshot_w58.rs", "w") as f:
    f.write(content)
