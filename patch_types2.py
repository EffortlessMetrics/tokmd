with open("crates/tokmd-types/tests/proptest_w69.rs", "r") as f:
    content = f.read()

old = "avg_lines: if total_files > 0 { total_lines / total_files } else { 0 },"
new = "avg_lines: total_lines.checked_div(total_files).unwrap_or(0),"

content = content.replace(old, new)
with open("crates/tokmd-types/tests/proptest_w69.rs", "w") as f:
    f.write(content)
