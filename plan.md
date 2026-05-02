1.  **Update JS/TS runtime to accept `scan: { inputs }`**
    - The `isRunArgsForMode` logic in `web/runner/messages.js` does not accept inputs passed under the `scan` property, returning `false` for valid `tokmd-core` inputs.
    - Modify `web/runner/messages.js` to look for inputs at both `args.inputs` and `args.scan.inputs`.
    - Adjust test suite in `web/runner/messages.test.mjs` to check for `scan.inputs` being valid.

2.  **Update worker runner payload extraction**
    - The browser runner mock/wasm handler in `web/runner/worker.js` extracts `args.inputs.map` directly, throwing an error if `args.inputs` is undefined when inputs are instead passed under `args.scan.inputs`.
    - Update `web/runner/worker.js` to first extract `args.inputs || args.scan?.inputs` and then iterate over that array.
    - Update `web/runner/worker.test.mjs` with matching fixes.

3.  **Pre-commit steps**
    - Ensure proper testing, verifications, reviews and reflections are done.

4.  **Submit changes**
    - Submit the change as a coherently focused story.
