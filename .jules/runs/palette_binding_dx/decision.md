# Decision

## Option A
Fix `isRunMessage` in `web/runner/messages.js` to allow `args.paths` and `args.scan` structures, rather than solely requiring `args.inputs`. The memory says: "In the tokmd web/runner, run message arguments can be passed via inputs (in-memory file arrays), paths (string arrays), or scan objects. Validation logic (e.g., isRunMessage) must accept payloads utilizing any of these valid structures, not strictly requiring inputs in all cases."
This is a Palette DX improvement because the browser runner currently rejects valid structures with an unhelpful validation error.

## Option B
Find an issue in the Python or Node APIs and fix it. While there might be other issues, the `web/runner` memory directly points to a bug with a clear fix that improves DX for clients.

## Decision
Option A. It's perfectly aligned with the assigned task "Improve or lock runtime-facing ergonomics across bindings/targets when the repo proves those surfaces exist" and is directly supported by our project memory.
