## 💡 Options A and B

### Option A: Improve error message formatting and guidance for Unrecognized subcommand
- The current message is:
  `Error: Unrecognized subcommand 'abc'`
  `Hints:`
  `- Verify the input path exists and is readable.`
  `- Use an absolute path to avoid working-directory confusion.`
- The hints are confusing because they refer to paths, not subcommands. The subcommand logic was added as a fallback to catch mis-parsed subcommands vs paths.
- I will improve the hint to include listing available subcommands via `tokmd --help` instead of showing path hints when it is definitively interpreted as a subcommand.

### Option B: Fix enum value validation error messages
- `error: invalid value 'xyz' for '--format <FORMAT>'` could provide better hints on how to list all options.
- This is primarily handled by clap, so changing it might be more involved.

Decision: Option A.
