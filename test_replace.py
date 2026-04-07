import re

def repl(m):
    obj = m.group(1)
    fmt_str = m.group(2)
    args = m.group(3)
    # add \n if the original was push_str(&format!("...", ...)) wait, push_str does not add newline.
    # so we can use write!
    if args:
        return f"let _ = write!({obj}, {fmt_str}, {args});"
    else:
        return f"let _ = write!({obj}, {fmt_str});"

with open("crates/tokmd-analysis-format/src/lib.rs", "r") as f:
    text = f.read()

# Replace out.push_str(&format!("...", args));
text = re.sub(r'([a-zA-Z0-9_]+)\.push_str\(&format!\(([^,]+)(?:,\s*(.*?))?\)\);', repl, text)

with open("crates/tokmd-analysis-format/src/lib.rs", "w") as f:
    f.write(text)
