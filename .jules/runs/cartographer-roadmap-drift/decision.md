# Cartographer Decision

## Options Considered

### Option A: Update `implementation-plan.md` to reflect completed WASM/Browser work
The ROADMAP.md indicates that v1.9.0 (WASM/Browser Productization) is `✅ Complete` and has shipped:
```markdown
## v1.9.0 — Browser/WASM Productization
### What shipped in v1.9.0
- [x] `tokmd-io-port`, in-memory scan/model/core workflow seams...
- [x] `tokmd-wasm` exposes browser-friendly entrypoints...
- [x] Native-vs-wasm parity coverage exists...
- [x] `web/runner` boots the real `tokmd-wasm` bundle...
- [x] Public GitHub repo acquisition uses the browser-safe GitHub tree...
- [x] `tokmd-wasm` browser bundle is deployed...
- [x] Browser runner guardrails and UX hardening landed...
```
However, `docs/implementation-plan.md` still lists Phase 5 (WASM-Ready Core + Browser Runner (v1.9.0)) as pending, with unchecked boxes:
```markdown
### Work Items
- [ ] Route scan and walk through host-provided I/O traits
- [ ] Add wasm CI builds and parity checks against native output
- [ ] Expose JS-friendly wasm bindings for `lang`, `module`, `export`, and `analyze`
- [ ] Build a browser runner with progress, cancel, and download flows
- [ ] Add cache/guardrail policy for archive size, file count, and bytes read
```
This is a clear drift between the shipped reality and the implementation plan docs.

### Option B: Look for other roadmap drifts
We could look for drifts around the "tokmd-core Stabilization" phase.

## Decision
**Option A** is the best choice. It directly fixes a stale implementation plan section that misleads contributors into thinking WASM/Browser work is incomplete when it actually shipped in v1.9.0.

I will update `docs/implementation-plan.md` to mark Phase 5 as complete and check all the boxes, bringing it in sync with `ROADMAP.md` and the actual repository state.
