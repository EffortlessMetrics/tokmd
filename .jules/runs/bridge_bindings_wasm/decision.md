# Decision

## Option A
Update `isRunArgsForMode` in `web/runner/messages.js` to correctly accept `inputs` (in-memory file arrays), `paths` (string arrays), or `scan` objects as valid run arguments.

## Option B
Do not fix `web/runner/messages.js` but create a learning PR documenting the friction. The #1367 review disposition superseded this request, meaning the runner must remain an in-memory input surface and reject native path/scan payloads until the capability matrix explicitly supports it.

## Decision
Choose Option B to follow the updated review disposition. The change is obsolete. I will document this friction as a learning PR instead.
