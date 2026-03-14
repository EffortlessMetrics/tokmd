# Avoid push_str(&format!(...))

Context: Formatting tasks frequently build large strings to output files or standard output.
Pattern: Replace `out.push_str(&format!(...))` with `let _ = write!(out, ...)` or `writeln!(out, ...)`
Evidence: Reduced intermediate memory allocations by directly writing to the pre-allocated string buffer `out`.
Prevention: Use `write!` and `writeln!` macros from `std::fmt::Write`. To prevent trait ambiguity with `std::io::Write`, always scope `use std::fmt::Write;` locally inside the function instead of globally at the top of the file.
Links: None
