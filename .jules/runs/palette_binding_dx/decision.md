# Decision

## Problem
The browser runner in `web/runner/runtime.js` has a bug in its `extractRunnerError` function. When it encounters an error string like `"[invalid_settings]"`, the regex `/^\[([^\]]+)\]\s*(.*)$/` matches. The first group is `"invalid_settings"`, and the second group is `""`.

Then it creates the extracted object:
```js
    const match = message.match(/^\[([^\]]+)\]\s*(.*)$/);
    if (match) {
        return { code: match[1], message: match[2] || message };
    }
```
Because `match[2]` is `""`, `match[2] || message` evaluates to `message`, which is the original unparsed string `"[invalid_settings]"`. This results in the error object having `code: "invalid_settings"` and `message: "[invalid_settings]"`.

This is confusing for runtime developers, as the error message still contains the bracket code, or is just the bracket code itself, instead of cleanly stripping the bracket and providing a clean message (or falling back to the code as the message if there's no additional text).

## Option A (recommended)
Fix `extractRunnerError` in `web/runner/runtime.js` to correctly handle bracketed codes without trailing messages.
Instead of `match[2] || message`, we can use `match[2] || match[1]` or simply check if `match[2]` is falsy and fall back to `match[1]` (the code) as the message if no message text is provided.

Wait, the best fallback for an empty `match[2]` is probably `match[1]` (the code itself), or we could just use `match[2] || message` but replace it with `match[2] ? match[2] : match[1]`. Wait, if we use `match[1]`, then `"[invalid_settings]"` becomes `code: "invalid_settings", message: "invalid_settings"`. If we just leave it empty, `message: ""` isn't great. Let's fall back to `match[1]`.

Let's check what `tokmd-wasm` actually returns. When `tokmd-wasm` panics or returns an error, it usually has `"[invalid_settings] Cannot use both paths and inputs"`. But if it only had `"[invalid_settings]"`, the current behavior reproduces `"[invalid_settings]"`. Wait, what if we just do `match[2] || match[1]`?

Let's do Option A: Update `extractRunnerError` to use `match[2] || match[1]` so that empty message strings cleanly fall back to the bare code rather than the raw unparsed bracket string. We also add a test to `web/runner/runtime.test.mjs` to lock in this behavior.

## Option B
Do not fix this, as it is a minor issue. (Not recommended).
