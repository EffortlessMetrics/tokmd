import re
import os
import glob

def process_file(filepath):
    with open(filepath, "r") as f:
        text = f.read()

    def replace_push_str_format(text):
        out = []
        i = 0
        changed = False
        while i < len(text):
            match = re.search(r'([a-zA-Z0-9_]+)\.push_str\(&format!\(', text[i:])
            if not match:
                out.append(text[i:])
                break

            changed = True
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
            if text[curr:curr+2] != ");":
                # rollback if not matching
                print(f"Failed to match )); in {filepath}")
                out.append(text[start_idx:curr])
                i = curr
                continue
            curr += 2

            out.append(f"let _ = write!({obj}, {format_content});")
            i = curr

        return "".join(out), changed

    new_text, changed = replace_push_str_format(text)
    if changed:
        if "use std::fmt::Write;" not in new_text:
            new_text = new_text.replace("use anyhow::Result;", "use anyhow::Result;\nuse std::fmt::Write;")
        with open(filepath, "w") as f:
            f.write(new_text)

files = glob.glob("crates/**/src/**/*.rs", recursive=True) + glob.glob("crates/**/tests/**/*.rs", recursive=True)
for f in files:
    process_file(f)
