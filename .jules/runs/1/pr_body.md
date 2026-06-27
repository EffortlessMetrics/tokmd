## 💡 Summary
Explored hardening the FFI setting payloads boundary in tokmd-core by replacing .unwrap_or(args) with strict object type checks. Discovered a duplicate/superseding PR already landed this exact fix, so concluding with a learning PR instead.

## 🎯 Why
In the FFI `run_json` boundary, which serves Python and Node wrappers, the configuration payload logic was extracting nested setting objects via `.unwrap_or(args)`. If a user incorrectly provided a non-object (e.g. `{"lang": "bogus"}`), the code would fail to parse the inner block and silently fallback to using `args` as the context object, missing explicit validation on the structure boundaries.

## 🔎 Evidence
- `crates/tokmd-core/src/ffi/settings_parse.rs`
- FFI configuration structure assumes the extracted properties are objects.

## 🧭 Options considered
### Option A (recommended)
- Record the findings as a friction item and end with a learning PR, as the issue was reported as superseded by another PR.
- Preserves the insights gathered.
- Trade-offs: None.

### Option B
- Attempt to force the patch despite being told to stop.
- Risks conflicts and wasted effort.
- Trade-offs: Unnecessary.

## ✅ Decision
Option A. End with a learning PR to respect the superseded notification.

## 🧱 Changes made (SRP)
- `.jules/friction/open/FRIC-20260626-001.md`
- `.jules/personas/fuzzer/notes/input_boundary_hardening.md`

## 🧪 Verification receipts
```text
cat .jules/friction/open/FRIC-20260626-001.md
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: Internal documentation
- Risk class: Zero
- Rollback: Delete the added files.
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/1/envelope.json`
- `.jules/runs/1/decision.md`
- `.jules/runs/1/receipts.jsonl`
- `.jules/runs/1/result.json`
- `.jules/runs/1/pr_body.md`
- `.jules/friction/open/FRIC-20260626-001.md`
- `.jules/personas/fuzzer/notes/input_boundary_hardening.md`

## 🔜 Follow-ups
- Address FRIC-20260626-001 if the superseding PR didn't fully resolve it.
