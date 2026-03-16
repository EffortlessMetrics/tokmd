import re

with open("crates/tokmd-core/src/lib.rs", "r") as f:
    content = f.read()

content = content.replace(
    '    use std::fs;',
    '    #[cfg(feature = "analysis")]\n    use std::fs;'
)

content = content.replace(
    '    use std::path::{Path, PathBuf};',
    '    #[cfg(feature = "analysis")]\n    use std::path::{Path, PathBuf};'
)

content = content.replace(
    '    use std::time::{SystemTime, UNIX_EPOCH};',
    '    use std::time::{SystemTime, UNIX_EPOCH};'
)

content = content.replace(
    '    use crate::settings::AnalyzeSettings;',
    '    #[cfg(feature = "analysis")]\n    use crate::settings::AnalyzeSettings;'
)

content = content.replace(
    '    struct TempDirGuard(PathBuf);',
    '    #[cfg(feature = "analysis")]\n    struct TempDirGuard(PathBuf);'
)

content = content.replace(
    '    impl Drop for TempDirGuard {',
    '    #[cfg(feature = "analysis")]\n    impl Drop for TempDirGuard {'
)

content = content.replace(
    '    fn mk_temp_dir(prefix: &str) -> PathBuf {',
    '    #[cfg(feature = "analysis")]\n    fn mk_temp_dir(prefix: &str) -> PathBuf {'
)

content = content.replace(
    '    fn write_file(path: &Path, contents: &str) {',
    '    #[cfg(feature = "analysis")]\n    fn write_file(path: &Path, contents: &str) {'
)


with open("crates/tokmd-core/src/lib.rs", "w") as f:
    f.write(content)
