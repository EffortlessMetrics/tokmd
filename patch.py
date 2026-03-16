import re

with open("crates/tokmd-core/src/lib.rs", "r") as f:
    code = f.read()

code = code.replace("mod tests {\n    \n    \n    \n    ", "mod tests {\n    use super::*;\n    #[cfg(feature = \"analysis\")]\n    use crate::settings::AnalyzeSettings;\n")

with open("crates/tokmd-core/src/lib.rs", "w") as f:
    f.write(code)
