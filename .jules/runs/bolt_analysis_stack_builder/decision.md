## Options Considered

### Option A: Read text directly from bytes using std::str::from_utf8 instead of String::from_utf8_lossy or optimize from_utf8_lossy calls.
- **What it is:** The code currently calls String::from_utf8_lossy(&bytes) to read file text after checking is_text_like(&bytes). Since is_text_like already checks std::str::from_utf8(&bytes).is_ok(), we know the bytes are valid UTF-8. We can change the pattern to use std::str::from_utf8(&bytes).unwrap() to avoid string allocation/cloning.
- **Why it fits:** The prompt says 'Find and land one meaningful performance improvement inside the shard' and target ranking #2 is 'unnecessary allocations / cloning / string building'. By avoiding String::from_utf8_lossy allocation which creates a new Cow and allocates if the data is borrowed. String::from_utf8_lossy returns a Cow. The issue is it is often assigned to let text = ... and then used. If we can avoid it and just use std::str::from_utf8, it returns a str directly.
- **Trade-offs:** We skip redundant UTF-8 checking/copying if we know it is valid UTF-8 from is_text_like.

### Option B: Cache is_text_like parsing results
- **What it is:** Change is_text_like to return Option string slice instead of bool. That way we do the UTF-8 validation once and immediately get the string slice without having to call String::from_utf8_lossy or re-validate UTF-8.
- **When to choose it:** It reduces repeated work.
- **Trade-offs:** Changes is_text_like signature which might impact other parts of the codebase.

## Decision
Option B is the best. Target ranking #3 is 'repeated parsing/formatting that can be reused'. Currently, is_text_like does std::str::from_utf8(bytes).is_ok(). Then immediately after, callers do String::from_utf8_lossy(&bytes). We traverse the bytes twice for UTF-8 validation! We can change is_text_like to return Option string slice and name it something like as_text.
Let us create a new function as_text(bytes: &[u8]) -> Option<&str> which checks bytes.contains(&0) and then returns std::str::from_utf8(bytes).ok().
Then update callers to use it to get the string slice without a second UTF-8 pass.
This entirely eliminates the double-UTF-8 pass AND the String::from_utf8_lossy wrapper.
