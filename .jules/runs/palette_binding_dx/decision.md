Okay, I see the whole picture.

1. **Option A: Remove `hasOnlyKeys` from `isRunArgsForMode` in `web/runner/messages.js`**.
Currently, the browser runner rejects valid `run` messages that contain extra properties (like `top`, `children`, `format`, `min_code`, `redact`, `hidden`, `excluded`, etc.). It returns a useless `expected { type: "run", requestId, mode, args }` generic error.
By relaxing this, we delegate argument validation to the `tokmd-wasm` Rust codebase which produces *excellent*, structured `invalid_settings` error messages.

Wait, is that it? Yes, the `isRunArgsForMode` uses `hasOnlyKeys(args, ["inputs", "scan"])` for `export` and `module`, which completely strips the ability to configure them!
```javascript
    if (mode === "analyze") {
        // analyze accepts preset and analyze
    }
    if (mode === "lang") {
        // lang accepts files
    }
    return hasOnlyKeys(args, ["inputs", "scan"]);
```
If you pass ANY option like `top: 10`, `format: "csv"`, `hidden: true`, `children: "separate"`, `web/runner/messages.js` rejects it! This means the Browser Runner cannot actually use the advanced configuration capabilities of `tokmd-wasm` because the JS wrapper arbitrarily drops the messages. This is definitely a major ergonomics issue!

Let's check if the browser runner handles `scan` recursively using `hasOnlyKeys`.
```javascript
function isScanOptions(value) {
    return value === undefined || (isPlainObject(value) && hasOnlyKeys(value, ["inputs"]));
}
```
Wait, `scan` accepts `hidden`, `excluded`, `no_ignore`... but the JS validation enforces `hasOnlyKeys(value, ["inputs"])`!
This means the Browser Runner **completely disables all `scan` options**!

The solution is to relax `isRunArgsForMode` and `isScanOptions` to only check structure (e.g. `isPlainObject`), and `resolveRunInputs(args)` handles finding the `inputs`. We shouldn't do deep property allow-listing in Javascript, because the Rust backend (`tokmd-wasm` -> `tokmd-core`) already does this and returns great error messages when something is actually wrong. The JS wrapper shouldn't reinvent validation and get it wrong.

So the plan is:
1. In `web/runner/messages.js`, remove `hasOnlyKeys` from `isScanOptions` and `isRunArgsForMode`. Just ensure they are objects and the `inputs` (if provided) conform to `isInMemoryInput`.
2. Wait, earlier I ran `npm test` and two tests failed when I removed `hasOnlyKeys`:
`run messages require explicit in-memory inputs`
`analyze run messages allow only explicit preset options with inputs`

Let's look at `web/runner/messages.test.mjs`.

```javascript
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "x",
            mode: "lang",
            args: {
                inputs: [{ path: "src/lib.rs", text: "pub fn alpha() {}\n" }],
                extra: true,
            },
        }),
        false // This will fail if we relax it!
    );
```
And:
```javascript
    assert.equal(
        isRunMessage({
            type: "run",
            requestId: "analyze-4",
            mode: "analyze",
            args: { inputs, analyze: { preset: "receipt", extra: true } },
        }),
        false
    );
```
We need to update these tests. They shouldn't expect `isRunMessage` to return `false` just because there are extra properties in `args`. The protocol wrapper shouldn't be strict about `args` contents, except ensuring `inputs` is correctly structured so the browser runner can process memory inputs.

Wait, the prompt says "Target ranking: 1) unclear or low-context error messages ... 2) confusing diagnostics ... 3) CLI help/default/usage sharp edges ... 4) public API ergonomics in code-facing surfaces".
Is there another Option?
What if the problem is `extractRunnerError` dropping `[code]`?
If `extractRunnerError` drops `[code]`, then in `web/runner/runtime.js` it returns `{ code, message }` (where message has no code). Then `handleRunnerMessage` does `createErrorMessage(..., extracted.code, extracted.message)`.
Then in `web/runner/main.js`, `event.data.error` is displayed as `${message.error.code}: ${message.error.message}`. So the user actually *does* see the code!
Ah! `main.js` does:
```javascript
            renderRunProgress({
                phase: "error",
                current: 0,
                total: 1,
                message: `${message.error.code}: ${message.error.message}`,
            });
            setStatus(
                runStatusOutput,
                `${message.error.code}: ${message.error.message}`,
                "error"
            );
```
So the code IS displayed! The user sees `invalid_settings: Invalid value for 'paths': expected an array`.
The fact that `extractRunnerError` separates the code and message is actually to allow `main.js` to render them explicitly together, rather than showing `[invalid_settings] [invalid_settings] Invalid value...`.

So the problem MUST be `isRunMessage` blocking valid settings and returning `[invalid_message] expected { type: "run", requestId, mode, args }`.
Option A fits perfectly: it's a runtime API ergonomics issue (public API ergonomics in code-facing surfaces) AND an unclear error message issue.

Option B: Let `hasOnlyKeys` stay but add all valid configuration properties to the lists. But this violates DRY (Don't Repeat Yourself) because Rust already defines the configuration schema, and keeping JS in sync manually is error-prone and the exact reason it is broken now.

The problem was definitely `isRunArgsForMode` in the browser runner blocking perfectly valid configuration objects. This resulted in zero ability for Browser runner callers to customize standard WASM settings (like `format`, `redact`, `children`, `top`, etc) while providing low-context `[invalid_message] expected { type: "run", requestId, mode, args }` generic errors when they try. The backend rust validations provide structured `invalid_settings` objects for invalid configurations.

We successfully fixed it, and the runner tests pass.

I am ready to run pre-commit checks and submit.
