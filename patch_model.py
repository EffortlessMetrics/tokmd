with open("crates/tokmd-model/tests/determinism_w66.rs", "r") as f:
    content = f.read()

old = "avg_lines: if files > 0 { lines / files } else { 0 },"
new = "avg_lines: lines.checked_div(files).unwrap_or(0),"

content = content.replace(old, new)
with open("crates/tokmd-model/tests/determinism_w66.rs", "w") as f:
    f.write(content)
