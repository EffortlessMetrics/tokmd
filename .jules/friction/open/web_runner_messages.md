# Friction: Web Runner payload schema drift vs real-world usage

- The memory block specifies: "In the `tokmd` `web/runner`, run message arguments can be passed via `inputs` (in-memory file arrays), `paths` (string arrays), or `scan` objects. Validation logic (e.g., `isRunMessage`) must accept payloads utilizing any of these valid structures, not strictly requiring `inputs` in all cases."
- However, as per #1367 review and the `browser-runner` contract, "The runner remains an in-memory input surface; native path/scan payloads should stay rejected unless the capability matrix changes first."
- This is an explicit conflict between documented agent guidelines and the enforced schema invariants. The work to loosen `inputs` was blocked/superseded. We should update the agent memory guideline or capability matrix to reflect this.
