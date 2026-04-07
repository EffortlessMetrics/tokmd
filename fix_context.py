path = 'crates/tokmd/src/commands/context.rs'

with open(path, 'r') as f:
    content = f.read()

import re

# Need to change push_str(&format!(...)) into write!(out, ...)
# BUT import std::fmt::Write as _; to avoid Write trait collision.

# The original has things like out.push_str(&format!("Budget: {} tokens\n", budget));
pattern1 = re.compile(r'([a-zA-Z0-9_]+)\.push_str\(&format!\(\s*"([^"]*)\\n"\s*(?:,\s*([^)]*))?\)\);', re.DOTALL)
def repl_writeln(m):
    var_name = m.group(1)
    fmt_str = m.group(2)
    args = m.group(3)
    if args:
        return f'let _ = write!({var_name}, "{fmt_str}\\n", {args});'
    else:
        return f'let _ = write!({var_name}, "{fmt_str}\\n");'
content = pattern1.sub(repl_writeln, content)

pattern2 = re.compile(r'([a-zA-Z0-9_]+)\.push_str\(&format!\(\s*"([^"]*)"\s*(?:,\s*([^)]*))?\)\);', re.DOTALL)
def repl_write(m):
    var_name = m.group(1)
    fmt_str = m.group(2)
    args = m.group(3)
    if args:
        return f'let _ = write!({var_name}, "{fmt_str}", {args});'
    else:
        return f'let _ = write!({var_name}, "{fmt_str}");'
content = pattern2.sub(repl_write, content)

content = content.replace('use std::io::{self, BufWriter, Write};', 'use std::io::{self, BufWriter, Write};\nuse std::fmt::Write as _;')

with open(path, 'w') as f:
    f.write(content)
