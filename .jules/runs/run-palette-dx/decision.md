# Decision

## Option A (recommended)
Update `isRunMessage` in `web/runner/messages.js` to correctly accept `inputs` (in-memory file arrays), `paths` (string arrays), or `scan` (objects) as valid arguments. Update the `messages.test.mjs` test suite to assert these new supported argument structures.
- Fits the shard (web/runner) and persona (Palette, fixing DX by unblocking valid runtime inputs).
- Trade-offs: Structure is better as validation happens at the boundary; Governance is good as tests are included.

## Option B
Change `isRunMessage` to just check the envelope and let the underlying engine validate arguments instead.
- When to choose: if argument structures are extremely complex or change frequently.
- Trade-offs: Reduced structure and DX because validation errors would be more cryptic or deferred to lower layers.

## Decision
Proceeding with Option A because the structure of inputs/paths/scan is relatively stable and validating early improves runtime DX and error messages.
