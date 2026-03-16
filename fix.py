with open("crates/tokmd-core/src/lib.rs", "r") as f:
    content = f.read()

# Replace `use crate::settings::AnalyzeSettings;` -> `#[cfg(feature = "analysis")]\n    use crate::settings::AnalyzeSettings;`
content = content.replace("    use crate::settings::AnalyzeSettings;", '    #[cfg(feature = "analysis")]\n    use crate::settings::AnalyzeSettings;')

# Replace `struct TempDirGuard` -> `#[cfg(feature = "analysis")]\n    struct TempDirGuard`
content = content.replace("    struct TempDirGuard(PathBuf);", '    #[cfg(feature = "analysis")]\n    struct TempDirGuard(PathBuf);')

# Replace `impl Drop for TempDirGuard`
content = content.replace("    impl Drop for TempDirGuard {", '    #[cfg(feature = "analysis")]\n    impl Drop for TempDirGuard {')

# Replace `fn mk_temp_dir`
content = content.replace("    fn mk_temp_dir(prefix: &str) -> PathBuf {", '    #[cfg(feature = "analysis")]\n    fn mk_temp_dir(prefix: &str) -> PathBuf {')

# Replace `fn write_file`
content = content.replace("    fn write_file(path: &Path, contents: &str) {", '    #[cfg(feature = "analysis")]\n    fn write_file(path: &Path, contents: &str) {')

with open("crates/tokmd-core/src/lib.rs", "w") as f:
    f.write(content)
with open("crates/tokmd-core/src/lib.rs", "r") as f:
    content = f.read()

content = content.replace("    use std::fs;", '    #[cfg(feature = "analysis")]\n    use std::fs;')
content = content.replace("    use std::path::{Path, PathBuf};", '    #[cfg(feature = "analysis")]\n    use std::path::{Path, PathBuf};')
content = content.replace("    use std::time::{SystemTime, UNIX_EPOCH};", '    #[cfg(feature = "analysis")]\n    use std::time::{SystemTime, UNIX_EPOCH};')

with open("crates/tokmd-core/src/lib.rs", "w") as f:
    f.write(content)
