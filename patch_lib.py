import re

with open("crates/tokmd-core/src/lib.rs", "r") as f:
    code = f.read()

# We need to make the tests compile AND remove dead code warnings.
# Notice how AnalyzeSettings is used in tests BUT only under `#[cfg(feature = "analysis")]`.
# Also TempDirGuard, mk_temp_dir, write_file are only used in `analyze_workflow_estimate_preset_populates_effort_and_size_basis_breakdown` which is under `#[cfg(feature = "analysis")]`.
# Wait! In the CI failure, it said `AnalyzeSettings is not found`.
# Let's just wrap TempDirGuard, mk_temp_dir, write_file in `#[cfg(feature = "analysis")]`.

# AND fix the AnalyzeSettings import. `use crate::settings::AnalyzeSettings;` should be `use crate::settings::AnalyzeSettings;` but maybe it's not imported correctly in tests.

code = code.replace(
    '    use crate::settings::AnalyzeSettings;',
    '    #[cfg(feature = "analysis")]\n    use crate::settings::AnalyzeSettings;'
)

code = code.replace(
    '    struct TempDirGuard(PathBuf);',
    '    #[cfg(feature = "analysis")]\n    struct TempDirGuard(PathBuf);'
)

code = code.replace(
    '    impl Drop for TempDirGuard {',
    '    #[cfg(feature = "analysis")]\n    impl Drop for TempDirGuard {'
)

code = code.replace(
    '    fn mk_temp_dir(prefix: &str) -> PathBuf {',
    '    #[cfg(feature = "analysis")]\n    fn mk_temp_dir(prefix: &str) -> PathBuf {'
)

code = code.replace(
    '    fn write_file(path: &Path, contents: &str) {',
    '    #[cfg(feature = "analysis")]\n    fn write_file(path: &Path, contents: &str) {'
)

with open("crates/tokmd-core/src/lib.rs", "w") as f:
    f.write(code)
