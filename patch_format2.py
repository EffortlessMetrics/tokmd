with open("crates/tokmd-format/tests/edge_w76.rs", "r") as f:
    content = f.read()

old = "avg_lines: if files > 0 { lines / files } else { 0 },"
new = "avg_lines: lines.checked_div(files).unwrap_or(0),"

content = content.replace(old, new)
with open("crates/tokmd-format/tests/edge_w76.rs", "w") as f:
    f.write(content)
