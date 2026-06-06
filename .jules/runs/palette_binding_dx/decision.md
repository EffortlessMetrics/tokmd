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
Instead of `match[2] || message`, we can use `match[2] || match[1]`.

Let's do Option A: Update `extractRunnerError` to use `match[2] || match[1]` so that empty message strings cleanly fall back to the bare code rather than the raw unparsed bracket string. We also add a test to `web/runner/runtime.test.mjs` to lock in this behavior.

## Option B
Do not fix this, as it is a minor issue. (Not recommended).
