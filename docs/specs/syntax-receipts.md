# Spec: Syntax Receipts

- Status: draft
- Schema family, if any: `tokmd.syntax_receipt.v1`
- Related ADRs: `docs/adr/0008-ast-foundation.md`
- Related proof scopes: `analysis_ast_shadow`

## Contract

Syntax receipts are feature-gated parser availability and parse-status evidence
for future review packets. They exist to prove that tokmd can parse selected
languages deterministically and report degradation explicitly before syntax
facts are promoted into default analysis output.

The first syntax receipt slice must:

- stay behind the explicit `ast` feature;
- lock parser support to TypeScript, TSX, Rust, and Python;
- report unsupported, skipped, parse-degraded, and parser-failed files as
  advisory receipt states instead of silent omissions;
- keep generated and vendor paths skipped by policy unless explicitly included
  by a future caller;
- keep large-file skips bounded by a recorded byte limit;
- avoid network access, global parser discovery, timestamps, absolute paths,
  and nondeterministic ordering;
- leave default `tokmd analyze`, `tokmd cockpit`, `tokmd context`,
  `tokmd handoff`, FFI, Python, Node, and WASM outputs unchanged until a later
  schema-reviewed receipt promotion.

Syntax receipts must not claim that AST evidence proves undefined behavior,
panic reachability, semantic call edges, public reachability, or parser risk.
Those claims require separate receipts with their own proof and schema review.

## Parser Registry

The locked parser registry is:

| Language | Extensions | Parser crate | Grammar symbol |
| --- | --- | --- | --- |
| Rust | `rs` | `tree-sitter-rust` | `tree_sitter_rust` |
| TypeScript | `ts`, `mts`, `cts` | `tree-sitter-typescript` | `tree_sitter_typescript` |
| TSX | `tsx` | `tree-sitter-typescript` | `tree_sitter_tsx` |
| Python | `py`, `pyw` | `tree-sitter-python` | `tree_sitter_python` |

Adding a language is a schema-affecting registry change. It must include parser
metadata, extension routing tests, degradation tests, and proof that the parser
does not require network or environment-specific setup.

## Inputs

The first syntax receipt builder accepts:

- a normalized or normalizable repository-relative path;
- caller-supplied source text;
- a parser option set containing the maximum syntax byte limit and generated or
  vendor skip policy;
- the feature-gated locked parser registry compiled into `tokmd-analysis`.

The syntax receipt path must not require:

- network access;
- runtime parser downloads;
- GitHub Actions metadata;
- Codecov upload;
- evidencebus runtime dependencies;
- browser, WASM, Python, or Node binding support.

## Outputs

The single-file output is a library-facing `tokmd.syntax_receipt.v1` value. The
feature-gated CLI producer emits a top-level `tokmd.syntax_receipts.v1` packet
that indexes one or more file receipts for a scoped path set. It is not emitted
by default `tokmd analyze`, cockpit, context, handoff, FFI, Python, Node, or
WASM paths.

Every receipt records:

- normalized path;
- optional language wire value;
- optional parser crate and grammar symbol;
- parse status;
- advisory flag;
- optional human-readable reason;
- source byte count;
- optional root node kind;
- parser error state.
- syntax fact arrays for symbols, imports, exports, call sites, and risk seams;
- derived review signals that normalize language-specific seams into advisory
  categories for later review-priority consumers.

The output must avoid timestamps, absolute paths, environment-specific temporary
directories, and nondeterministic ordering.

## Receipt Shape

The explicit syntax producer is available only when the `tokmd` binary is built
with the `ast` feature:

```bash
tokmd syntax src/runtime/api src/bun.js/bindings
```

It emits a packet with schema family `tokmd.syntax_receipts.v1`:

```json
{
  "schema": "tokmd.syntax_receipts.v1",
  "status": "partial",
  "paths": ["src/runtime/api"],
  "max_bytes": 1048576,
  "skip_generated_vendor": true,
  "receipts": [],
  "warnings": [],
  "errors": [],
  "non_claims": [
    "syntax receipts package advisory parser evidence; they do not prove reachability, bug presence, UB presence, safety, or merge readiness"
  ]
}
```

Packet status is `complete` when all file receipts are complete, `partial` when
one or more file receipts are advisory or the scoped path set is empty, and
`failed` when requested inputs are missing, unreadable, or cannot be walked. A
failed packet is printed before the command exits nonzero so bots can attach or
inspect the named error.

A syntax parse receipt uses schema family `tokmd.syntax_receipt.v1`:

```json
{
  "schema": "tokmd.syntax_receipt.v1",
  "path": "src/runtime/api/example.ts",
  "language": "typescript",
  "parser_crate": "tree-sitter-typescript",
  "grammar_symbol": "tree_sitter_typescript",
  "status": "complete",
  "advisory": false,
  "reason": null,
  "source_bytes": 128,
  "root_kind": "program",
  "has_error": false,
  "symbols": [
    {
      "kind": "function",
      "name": "bindNative",
      "span": {
        "start_line": 10,
        "start_column": 1,
        "end_line": 13,
        "end_column": 2
      },
      "exported": true,
      "public_surface": true
    }
  ],
  "imports": [
    {
      "kind": "static",
      "module": "bun:ffi",
      "imported": ["FFIType", "dlopen"],
      "dynamic": false,
      "span": {
        "start_line": 1,
        "start_column": 1,
        "end_line": 1,
        "end_column": 41
      }
    }
  ],
  "exports": [
    {
      "kind": "function",
      "name": "bindNative",
      "span": {
        "start_line": 10,
        "start_column": 1,
        "end_line": 13,
        "end_column": 2
      }
    }
  ],
  "call_sites": [
    {
      "kind": "call",
      "callee": "dlopen",
      "dynamic": false,
      "span": {
        "start_line": 5,
        "start_column": 23,
        "end_line": 9,
        "end_column": 3
      }
    }
  ],
  "risk_seams": [
    {
      "kind": "native_boundary_hint",
      "evidence": "dlopen",
      "span": {
        "start_line": 5,
        "start_column": 23,
        "end_line": 9,
        "end_column": 3
      }
    }
  ],
  "review_signals": [
    {
      "category": "native_boundary",
      "severity": "high",
      "score": 90,
      "kind": "native_boundary_hint",
      "reason": "native, FFI, or binding-ish boundary hint",
      "evidence": "dlopen",
      "span": {
        "start_line": 5,
        "start_column": 23,
        "end_line": 9,
        "end_column": 3
      }
    }
  ]
}
```

Supported statuses:

| Status | Meaning |
| --- | --- |
| `complete` | The locked parser produced a tree and the root node has no syntax errors. |
| `parse_degraded` | The parser recovered a tree, but syntax errors were present. |
| `parser_failed` | The parser could not be loaded or produced no tree. |
| `skipped_generated_or_vendor` | Policy skipped a generated or vendor path. |
| `skipped_too_large` | The file exceeded the configured syntax byte limit. |
| `unsupported_language` | No locked parser exists for the file extension. |

Every status except `complete` must set `advisory` to `true` and include a
reason suitable for a human reviewer and a bot log.

Fact arrays are deterministic and may be empty. Spans use 1-based line and
column numbers.

Review signals are deterministic, derived from the fact arrays, and may be
empty. They are advisory ordering hints for later evidence packets and review
priority summaries, not semantic reachability or bug claims. Signal categories
are intentionally language-agnostic so consumers can rank review targets without
knowing every parser-specific seam kind.

For panic/native review presets such as `bun-ub`, syntax ranking deprioritizes
`panic_seam` signals tagged with `test_context: true`. These come from Rust
`#[cfg(test)]` modules, `mod tests` blocks, `#[test]` functions, or equivalent
test-only regions where assertion macros would otherwise crowd production panic
seams. Deprioritized signals keep their original kind and evidence but surface
with lower effective `severity`/`score` and an explicit reason suffix.

| Category | Typical source |
| --- | --- |
| `native_boundary` | FFI, native, binding, `dlopen`, `ctypes`, or similar evidence. |
| `panic_seam` | Rust panic, assertion, unwrap/expect, indexing, or allocation seams. |
| `dynamic_execution` | Dynamic eval/call sites or dynamic constructors. |
| `dynamic_import` | Runtime imports. |
| `process_boundary` | Subprocess or shell-call seams. |
| `io_boundary` | File I/O seams. |
| `exception_path` | Python exception raise/handler seams. |
| `entrypoint` | Entrypoint-like call or `__main__` patterns. |
| `public_surface` | Exported or public/API-ish symbols. |
| `guard_evidence` | Nearby guard evidence that may bound a higher-risk seam. |

The TypeScript/TSX first slice populates:

- exported functions, classes, members, and variables as symbols and exports;
- static imports and dynamic `import(...)` calls;
- function and constructor call sites;
- risky casts/assertions, non-null assertions, dynamic imports, dynamic calls,
  native or binding-ish hints, and entrypoint-like calls such as `Bun.serve`.

The Rust first slice populates:

- public/API-ish functions, structs, enums, and traits as symbols and exports;
- `use` declarations as imports;
- function calls, method calls, and macro invocations as call sites;
- `unwrap`, `expect`, `try_from(...).expect(...)`, indexing expressions,
  capacity/allocation calls, panic/assert/unreachable/todo macros, and nearby
  `if`/`match` guard evidence as risk seams.

The Python first slice populates:

- module, class, and function symbols, with public names surfaced as exports;
- `import` and `from ... import ...` declarations;
- call sites;
- `if __name__ == "__main__"` and common `main`/`app.run` entrypoints,
  `subprocess`/`os.system` calls, `eval`/`exec`/`compile`, dynamic imports,
  dynamic calls, file-open calls, native or FFI-ish hints such as
  `ctypes`/`cffi`, exception raises/handlers, and nearby `if`/`try`/`with`
  guard evidence as risk seams.

## Compatibility

This slice is additive and feature-gated. It adds optional parser dependencies
behind the existing `ast` feature and does not change default receipt schemas or
default CLI behavior.

Compatibility requirements:

- builds without `--features ast` must not require the new parser crates;
- default `tokmd analyze`, `tokmd cockpit`, `tokmd context`, `tokmd handoff`,
  FFI, Python, Node, and WASM outputs remain unchanged;
- AST shadow comparison artifacts keep schema family `tokmd.ast_shadow.v1`;
- non-Rust shadow inputs are reported as unsupported rather than parsed until a
  later comparison-runner PR promotes a language;
- parser failures and parse degradation remain advisory evidence, not proof
  promotion or merge verdicts.

## Proof Requirements

The parser registry proof must cover:

- stable language wire values and schema name;
- parser crate and grammar symbol metadata for TypeScript, TSX, Rust, and
  Python;
- extension routing, including uppercase extension normalization;
- successful parse receipts for supported languages;
- explicit `parse_degraded` receipts for malformed syntax;
- `unsupported_language` receipts for files outside the locked registry;
- generated/vendor policy skips;
- large-file skip receipts with the configured byte limit;
- non-Rust shadow inputs remain unsupported in the AST shadow comparison path
  until a later comparison runner promotes them.
- TypeScript and TSX fixtures prove exports, imports, dynamic imports,
  entrypoint calls, native or binding-ish hints, call sites, and risky
  cast/assertion seams.
- Rust fixtures prove public symbols, `use` imports, call and macro sites,
  unwrap/expect, fallible conversion plus `expect`, indexing, capacity
  allocation, panic/assert macros, and guard evidence.
- Python fixtures prove module/class/function symbols, imports, call sites,
  entrypoints, subprocess/eval/dynamic import/call and file-open seams, native
  or FFI-ish hints, exception signals, and guard evidence.
- cross-language review signal normalization proves TypeScript/TSX, Rust, and
  Python fixtures emit comparable categories and rank high-severity signals
  first.

The `tokmd syntax` command is the first explicit producer for these receipts.
Default analysis, cockpit, context, handoff, FFI, Python, Node, and WASM outputs
remain unchanged. Evidence packet manifests may index `syntax.json` and surface
its `review_signals` as advisory `review_priority` items with refs back to the
syntax receipt. Specialized panic-seam receipts remain a later, separate
contract.
