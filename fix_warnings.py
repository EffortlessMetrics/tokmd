import re

with open("crates/tokmd-core/src/lib.rs", "r") as f:
    content = f.read()

start_idx = content.find("mod tests {")
if start_idx != -1:
    end_idx = content.find("pub mod readme_doctests", start_idx)
    if end_idx != -1:
        tests_block = content[start_idx:end_idx]

        tests_block = tests_block.replace(
            "    #[allow(dead_code)]\n    #[derive(Debug)]\n    #[allow(dead_code)]\n    struct TempDirGuard(PathBuf);",
            "    #[allow(dead_code)]\n    #[derive(Debug)]\n    struct TempDirGuard(PathBuf);"
        )
        tests_block = tests_block.replace(
            "    #[allow(dead_code)]\n    #[allow(dead_code)]\n    #[derive(Debug)]\n    struct TempDirGuard(PathBuf);",
            "    #[allow(dead_code)]\n    #[derive(Debug)]\n    struct TempDirGuard(PathBuf);"
        )

        content = content[:start_idx] + tests_block + content[end_idx:]

with open("crates/tokmd-core/src/lib.rs", "w") as f:
    f.write(content)
