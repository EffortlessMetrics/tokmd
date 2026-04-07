import re

with open("crates/tokmd-analysis-format/src/lib.rs", "r") as f:
    text = f.read()

def repl(m):
    obj = m.group(1)
    fmt_str = m.group(2)
    args = m.group(3)
    if args:
        return f"let _ = write!({obj}, {fmt_str}, {args});"
    else:
        return f"let _ = write!({obj}, {fmt_str});"

new_text = re.sub(r'([a-zA-Z0-9_]+)\.push_str\(&format!\(([^,]+)(?:,\s*(.*?))?\)\);', repl, text)

# Inject `use std::fmt::Write;` ONLY in functions that use `write!` and don't already have it
import ast
# We can just do string replacements for known functions if we want.
# Actually, let's just use `std::fmt::Write` globally if there are no `std::io::Write` traits used.
