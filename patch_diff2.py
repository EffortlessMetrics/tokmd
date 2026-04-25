with open("crates/tokmd/tests/diff_w71.rs", "r") as f:
    content = f.read()

old = """    let total_avg = if total_files > 0 {
        total_lines / total_files
    } else {
        0
    };"""

new = """    let total_avg = total_lines.checked_div(total_files).unwrap_or(0);"""

content = content.replace(old, new)
with open("crates/tokmd/tests/diff_w71.rs", "w") as f:
    f.write(content)
