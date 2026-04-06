import sys

filepath = "crates/tokmd/tests/run_diff.rs"
with open(filepath, 'r') as f:
    content = f.read()

# Remove individual #[cfg(feature = "git")]
content = content.replace('#[cfg(feature = "git")]\n', '')

# Insert #![cfg(feature = "git")] before or after the module docs
if "#![cfg(feature = \"git\")]" not in content:
    lines = content.split('\n')
    for i, line in enumerate(lines):
        if line.startswith('mod common;'):
            lines.insert(i, '#![cfg(feature = "git")]')
            break
    content = '\n'.join(lines)

with open(filepath, 'w') as f:
    f.write(content)
