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

### Archive ingestion (proposed sub-seam)

A primary motivation for a provider-agnostic snapshot is scanning a repository
that arrives as a **compressed archive** (a downloaded ZIP/tarball, a CI
artifact, or an upload to a browser/worker target) without first extracting it
to the host filesystem. This sub-seam names that capability so the security
boundary is fixed before any implementation lands.

- **ArchiveProvider (proposed role)** — a `FileProvider` (`ReadFs`) backend that
  adapts the entries of an in-memory or streamed archive into the same
  read-only, normalized-path view that `HostFs` and `MemFs` already expose. An
  `ArchiveProvider` is a `FileProvider`; it does not introduce a parallel
  access contract. Conceptually it is `MemFs` populated from archive entries
  rather than from explicit `add_file` calls.
- **Supported container shape** — the first contemplated container is ZIP
  (central-directory enumerable, per-entry uncompressed size known before
  inflation). Tar-family containers are a later option behind the same role.
  This stub fixes the role and its limits, not the concrete crate or container
  matrix.

Archive ingestion is **trust-boundary-crossing input**: the archive may be
attacker-controlled. The contract below treats every entry as hostile until it
passes normalization and the resource limits. Ingestion must **fail closed**:
a single rejected entry fails the snapshot build with a named error rather than
silently dropping the entry and producing a misleadingly "complete" view.

#### Path safety limits

Each archive entry name is untrusted and must be normalized and validated
before it can become a `VirtualFile` path. An entry is **rejected** (not
sanitized-and-kept) when it:

- is absolute (leading `/`) or carries a drive/UNC prefix (e.g. `C:\`, `\\`);
- contains a `..` parent-traversal component after slash normalization;
- decodes to a non-UTF-8 or NUL-containing name;
- is a symlink, hardlink, device, or other non-regular-file entry (only regular
  file entries and the directories implied by them are admissible);
- collides case-insensitively or post-normalization with an already-admitted
  entry (ambiguous duplicate names are rejected, not last-write-wins).

The resulting path must satisfy the same forward-slash normalization rule as
`crates/tokmd-model` and must remain rooted inside the logical snapshot root.
No admitted path may escape the root via normalization.

#### Resource (zip-bomb) limits

Because compressed archives can expand by orders of magnitude, ingestion must
enforce explicit, caller-visible limits and reject (fail closed) on breach:

- a **per-entry uncompressed size** cap;
- a **total uncompressed size** cap across all admitted entries;
- a **maximum entry count** cap;
- a **maximum compression-ratio** guard per entry (declared/actual inflated
  size vs. compressed size) to catch highly compressible bomb entries even when
  individual caps are not yet hit.

Limits are part of the ingestion inputs (below), have documented conservative
defaults, and are recorded on rejection so a caller learns which bound tripped.
Inflation must honor the per-entry cap **during** decompression (bounded
reads), not only after, so a malicious declared size cannot force unbounded
allocation.

#### Incremental implementation status

The archive sub-seam landed in two deliberately separated steps so the
security-critical core is provable before any decompression dependency enters
the workspace:

1. **Admission engine (landed)** — the fail-closed path-safety and resource-limit
   core lives in `crates/tokmd-io-port/src/archive.rs` behind the
   `archive` Cargo feature, which carries **zero decompression dependencies**.
   It exposes `ArchiveLimits` (with conservative defaults),
   `RawArchiveEntry`/`EntryKind` (provider-agnostic descriptors of already
   decoded entries), a typed `ArchiveError`, and
   `snapshot_from_entries(root, entries, limits) -> Result<RepoSnapshot, ArchiveError>`.
   The engine validates and normalizes each untrusted name, rejects
   absolute/drive/UNC paths, `..` traversal, NUL/empty names, non-regular
   entries, and case-insensitive/post-normalization duplicates, then enforces
   the per-entry, total, entry-count, and compression-ratio limits and fails
   the whole build on the first violation. It reuses the existing `MemFs` +
   `RepoSnapshot` builder so admitted entries get host/in-memory parity for
   free.
2. **ZIP codec adapter (landed)** — a concrete ZIP decoder,
   `snapshot_from_zip_bytes(root, bytes, limits) -> Result<RepoSnapshot, ArchiveError>`,
   lives in the same module behind a separate `archive-zip` Cargo feature
   (`archive-zip = ["archive", "dep:zip"]`). The plain `archive` feature stays
   decompression-dependency-free; `archive-zip` is the trust-surface feature
   that pulls the audited [`zip`](https://crates.io/crates/zip) crate
   (deflate-only, `default-features = false`, so no aes-crypto/bzip2/lzma/xz/
   zstd/ppmd back-ends). The codec enumerates the ZIP central directory,
   classifies each entry (symlink/device/other via unix type bits → rejected;
   directory flag → no file; otherwise a regular file), **bounded-inflates**
   each regular file through a reader capped at `max_entry_size + 1` byte so a
   hostile declared size cannot force unbounded allocation, runs a running-total
   guard during decode, and delegates the authoritative admission policy to
   `snapshot_from_entries`. Malformed containers and unsupported compression
   methods surface as `ArchiveError::MalformedArchive`. The first
   implementation is buffered (whole archive supplied as a byte slice).
3. **Archive → scan consumer (landed)** — `crates/tokmd-scan` exposes
   `scan_snapshot_from_zip(root, bytes, limits, options)` behind its own
   `archive-zip` Cargo feature, which propagates `tokmd-io-port/archive-zip`.
   It composes the codec adapter (`snapshot_from_zip_bytes`) with the existing
   `scan_snapshot` materialization path: untrusted ZIP bytes are admitted
   fail-closed into a `RepoSnapshot`, then the admitted file set is routed
   through the same in-memory scan as every other snapshot. The default
   `tokmd-scan` surface stays decompression-dependency-free. A parity test
   (`crates/tokmd-scan/tests/archive_scan_parity.rs`) proves a benign ZIP scan
   matches the host scan of the equivalent extracted tree, and that a hostile
   traversal entry fails closed with no partial scan.

This split keeps the engine — the part that must be correct against hostile
input — fully unit-tested without committing the default `archive` dependency
graph to a codec, while the optional `archive-zip` feature carries the audited
decompression surface.

## Inputs

- A `FileProvider` (`ReadFs`) instance: `HostFs` for real runs, `MemFs` for
  tests and WASM, or any future host-supplied backend.
- A logical repository root path.
- The set of in-scope file paths (produced by the existing scan/walk surface;
  this spec does not change how that set is discovered).

A future `RepoSnapshot` builder reads each in-scope path through the provider
(`read_bytes` / `read_to_string`) and records a `VirtualFile` with a normalized
path and byte length.

For archive ingestion specifically, the inputs are:

- The archive bytes (an in-memory buffer or a bounded reader). This stub does
  not require streaming, but the limits must be enforceable without first
  inflating the whole archive.
- An ingestion limit set: per-entry uncompressed cap, total uncompressed cap,
  maximum entry count, and maximum per-entry compression ratio. Each has a
  documented conservative default; callers may tighten or (explicitly) relax
  them.
- The logical repository root the admitted entries are rooted under.

The path-safety and resource limits above are validated as entries are read.
The output of a successful archive ingestion is an `ArchiveProvider`
(`FileProvider`) — or directly a `RepoSnapshot` — that is indistinguishable
from the equivalent host-extracted scan for in-scope files.

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

For archive ingestion, the observable outputs are either a successful
provider/snapshot whose in-scope receipts match a host-extracted run, or a
named, fail-closed error identifying the first violated path-safety or resource
limit. Rejected archives produce no partial snapshot. The error surface (which
limit, which entry) is part of the contract; the concrete error type is left to
the implementing PR.

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
- Archive ingestion is optional and additive. It must sit behind a Cargo
  feature so the default `crates/tokmd-io-port` surface (and the default CLI)
  keeps zero archive/decompression dependencies. The host-backed path must not
  gain an archive dependency by default.
- Any archive/decompression dependency is a new trust surface. The implementing
  PR must select an audited, maintained crate, record license and security
  impact in the PR body (per the dependency rules), and route it through the
  normal `deny.toml` / dependency-maintenance proof. This stub does not pick or
  pin a crate.
- The ingestion limits are part of the support promise: tightening a default
  limit is a compatible hardening; loosening or removing one is a
  security-relevant change that must update this spec and its proof.

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

Additional proof obligations for the archive-ingestion sub-seam, when
implemented:

- Path-traversal rejection: fixtures containing `../` escapes, absolute paths,
  drive/UNC prefixes, NUL/non-UTF-8 names, and symlink/non-regular entries must
  each be rejected with a named error; none may produce a path outside the
  logical root.
- Resource-limit rejection: a zip-bomb-style fixture (small compressed, huge
  declared/inflated) must trip the per-entry, total, entry-count, or
  compression-ratio guard and fail closed without unbounded allocation.
- Archive/host parity: a benign archive fixture ingested through the
  `ArchiveProvider` must yield the same normalized file set and aggregated
  receipt as scanning the equivalent extracted tree through `HostFs`. The
  tokei-`Languages` inventory layer of this is covered by
  `crates/tokmd-scan/tests/archive_scan_parity.rs`; the full tokmd
  `LangReport`, `ModuleReport`, and `ExportData` receipts (including the
  `tokmd-model` byte and token totals) are anchored to a host filesystem scan
  by `crates/tokmd-core/tests/archive_host_receipt_parity.rs`. Host `module`
  scans auto-strip a single scan root; host `export` parity uses explicit
  `strip_prefix` matching the fixture root.
- Fail-closed semantics: a single rejected entry fails the whole snapshot build
  rather than silently dropping the entry and reporting `complete`.

Doc-shape checks for this spec stub itself:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
```

## Next integration points

The seam is intentionally inert until a consumer reads from it. The next
integration steps, in rough dependency order, are:

- **`crates/tokmd-scan` (snapshot-backed scanning)** — today `tokmd-scan`
  wraps tokei over the host filesystem (`HostFs`) and owns directory
  walking/ignore semantics. The first consumer seam is a snapshot-or-provider
  entry point that aggregates an already-captured `RepoSnapshot` (or any
  `ReadFs` provider) instead of re-touching `std::fs`, while leaving the
  default host-backed path and its receipts unchanged. Walking/ignore stays in
  `tokmd-scan`; the snapshot only supplies the in-scope file set and bytes.
- **`crates/tokmd-model` (parity oracle)** — the existing host/in-memory parity
  contract (`crates/tokmd-io-port/tests/repo_snapshot.rs` and
  `memfs_bdd.rs`) is the oracle a snapshot-backed scan must match: the same
  normalized file set and aggregated receipt as the equivalent host run.
- **`crates/tokmd-wasm` (host-free caller)** — the motivating consumer. A WASM
  or worker caller that cannot use `std::fs` builds a `RepoSnapshot` from a
  `MemFs` (or, once the codec adapter lands, directly from archive bytes) and
  runs the same aggregation, producing receipts indistinguishable from a
  host-backed run for in-scope files.
- **ZIP codec adapter (`archive-zip` feature, landed)** —
  `snapshot_from_zip_bytes` wires a buffered ZIP decoder over the admission
  engine (see the incremental status note above).
- **Archive → scan consumer (`tokmd-scan` `archive-zip` feature, landed)** —
  `scan_snapshot_from_zip` builds a snapshot from uploaded ZIP bytes and runs
  the existing aggregation (see the incremental status note above). The
  remaining archive-upload work is a host-free caller
  (`crates/tokmd-wasm`) that wires this consumer to a browser/worker upload
  surface.

These are forward-looking seams; none change current behavior. Each should land
as its own narrow PR behind the experimental marker until a real consumer
promotes the surface.

## Open Questions

- Should `RepoSnapshot` own file bytes eagerly, or hold a `FileProvider` handle
  and read lazily on demand? Eager capture is simpler and more deterministic;
  lazy reads lower peak memory for large repositories.
- Where should the proposed types live: a new module inside `crates/tokmd-scan`,
  or a dedicated crate once a second consumer (beyond `crates/tokmd-wasm`)
  appears? The default promotion ladder favors an internal module first.
- Does the seam need to carry per-file metadata (size is enough today) such as
  modified time or a content hash, or should that stay in the analysis layer?
- Which container matrix beyond ZIP (tar/gzip) is worth supporting? The first
  codec selected the audited [`zip`](https://crates.io/crates/zip) crate
  (deflate-only, default features off) behind `archive-zip`; tar-family
  containers remain an open option behind the same admission engine.
- What are the right default values for the ingestion limits (per-entry cap,
  total cap, entry-count cap, compression-ratio guard), and should they scale
  with an explicit caller "expected repo size" hint instead of fixed constants?
  The landed admission engine ships conservative tunable constants (64 MiB per
  entry, 1 GiB total, 65,536 entries, 100x ratio) via `ArchiveLimits::default`;
  these remain provisional and may be revised once a real codec/consumer
  exercises them.
- Should archive ingestion stream entries (lower peak memory, harder to bound)
  or require a fully buffered archive (simpler limit enforcement) for the first
  implementation?
- Should rejected-entry diagnostics be aggregated (report all violations) or
  fail on the first violation? Fail-fast is simpler and is the default this
  stub assumes.
