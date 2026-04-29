## Issue Analysis
The `tokmd` web runner (`web/runner/messages.js`) currently enforces strict input validation for `isRunMessage`. Specifically, the function `isRunArgsForMode` currently dictates that run arguments **must** include an `inputs` array (in-memory inputs). It rejects any messages that contain `paths` (string arrays) or `scan` objects directly in the arguments.

However, as per the system memory:
"In the `tokmd` `web/runner`, run message arguments can be passed via `inputs` (in-memory file arrays), `paths` (string arrays), or `scan` objects. Validation logic (e.g., `isRunMessage`) must accept payloads utilizing any of these valid structures, not strictly requiring `inputs` in all cases."

If we look at the test `run messages require explicit in-memory inputs` inside `web/runner/messages.test.mjs`, it explicitly tests that messages containing `paths` or `scan` objects are rejected:
```js
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "x",
            mode: "lang",
            args: { paths: ["src/lib.rs"] },
        }),
        false
    );
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "x",
            mode: "lang",
            args: {
                scan: {
                    inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                },
            },
        }),
        false
    );
```

We need to fix `isRunArgsForMode` in `web/runner/messages.js` to allow `inputs`, `paths`, and `scan` correctly, and update the associated tests in `web/runner/messages.test.mjs`.

## Option A (Recommended)
1. Update `isRunArgsForMode` in `web/runner/messages.js` to permit `inputs`, `paths`, or `scan` optionally, provided at least one valid source of input is present, or just allow these structures. Let's look closely at `isRunArgsForMode`.

```javascript
function isRunArgsForMode(mode, args) {
    if (!args || typeof args !== "object" || Array.isArray(args)) {
        return false;
    }

    const hasInputs = Array.isArray(args.inputs) && args.inputs.every(isInMemoryInput);
    const hasPaths = Array.isArray(args.paths) && args.paths.every(p => typeof p === "string");
    const hasScan = typeof args.scan === "object" && args.scan !== null;

    if (!hasInputs && !hasPaths && !hasScan) {
        return false;
    }
...
```
Wait, the memory says: "Validation logic (e.g., `isRunMessage`) must accept payloads utilizing any of these valid structures, not strictly requiring `inputs` in all cases."

Let's modify `isRunArgsForMode` and `hasOnlyKeys` check.

## Option B
Return a learning PR, however a direct code fix exists.

Decision: Option A
