# Spec: Repo Snapshot Portability Seam

- Status: draft
- Schema family, if any: none yet (no serialized schema is introduced by this stub)
- Related ADRs: none yet
- Related proof scopes: `io_port`, `scan`, `model`
- Related crates: `crates/tokmd-io-port`, `crates/tokmd-scan`, `crates/tokmd-model`, `crates/tokmd-wasm`

## Contract

This spec names a portability seam that lets tokmd scan and aggregate a
repository view that is **not bound to the host filesystem**. It is a
forward-looking contract stub: it records the intended shape and boundaries so
later implementation work has a stable target. It introduces no runtime
behavior change on its own.

Three roles make up the seam. Two already exist; one is proposed.

- **FileProvider (role, already present)** — the abstract read-only file access
  port. Today this role is filled by the `ReadFs` trait in
  `crates/tokmd-io-port/src/lib.rs`, with two backends:
  - `HostFs` delegates to `std::fs` (the default, used by the CLI today).
  - `MemFs` is an in-memory store used by tests and WASM targets.
  This spec does **not** redefine `ReadFs`; it adopts it as the FileProvider
  contract and gives that role a durable name.

- **VirtualFile (proposed type)** — a single provider-agnostic file entry: a
  normalized forward-slash path plus its bytes (or a lazily readable handle)
  and byte length. No `VirtualFile` type exists yet; this stub fixes the
  intended fields and naming so the future type does not drift.

- **RepoSnapshot (proposed type)** — a deterministic, captured set of
  `VirtualFile` entries rooted at a logical repository root, built from any
  `FileProvider`. A snapshot is the unit that scanning and aggregation should be
  able to consume without re-touching the host filesystem. No `RepoSnapshot`
  type exists yet.

The boundary this seam protects:

- The snapshot is **read-only** and **provider-agnostic**: once built, scan and
  model logic operate on the snapshot, not on `std::fs` directly.
- Path normalization stays consistent with the existing `crates/tokmd-model`
  rule (forward slashes, no OS-specific separators).
- Determinism is mandatory: a snapshot built from the same provider state must
  enumerate files in a stable order (the in-memory backend already keys files
  in a sorted map; see `crates/tokmd-io-port/src/lib.rs`).

Out of scope for this stub: directory walking/ignore semantics (those remain in
`crates/tokmd-scan`), content analysis, and any serialized on-disk snapshot
format.

## Inputs

- A `FileProvider` (`ReadFs`) instance: `HostFs` for real runs, `MemFs` for
  tests and WASM, or any future host-supplied backend.
- A logical repository root path.
- The set of in-scope file paths (produced by the existing scan/walk surface;
  this spec does not change how that set is discovered).

A future `RepoSnapshot` builder reads each in-scope path through the provider
(`read_bytes` / `read_to_string`) and records a `VirtualFile` with a normalized
path and byte length.

## Outputs

- A `RepoSnapshot` value: an ordered, provider-agnostic collection of
  `VirtualFile` entries plus the logical root.
- The snapshot is consumable by aggregation in `crates/tokmd-model` and by
  callers that cannot use the host filesystem (notably `crates/tokmd-wasm`),
  without changing the receipts those callers already produce for the
  equivalent host-backed run.

No new serialized output (JSON/JSONL/CSV) is defined here. If a serialized
snapshot format is later required, it must get its own schema version and a
follow-on spec.

## Compatibility

- Additive only. The default CLI path keeps using `HostFs` and the existing
  `crates/tokmd-scan` traversal; this seam must not alter host-backed receipts.
- Adopting the FileProvider name for `ReadFs` is a documentation move, not a
  trait change. Any future change to `ReadFs` itself remains governed by normal
  crate-boundary and semver rules for `crates/tokmd-io-port`.
- `RepoSnapshot` / `VirtualFile`, when implemented, should land first as private
  or `pub(crate)` types behind a real boundary and only be promoted to a public
  support surface once a consumer needs it (see the crate/module boundary rules
  in `docs/architecture.md`).

## Proof Requirements

These are the proof obligations a future implementing PR must satisfy; this
stub asserts none of them as already met.

- Host/in-memory parity: scanning a fixture through `HostFs` and through
  `MemFs` must yield the same normalized file set and the same aggregated
  receipt. The in-memory backend already has contract coverage in
  `crates/tokmd-io-port/tests/memfs_bdd.rs` to build on.
- Determinism: snapshot enumeration order must be stable across runs and
  independent of insertion order.
- Path normalization: snapshot paths must match the forward-slash rule enforced
  by `crates/tokmd-model`.

Doc-shape checks for this spec stub itself:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
```

## Open Questions

- Should `RepoSnapshot` own file bytes eagerly, or hold a `FileProvider` handle
  and read lazily on demand? Eager capture is simpler and more deterministic;
  lazy reads lower peak memory for large repositories.
- Where should the proposed types live: a new module inside `crates/tokmd-scan`,
  or a dedicated crate once a second consumer (beyond `crates/tokmd-wasm`)
  appears? The default promotion ladder favors an internal module first.
- Does the seam need to carry per-file metadata (size is enough today) such as
  modified time or a content hash, or should that stay in the analysis layer?
