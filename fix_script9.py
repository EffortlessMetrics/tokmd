with open("crates/tokmd/src/commands/check_ignore.rs", "r") as f:
    content = f.read()

import re
# Look for consecutive #[test] tags
content = re.sub(r'#\[test\]\s*#\[test\]', '#[test]', content)

with open("crates/tokmd/src/commands/check_ignore.rs", "w") as f:
    f.write(content)
