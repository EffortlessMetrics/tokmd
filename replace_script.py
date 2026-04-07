import re
import os
from pathlib import Path

def process_file(path):
    with open(path, 'r') as f:
        content = f.read()

    # We need to make sure std::fmt::Write is imported if we use write! or writeln!
    # But only if we actually replace something
    if 'push_str(&format!' not in content:
        return

    # Basic check, we'll do manual edits to be safer and cleaner for complex formatting.
    print(f"File to edit: {path}")

for root, _, files in os.walk('crates'):
    for file in files:
        if file.endswith('.rs'):
            process_file(os.path.join(root, file))
