import re

with open("crates/tokmd-analysis-format/src/lib.rs", "r") as f:
    lines = f.readlines()

new_lines = []
in_fn = False
fn_name = ""
has_write = False
fn_lines = []

for line in lines:
    if re.match(r'^(pub )?fn [a-zA-Z0-9_]+\(', line):
        in_fn = True
        has_write = False
        fn_lines = [line]
        continue

    if in_fn:
        fn_lines.append(line)
        if "out.push_str(&format!(" in line:
            has_write = True
        if line.startswith("}"):
            in_fn = False
            # process fn_lines
            if has_write:
                # insert use std::fmt::Write;
                # find first { and insert after it
                for i, fl in enumerate(fn_lines):
                    if "{" in fl:
                        # only insert if not already there
                        if not any("use std::fmt::Write;" in x for x in fn_lines):
                            fn_lines.insert(i+1, "    use std::fmt::Write;\n")
                        break

            # replace format
            for i in range(len(fn_lines)):
                fn_lines[i] = re.sub(
                    r'([a-zA-Z0-9_]+)\.push_str\(&format!\(([^,]+)(?:,\s*(.*?))?\)\);',
                    lambda m: f"let _ = write!({m.group(1)}, {m.group(2)}" + (f", {m.group(3)}" if m.group(3) else "") + ");",
                    fn_lines[i]
                )
            new_lines.extend(fn_lines)
            fn_lines = []
    else:
        new_lines.append(line)

with open("crates/tokmd-analysis-format/src/lib.rs", "w") as f:
    f.writelines(new_lines)
