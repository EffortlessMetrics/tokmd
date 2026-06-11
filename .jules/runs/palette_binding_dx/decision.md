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
Record a learning PR and a friction item.
Stop work as instructed by the user and preserve the context.

## Option B
Continue modifying the file in the wrong repository. (Not recommended).

## Decision
Chose Option A to respect the correct repository topology and avoid a wrong-repo intake.
