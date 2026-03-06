import sys
import re

def process(filepath):
    with open(filepath, "r") as f:
        content = f.read()

    # Find `let _ = write!(out, "...\n", ...);`
    # Replace with `let _ = writeln!(out, "...", ...);`
    # Warning: Only when it ends with `\n"`

    res = ""
    idx = 0
    while idx < len(content):
        start = content.find("let _ = write!(", idx)
        if start == -1:
            res += content[idx:]
            break

        res += content[idx:start]

        # Find parenthesis block
        pos = start + len("let _ = write!(")
        paren_count = 1
        in_str = False
        escape = False

        block = ""
        while pos < len(content) and paren_count > 0:
            c = content[pos]
            block += c

            if escape:
                escape = False
            elif c == '\\':
                escape = True
            elif c == '"':
                in_str = not in_str
            elif not in_str:
                if c == '(':
                    paren_count += 1
                elif c == ')':
                    paren_count -= 1

            pos += 1

        if paren_count == 0:
            # check if block has \n"
            # It's basically the args of write!
            args = block[:-1] # without closing ')'

            # Use regex to find `\n",` or `\n"` at the end of the format string
            # This is a bit tricky, let's just find the first comma
            first_comma = args.find(',')
            if first_comma != -1:
                out_arg = args[:first_comma]
                rest_args = args[first_comma+1:]

                # find the format string
                fmt_start = rest_args.find('"')
                if fmt_start != -1:
                    fmt_end = rest_args.rfind('"')
                    if fmt_end != -1 and fmt_end > fmt_start:
                        fmt_str = rest_args[fmt_start:fmt_end+1]
                        if fmt_str.endswith('\\n"'):
                            new_fmt = fmt_str[:-3] + '"'
                            new_rest = rest_args[:fmt_start] + new_fmt + rest_args[fmt_end+1:]
                            res += f"let _ = writeln!({out_arg},{new_rest});"
                            idx = pos
                            continue

            # Fallback
            res += "let _ = write!(" + block
            idx = pos
        else:
            res += "let _ = write!("
            idx = start + len("let _ = write!(")

    with open(filepath, "w") as f:
        f.write(res)

process("crates/tokmd/src/commands/context.rs")
process("crates/tokmd-analysis-format/src/lib.rs")
