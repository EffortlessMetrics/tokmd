import re

with open("crates/tokmd-analysis-format/src/lib.rs", "r") as f:
    text = f.read()

# Match `.push_str(&format!(` then anything inside the format! invocation, then `));`
# We can use a simple state machine or balance parentheses.
def replace_push_str_format(text):
    out = []
    i = 0
    while i < len(text):
        match = re.search(r'([a-zA-Z0-9_]+)\.push_str\(&format!\(', text[i:])
        if not match:
            out.append(text[i:])
            break

        start_idx = i + match.start()
        format_start_idx = i + match.end() - 1 # points to (
        out.append(text[i:start_idx])

        obj = match.group(1)

        # find matching )
        paren_count = 1
        curr = format_start_idx + 1
        while curr < len(text) and paren_count > 0:
            if text[curr] == '(':
                paren_count += 1
            elif text[curr] == ')':
                paren_count -= 1
            curr += 1

        format_content = text[format_start_idx+1 : curr-1]

        # The remainder should be `));`
        # curr points to `)` of `.push_str(...)`
        assert text[curr:curr+2] == ");"
        curr += 2

        out.append(f"let _ = write!({obj}, {format_content});")
        i = curr

    return "".join(out)

text = replace_push_str_format(text)

with open("crates/tokmd-analysis-format/src/lib.rs", "w") as f:
    f.write(text)
