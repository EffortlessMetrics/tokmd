# Default PR gate

After PR 10, the ordinary `pull_request` gate runs only the cheap
"frontdoor" lanes plus the existing proof / cockpit / typos jobs.
Expensive lanes are gated on labels and on push-to-main:

| Job | Now triggers on PR when... |
|-----|----------------------------|
| `Build & Test` (Linux) | always |
| `Build & Test (Windows)` | label `windows` / `full-ci` (still on every push) |
| `Build & Test (macOS)` | push-only (was already) |
| `Wasm Compile & Test` | label `wasm` / `full-ci` |
| `Nix PR Package Gate` | label `nix` / `release-check` / `full-ci` |
| `Mutation Testing (Required)` | label `mutation` / `full-ci` (replaced by ripr advisory in PR 11) |
| `Proptest Smoke` | label `property-tests` / `full-ci` |
| `MSRV Check` | always |
| `Quality Gate` | always |
| `Cargo Deny` | always |
| `Typos` | always |
| `Proof Policy` | always |
| `Affected Proof Plan` | always |
| `Feature Boundaries` | always |
| `Publish Surface` | always (small dry-run) |
| `Version consistency` | always |
| `Docs Check` | always |

## CI (Required) summary

The aggregator's `if: always()` posture means **skipped jobs do not fail**
the summary — only `failure` and `cancelled` results do. So a default PR
that skips Windows, WASM, Nix, mutation, and proptest will still see a
green `CI (Required)` row provided the lanes that *did* run all passed.

## Default-PR LEM after the slimming

Roughly (per PR 02's `inventory.md`, with cache normalised in PR 09):

```text
msrv_check                   5
quality_gate                 8
proof_policy                 3
affected_proof_plan          4
feature_boundaries          10
typos                        1
cargo_deny                   4
version_consistency          2
docs_check                   4
build (Linux only)          12   (matrix Linux entry)
publish_surface              8
ci_required                  1
                          ----
                            62   default PR (was ~203)
```

PR 11 will add a `ripr` advisory at ~2 LEM in place of the demoted
mutation lane, leaving the default ordinary PR around 64 LEM — within
the `elevated` band on first roll-out, with PR 12's risk-pack routing
designed to bring this further down for narrow PRs.

## Anti-patterns

- Don't use `full-ci` to dodge a real failure; the deep lanes catch
  things the default lane is *intentionally* skipping.
- Don't apply per-pack labels to silence routing — fix the change.
- Don't depend on the matrix entry name "windows" appearing under
  `build` — the matrix split is intentional so `if:` can gate Windows
  independently.
