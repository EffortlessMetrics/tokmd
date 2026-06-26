## Target Evaluation
1. Fuzzers found that using `.unwrap_or(args)` allows invalid configuration shapes.
2. Specifically, if `lang` configuration is provided as a string instead of an object, it ignores the string and parses the root config, which masks an invalid type.

## Option A
Replace `.unwrap_or(args)` with a strict `nested_arg_object` helper that verifies the inner structure is an object and returns an explicit `TokmdError` if not.

- **Why**: Hardens the `run_json` FFI boundary against malformed inputs from Python/Node bindings, fulfilling the Fuzzer input hardening mission.
- **Trade-offs**: Stricter validation might break previously silently-accepted broken configurations.

## Option B
Do not fix it in code and only write a learning PR.

## Decision
Choose Option A. Hardening the core FFI boundary directly addresses the fuzzer persona's focus on input/parser surfaces.
