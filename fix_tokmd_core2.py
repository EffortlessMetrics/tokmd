import re

with open('crates/tokmd-core/tests/in_memory_w80.rs', 'r') as f:
    content = f.read()

# remove #[allow(dead_code)] we added earlier to fix formatting issue
content = re.sub(r'#\[allow\(dead_code\)\]\n', '', content)

# wrap the block in #[cfg(feature = "analysis")]
content = re.sub(
    r'(static CWD_LOCK:.*?fn with_current_dir<T>\(path: &Path, f: impl FnOnce\(\) -> T\) -> T \{.*?\n})',
    r'#[cfg(feature = "analysis")]\n\1',
    content,
    flags=re.DOTALL
)

with open('crates/tokmd-core/tests/in_memory_w80.rs', 'w') as f:
    f.write(content)
