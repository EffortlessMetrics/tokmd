import re

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    # The clippy error is about using `write!(out, "...\n", ...)`
    # It suggests using `writeln!(out, "...", ...)`

    # Use a regex to find `write!(out, "something\n", args)` and replace with `writeln!`

    # We want to match:
    # let _ = write!(out, "something\n", args);
    # and replace with:
    # let _ = writeln!(out, "something", args);

    # Also `write!(out, "something\n\n", args)` -> `writeln!(out, "something\n", args)`

    # It's safer to just replace `let _ = write!(out, ` and do string processing

    lines = content.split('\n')
    new_lines = []

    for i in range(len(lines)):
        line = lines[i]

        # Regex to find: write!(out, "([^"]*)\\n",
        # Actually clippy catches:
        # let _ = write!(out, "{}{} (lines: {}, tokens: {})\n", indent, name, node.lines, node.tokens);

        # We can just look for the literal string ending in \n"

        if 'write!(out,' in line and '\\n",' in line:
            line = line.replace('write!(out,', 'writeln!(out,')
            line = line.replace('\\n",', '",', 1)

        elif 'write!(out,' in line and '\\n\\n",' in line:
            line = line.replace('write!(out,', 'writeln!(out,')
            line = line.replace('\\n\\n",', '\\n",', 1)

        # specifically fix the multi-line ones shown in the error:
        # crates/tokmd-export-tree/src/lib.rs:31:17
        if '"{}{} (lines: {}, tokens: {})\\n",' in line:
            line = line.replace('write!(out,', 'writeln!(out,')
            line = line.replace('"{}{} (lines: {}, tokens: {})\\n",', '"{}{} (lines: {}, tokens: {})",')

        if '"{}{} (files: {}, lines: {}, tokens: {})\\n",' in line:
            line = line.replace('write!(out,', 'writeln!(out,')
            line = line.replace('"{}{} (files: {}, lines: {}, tokens: {})\\n",', '"{}{} (files: {}, lines: {}, tokens: {})",')

        new_lines.append(line)

    with open(filepath, 'w') as f:
        f.write('\n'.join(new_lines))

# We'll run it on the files that failed formatting and clippy
process_file('crates/tokmd-export-tree/src/lib.rs')
