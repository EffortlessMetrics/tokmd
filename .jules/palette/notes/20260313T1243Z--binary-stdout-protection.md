# Prevent Binary Output to TTY

**Context:** Tools that support both text and binary output formats (like `tokmd analyze --format midi`) risk dumping binary garbage characters directly into the user's terminal if stdout is not redirected.

**Pattern:** Use `std::io::IsTerminal` on `std::io::stdout()` before writing raw binary bytes. If it returns true, bail with a user-friendly error suggesting they redirect to a file or specify an output directory.

**Evidence:** `crates/tokmd/src/analysis_utils.rs` uses this to guard `RenderedOutput::Binary(bytes)`.

**Prevention Guidance:** Whenever a new binary format or renderer is introduced, ensure it either writes to a file or validates the stdout descriptor isn't a TTY.
