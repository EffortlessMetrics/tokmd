## Problem
The `web/runner/messages.js` has strict validation in `isRunArgsForMode`. If a user of the browser runner sends a validly structured run message but with a tiny mistake in `args` (for example, `args: { files: "yes" }` instead of `args: { files: true }` for `lang` mode, or passing `preset: "health"` for `analyze`), `isRunMessage` simply returns `false`.
Then `handleRunnerMessage` in `web/runner/runtime.js` responds with:
`[invalid_message] expected { type: "run", requestId, mode, args }`
This is incredibly confusing because the user *did* provide those fields! The real issue is deeply nested inside `args` validation (e.g. `isRunArgsForMode` failed because `files` isn't a boolean, or `scan` had an invalid structure). Wait, further exploration revealed the PR was closed due to a topology constraint (wrong-repo intake). I am acknowledging this.
