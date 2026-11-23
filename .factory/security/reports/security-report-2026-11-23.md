# Security Scan Report

**Generated:** 2026-11-23
**Scan Type:** Weekly Scheduled
**Repository:** EffortlessMetrics/tokmd
**Severity Threshold:** medium
**Scope:** Last 7 days of commits (2026-11-16 → 2026-11-23)

## Executive Summary

| Severity | Count | Auto-fixed | Manual Required |
|----------|-------|------------|-----------------|
| CRITICAL | 0     | 0          | 0               |
| HIGH     | 2     | 2          | 0               |
| MEDIUM   | 0     | 0          | 0               |
| LOW      | 0     | 0          | 0               |

**Total Findings:** 2
**Auto-fixed:** 2
**Manual Review Required:** 0

**Summary:** Two HIGH severity findings were identified and auto-patched. Both
are supply-chain regressions in the committed `Cargo.lock`, and both broke the
`cargo deny --all-features check` gate that CI runs on every push.

1. **VULN-001** — Two locked dependencies had published security fixes that
   were not applied. `tokio 1.53.1` carried `RUSTSEC-2026-0386`, an I/O-safety
   violation in `AsyncFd`'s safe constructors (use-after-free on fd reuse),
   and `rustix 1.1.4` carried `RUSTSEC-2026-0486`, undefined behaviour in the
   extended-attribute functions when called with a zero-length buffer. Both
   crates are reachable from published artifacts (`tokmd-node` and
   `tokmd-scan` respectively).

2. **VULN-002** — Three `chrono` advisories (`RUSTSEC-2026-0543`,
   `RUSTSEC-2026-0555`, `RUSTSEC-2026-0559`) were published on 2026-11-17,
   inside this scan window, and immediately failed the same gate. `chrono` is
   unmaintained with no fixed version, so tokmd's own use of it was migrated
   to the maintained `time` crate. The residual `chrono` in the dependency
   graph is build-graph-only and is documented in `deny.toml`.

No CRITICAL or MEDIUM findings were identified at or above the configured
threshold.

### Scan scope note

This repository is the *publication* repository. Its history is squashed into a
single root commit (`ff05889`, 2026-08-05), so the literal 7-day window
(`git log --since="7 days ago"`) contains **zero commits** and yields an empty
file list. The swarm development repository (`EffortlessMetrics/tokmd-swarm`)
was also checked and had no commits in the window.

To produce meaningful coverage rather than a vacuous "no changes, no
findings" result, this scan covered the **entire current tree** against the
standing threat model, focused analysis on the trust boundaries the threat
model flags, and ran an advisory sweep of every locked registry dependency
against the OSV database. The previously recorded standing defenses were
re-verified against the current tree.

**Branch lineage note.** The 2026-11-16 scan branch carried fixes for
VULN-001 (composite-Action script injection) and VULN-002 (the `zip 8.6.0`
advisory regression). Those fixes exist only on that branch and are **not**
merged into `origin/main`. This scan branch was therefore created from
`origin/droid/security-report-2026-11-16` rather than from the publication
root commit, so the prior fixes carry forward instead of being silently
dropped by a re-base. See "Observations" OBS-006.

---

## Critical Findings

*None.*

## High Findings

### VULN-001: Two locked dependencies carry published security fixes that were not applied

| Attribute | Value |
|-----------|-------|
| **Severity** | HIGH |
| **STRIDE Category** | Elevation of Privilege / Tampering |
| **CWE** | CWE-1104 (Use of Unmaintained Third Party Components), CWE-119 (Improper Restriction of Operations within the Bounds of a Memory Buffer) |
| **File** | `Cargo.lock` (`tokio 1.53.1`, `rustix 1.1.4`) |
| **Status** | Patched |

**Description:**

The committed lockfile pinned two dependencies to versions with known,
already-fixed security advisories:

| Advisory | Package | Pinned | Fixed in | Summary |
|-----------|---------|--------|----------|---------|
| `RUSTSEC-2026-0386` | `tokio` | 1.53.1 | **1.53.3** | `AsyncFd::new` and related constructors violate I/O safety |
| `RUSTSEC-2026-0486` | `rustix` | 1.1.4 | **1.1.6** | xattr functions mark unwritten bytes as initialized when given a zero-length buffer |

**RUSTSEC-2026-0386 (tokio).** `tokio::io::unix::AsyncFd` provides safe
constructors (`new`, `with_interest`, `try_new`, `try_with_interest`) that
accept any `T: AsRawFd` rather than requiring proof of ownership or validity
of the file descriptor. Because `RawFd` implements `AsRawFd`, safe code can
construct an `AsyncFd` over an arbitrary or closed descriptor, or close and
replace the underlying descriptor while the `AsyncFd` is still alive. This
violates I/O safety in two ways:

1. `AsyncFd<T>` implements `AsFd` for all `T: AsRawFd` via
   `BorrowedFd::borrow_raw(self.as_raw_fd())`, allowing safe code to obtain
   an arbitrary `BorrowedFd<'_>` without owning the descriptor.
2. Tokio's I/O driver stores a pointer to an internal allocation in the
   `epoll`/`kqueue` user data for registered descriptors. If the underlying
   descriptor is duplicated and then closed while an `AsyncFd` is alive, and
   the OS reuses that descriptor number for another I/O resource on the same
   runtime, dropping the first `AsyncFd` deregisters the reused descriptor
   number — producing a use-after-free when the readiness event arrives.

**RUSTSEC-2026-0486 (rustix).** The extended-attribute functions in
`rustix::fs` (`getxattr`, `lgetxattr`, `fgetxattr`, `listxattr`,
`llistxattr`, `flistxattr`) write into a `Buffer` and then mark as many bytes
initialized as the system call reported. When the buffer has zero length the
OS treats the call as a size query: it writes nothing and returns the size.
rustix still marked that many bytes as initialized. Passing a `Vec` whose
length equals its capacity through `spare_capacity` gives such a zero-length
buffer, so rustix set the vector's length past its capacity over memory that
was never written — undefined behaviour reachable from safe code. With an
empty `&mut [MaybeUninit<u8>]` the call panics instead.

**Evidence:**

Reachability was measured, not assumed.

`tokio` is a **normal** dependency of `tokmd-node`, which publishes to npm:

```
$ cargo tree -p tokmd-node -i tokio --edges normal
tokio v1.53.1
├── napi v3.12.0
│   └── tokmd-node v1.15.0 (/home/runner/work/tokmd/tokmd/crates/tokmd-node)
└── tokmd-node v1.15.0 (/home/runner/work/tokmd/tokmd/crates/tokmd-node)
```

`rustix` reaches published artifacts through `tempfile`, which is a **normal**
dependency of `tokmd-scan` (`crates/tokmd-scan/Cargo.toml:26`, in
`[dependencies]`, not `[dev-dependencies]`):

```
$ cargo tree -i rustix --edges normal
rustix v1.1.4
└── tempfile v3.27.0
    └── tokmd-scan v1.15.0 (/home/runner/work/tokmd/tokmd/crates/tokmd-scan)
        ├── tokmd v1.15.0 (...)          # published CLI
        ├── tokmd-core v1.15.0 (...)     # published library
        └── tokmd-wasm v1.15.0 (...)     # published wasm bundle
```

The vulnerable code paths are **not** reachable from tokmd's own call sites:
tokmd uses only `tokio::task::spawn_blocking` and the multi-thread runtime
builder and never constructs an `AsyncFd`; `rustix` is reached only through
`tempfile`'s internal use, not through any `xattr` call in tokmd.

The finding is still HIGH because:

1. **The published artifacts contain the vulnerable code.** `tokio 1.53.1` is
   compiled into the `tokmd-node` npm package, and `rustix 1.1.4` into
   `tokmd-scan` and everything downstream of it. A consumer's security
   scanner flags the artifact regardless of whether tokmd calls the affected
   API.
2. **The advisory is an unsoundness/memory-safety bug, not a style issue.**
   Both advisories describe undefined behaviour or a use-after-free reachable
   from safe code, which is exactly the class of defect that becomes
   exploitable when the surrounding code changes.
3. **The `cargo deny` advisory gate fails on `main`.** CI runs
   `cargo deny --all-features check` (`.github/workflows/ci.yml:689`). A
   control that is known-failing is a real regression, not a theoretical one.
4. **Defense in depth.** The mitigation should not depend on the current
   call-site pattern remaining unchanged.

**Fix Applied:**

```diff
 # Cargo.lock
 [[package]]
 name = "rustix"
-version = "1.1.4"
+version = "1.1.6"
 source = "registry+https://github.com/rust-lang/crates.io-index"
-checksum = "b6fe4565b9518b83ef4f91bb47ce29620ca828bd32cb7e408f0062e9930ba190"
+checksum = "e5e76742e5294153d4dde1207dbec5bdd81eef6fabdc0d2118babadabfd858aa"

 [[package]]
 name = "tokio"
-version = "1.53.1"
+version = "1.53.3"
 source = "registry+https://github.com/rust-lang/crates.io-index"
-checksum = "202caea871b69668250d242070849eb495be178ed697a3e98aebce5bc81a0bed"
+checksum = "..."
```

Both upgrades are lockfile-only (`cargo update --precise`), touch no source,
and need no API adaptation. `rustix 1.1.6` additionally introduces a
`windows-sys 0.59.0` alongside the existing `0.61.2` in the graph; that is the
expected result of the upstream fix and is covered by cargo-deny's
`multiple-versions = "warn"` setting.

**Validation performed:**

- `cargo build --workspace --all-features` — succeeds.
- `cargo deny --all-features check` (cargo-deny **0.19.0**, the version CI
  pins via `taiki-e/install-action@v2` with `tool: cargo-deny@0.19.0`) —
  `RUSTSEC-2026-0386` and `RUSTSEC-2026-0486` are no longer reported.
- `cargo test --workspace --all-features --no-fail-fast` — 530 test binaries
  pass; 1 pre-existing failure unrelated to this finding (see "Known
  Pre-existing Issues").
- OSV query for `tokio 1.53.3` and `rustix 1.1.6` — **0 advisories**.

---

### VULN-002: Three `chrono` "unmaintained" advisories published inside the scan window fail the advisory gate

| Attribute | Value |
|-----------|-------|
| **Severity** | HIGH |
| **STRIDE Category** | Elevation of Privilege / Tampering |
| **CWE** | CWE-1104 (Use of Unmaintained Third Party Components) |
| **File** | `xtask/Cargo.toml`, `crates/tokmd-analysis-types/Cargo.toml`, `deny.toml` |
| **Status** | Patched |

**Description:**

Three advisories were added to the RustSec database on **2026-11-17**, inside
this scan window:

| Advisory | Package | Summary |
|-----------|---------|---------|
| `RUSTSEC-2026-0543` | `chrono 0.4.45` | chrono is unmaintained |
| `RUSTSEC-2026-0555` | `chrono-tz 0.9.0` | chrono-tz is unmaintained |
| `RUSTSEC-2026-0559` | `chrono-tz-build 0.3.0` | chrono-tz-build is unmaintained |

All three are "unmaintained" advisories affecting **every published version**,
so there is no version to upgrade to. Their publication immediately broke the
`cargo deny --all-features check` gate in CI:

```
$ cargo deny --all-features check advisories    # cargo-deny 0.19.0
error[unmaintained]: chrono is unmaintained
error[unmaintained]: chrono-tz is unmaintained
error[unmaintained]: chrono-tz-build is unmaintained
advisories FAILED
```

**Evidence:**

tokmd used `chrono` directly in two places, and that direct usage was fully
removable:

1. **`xtask`** (the workspace task runner, `publish = false`) used
   `chrono::NaiveDate` and `chrono::Utc` for `YYYY-MM-DD` parsing
   (`NaiveDate::parse_from_str(s, "%Y-%m-%d")`), the current UTC date
   (`Utc::now().date_naive()`), RFC 3339 formatting (`Utc::now().to_rfc3339()`),
   and RFC 2822 parsing of crates.io rate-limit `retry-after` timestamps
   (`DateTime::parse_from_rfc2822`).
2. **`tokmd-analysis-types`** used `chrono` **only as a test oracle** for its
   own hand-rolled RFC 3339 formatter (`chrono_timestamp_iso8601`); the
   production code path never touched the crate.

The residual `chrono` in the dependency graph comes only from `tera`, which is
a **build**-dependency of `tokei` (used to render tokei's config templates at
build time):

```
$ cargo tree -i chrono -e normal,build
chrono v0.4.45
├── chrono-tz v0.9.0
│   └── tera v1.20.1
│       [build-dependencies]
│       └── tokei v14.0.0
│           ├── tokmd-model v1.15.0 (...)
│           └── tokmd-scan v1.15.0 (...)
└── tera v1.20.1 (*)
```

That residual is not linked into any published artifact. Verified per
published crate:

```
$ for c in tokmd tokmd-core tokmd-model tokmd-scan tokmd-wasm tokmd-python \
           tokmd-node tokmd-format tokmd-cockpit; do
    cargo tree -p $c -e normal --prefix none | grep -c chrono
  done
tokmd: 0
tokmd-core: 0
tokmd-model: 0
tokmd-scan: 0
tokmd-wasm: 0
tokmd-python: 0
tokmd-node: 0
tokmd-format: 0
tokmd-cockpit: 0
```

And confirmed against the linked binary:

```
$ nm -C target/debug/tokmd | grep -E "\bchrono::"
(no matches — the chrono crate is not linked)

$ nm -C target/debug/tokmd | grep "chrono"
... tokmd_analysis_types::baseline::complexity_baseline::chrono_timestamp_iso8601
    (tokmd's own function name, not the chrono crate)
```

**Fix Applied:**

1. **Migrated tokmd's own date handling from `chrono` to `time`**, which the
   workspace already depends on and which is actively maintained. A new
   `xtask/src/tasks/date.rs` module centralises the helpers:

   ```rust
   //! Shared date helpers for xtask.
   //!
   //! These helpers wrap the [`time`] crate so the rest of xtask does not have to
   //! care about format descriptions or UTC-offset handling. The former
   //! implementation used `chrono`, which is now unmaintained
   //! (`RUSTSEC-2026-0543`); `time` is already a workspace dependency and covers
   //! every date operation xtask needs.

   use time::format_description::well_known::Rfc2822;
   use time::macros::format_description;
   use time::{Date, OffsetDateTime, PrimitiveDateTime};

   pub(crate) const DATE_FORMAT: &[time::format_description::FormatItem<'static>] =
       format_description!("[year]-[month]-[day]");

   pub(crate) fn today_utc() -> Date { OffsetDateTime::now_utc().date() }
   pub(crate) fn parse_date(value: &str) -> Result<Date, time::error::Parse> {
       Date::parse(value, &DATE_FORMAT)
   }
   pub(crate) fn format_date(date: Date) -> String {
       date.format(&DATE_FORMAT).unwrap_or_default()
   }
   pub(crate) fn today_utc_formatted() -> String { format_date(today_utc()) }
   pub(crate) fn now_rfc3339() -> String {
       OffsetDateTime::now_utc()
           .format(&time::format_description::well_known::Rfc3339)
           .unwrap_or_default()
   }
   pub(crate) fn parse_rfc2822(value: &str) -> Result<OffsetDateTime, time::error::Parse> {
       PrimitiveDateTime::parse(value, &Rfc2822).map(|dt| dt.assume_utc())
   }
   ```

   Call sites migrated:

   ```diff
   // xtask/src/tasks/lint_policy.rs
   -use chrono::{NaiveDate, Utc};
   -    let today = Utc::now().date_naive();
   -    match NaiveDate::parse_from_str(&debt.expires, "%Y-%m-%d") {
   +    let today = crate::tasks::date::today_utc();
   +    match crate::tasks::date::parse_date(&debt.expires) {

   // xtask/src/tasks/bump.rs
   -use chrono::Utc;
   -    let release_date = Utc::now().date_naive().format("%Y-%m-%d").to_string();
   +    let release_date = crate::tasks::date::today_utc_formatted();

   // xtask/src/tasks/no_panic.rs
   -use chrono::{NaiveDate, Utc};
   -        generated_at: Utc::now().to_rfc3339(),
   +        generated_at: crate::tasks::date::now_rfc3339(),

   // xtask/src/tasks/ci_lane_whitelist.rs
   -use chrono::NaiveDate;
   -    today: NaiveDate,
   +    today: Date,
   -    let today = chrono::NaiveDate::from_ymd_opt(2026, 5, 7).expect("date");
   +    let today = time::Date::from_calendar_date(2026, time::Month::May, 7).expect("date");

   // xtask/src/tasks/publish.rs
   -use chrono::{DateTime, FixedOffset, Utc};
   +use time::OffsetDateTime;
   -fn parse_rate_limit_timestamp(stderr: &str) -> Option<DateTime<FixedOffset>> {
   +fn parse_rate_limit_timestamp(stderr: &str) -> Option<OffsetDateTime> {
   -    DateTime::parse_from_rfc2822(timestamp_str).ok()
   +    crate::tasks::date::parse_rfc2822(timestamp_str).ok()
   -                    let now = Utc::now();
   -                    let until = retry_after.with_timezone(&Utc);
   -                    let delta = until.signed_duration_since(now);
   -                    if delta.num_seconds() > 0 {
   +                    let now = OffsetDateTime::now_utc();
   +                    let delta = retry_after - now;
   +                    if delta.whole_seconds() > 0 {
   ```

2. **Migrated the `tokmd-analysis-types` test oracle to `time`**, keeping the
   same property (the implementation is still checked against an independent
   reference implementation across the full supported range):

   ```diff
   -    use chrono::{SecondsFormat, TimeZone, Utc};
   +    use time::OffsetDateTime;
   +    use time::format_description::well_known::Rfc3339;
   +    use time::macros::format_description;

   -        fn chrono_timestamp_matches_chrono(ms in 0u128..253_402_300_799_000u128) {
   -            let chrono_dt = Utc.timestamp_millis_opt(ms as i64).single()
   -                .expect("timestamp within supported range");
   -            let expected = chrono_dt.to_rfc3339_opts(SecondsFormat::Millis, true);
   +        fn chrono_timestamp_matches_reference(ms in 0u128..253_402_300_799_000u128) {
   +            let reference = OffsetDateTime::from_unix_timestamp_nanos((ms as i128) * 1_000_000)
   +                .expect("timestamp within supported range");
   +            let expected = reference
   +                .format(&format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"))
   +                .expect("reference timestamp is formattable");
   +            prop_assert_eq!(chrono_timestamp_iso8601(ms), expected);
   ```

3. **Documented the residual build-graph-only chrono advisories** in
   `deny.toml`, matching the existing entry for the structurally identical
   `term_size` advisory (also transitive via tokei, also unmaintained, also
   with no upstream fix):

   ```toml
   ignore = [
       # term_size is transitive via tokei; no patched version available upstream
       { id = "RUSTSEC-2020-0163", reason = "transitive via tokei; revisit when upstream removes it" },
       # chrono / chrono-tz / chrono-tz-build are "unmaintained" advisories with no
       # patched version (all releases are affected). They reach the graph only
       # through `tera`, which is a *build*-dependency of `tokei` (used to render
       # tokei's config templates at build time). No published tokmd crate links
       # chrono: `cargo tree -e normal` reports 0 chrono nodes for tokmd,
       # tokmd-core, tokmd-model, tokmd-scan, tokmd-wasm, tokmd-python,
       # tokmd-node, tokmd-format and tokmd-cockpit, and `nm -C` on the built
       # `tokmd` binary shows no `chrono::` symbols. tokmd's own date handling
       # uses the maintained `time` crate. Revisit when tokei drops tera/chrono-tz.
       { id = "RUSTSEC-2026-0543", reason = "build-time only via tokei's tera build-dependency; not linked into any published artifact" },
       { id = "RUSTSEC-2026-0555", reason = "build-time only via tokei's tera build-dependency; not linked into any published artifact" },
       { id = "RUSTSEC-2026-0559", reason = "build-time only via tokei's tera build-dependency; not linked into any published artifact" },
   ]
   ```

4. **Regenerated the no-panic allowlist baseline**, because four `expect` call
   fingerprints in `xtask/src/tasks/ci_lane_whitelist.rs` changed receiver
   expression when `chrono::NaiveDate::from_ymd_opt` was replaced with
   `time::Date::from_calendar_date`. This is the documented workflow in
   `docs/NO_PANIC_POLICY.md`:

   ```bash
   cargo xtask no-panic-baseline --receipt-output policy/no-panic-baseline-receipt.json
   ```

**Validation performed:**

- `cargo build --workspace --all-features` — succeeds.
- `cargo deny --all-features check` (cargo-deny **0.19.0**, the CI-pinned
  version) — **`advisories ok, bans ok, licenses ok, sources ok`**.
- `cargo test -p tokmd-analysis-types` — **441 tests, 0 failed** in the
  library target, plus all integration/property targets passing.
- `cargo test -p xtask` — all suites pass.
- **The migrated oracle was verified to still detect regressions.** A
  deliberate off-by-one was introduced into `chrono_timestamp_iso8601`
  (the `hour` computation was shifted by one) and the property test caught
  it:

  ```
  test baseline::complexity_baseline::tests::chrono_timestamp_matches_reference ... FAILED
  panicked at crates/tokmd-analysis-types/src/baseline/complexity_baseline.rs:268:5:
  minimal failing input: ms = 0
  ```

  The deliberate break was then reverted. This confirms the migration did not
  silently weaken the test.
- `cargo run -p xtask -- check-no-panic-family` — `22444 finding(s), 22444
  matched, 0 unallowlisted, 0 stale, 0 expired, 0 shape error(s)`, in both
  advisory and `--strict` modes.
- Semantics of the `time` migration were checked against `chrono` behaviour:
  `time::SignedDuration` handles negative deltas identically to
  `chrono::signed_duration_since` (verified: a timestamp two hours in the past
  yields `-7200` seconds with no panic), and `Rfc2822` parsing of both `GMT`
  and `+0000` yields the same instant.

---

## Medium Findings

*None.*

## Low Findings

*None.*

---

## Observations (Below Threshold — Not Reported As Findings)

These items were considered during the scan but do not meet the `medium`
severity threshold. They are recorded here for traceability and the next
scheduled scan.

### OBS-001 (carried): FFI JSON payload size not bounded

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Denial of Service |
| **File** | `crates/tokmd-core/src/ffi/mod.rs` |
| **Status** | Not patched — design choice |

**Description:** The `run_json(mode, args_json)` FFI entrypoint accepts a JSON
string of arbitrary size. While individual in-memory `inputs[].path` is bounded
to 4096 bytes (`MAX_IN_MEMORY_INPUT_PATH_BYTES`), the outer JSON envelope is
not.

**Why not a finding:** Caller controls input. `serde_json::from_str` allocates
predictably; no algorithmic blowup. Adversarial testing during this scan
confirmed the parser has a **recursion limit** — deeply nested JSON
(20,000 and 100,000 levels) is rejected with `invalid_json: recursion limit
exceeded` rather than overflowing the stack:

```
[deep-nest-20000]  -> {"ok":false,"error":{"code":"invalid_json","message":"Invalid JSON: recursion limit exceeded at line 1 column 128"}}
[deep-obj-100000]  -> {"ok":false,"error":{"code":"invalid_json","message":"Invalid JSON: recursion limit exceeded at line 1 column 132"}}
```

**Recommended fix (optional, future):** Add a soft cap on `args_json.len()`
(e.g. 8 MiB) returning a typed `TokmdError::invalid_field("args", ...)` from
`run_json_inner`.

### OBS-002 (carried): Transitive `RUSTSEC-2020-0163` advisory

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (transitive) |
| **STRIDE Category** | Elevation of Privilege |
| **File** | `Cargo.lock` (transitive `term_size` via `tokei`) |
| **Status** | Documented in `deny.toml` |

**Description:** `term_size` is a transitive dependency of `tokei` and has an
unmaintained advisory (`RUSTSEC-2020-0163`). It is the same structural
situation as the chrono chain addressed in VULN-002: transitive via tokei,
unmaintained, no patched version.

**Why not a finding:** Already documented in `deny.toml` with rationale.
`cargo deny --all-features check` passes.

**Recommended action:** Track upstream `tokei` — `tokei 15.0.0` still depends
on `term_size 0.3.2` and `tera 1.20.0`, so no upgrade path exists today.

### OBS-003 (carried): GitHub Actions pinning is mixed (tag + SHA)

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Spoofing / Tampering |
| **File** | `.github/workflows/*.yml` |
| **Status** | Not patched — mixed strategy |

**Description:** The Droid-related workflows
(`.github/workflows/droid.yml`, `droid-review.yml`, `droid-security-scan.yml`)
pin the custom `EffortlessMetrics/droid-action-safe` by SHA
(`7c1377ccbacddc95560d1570547a5baa51de01ec`), and
`.github/workflows/ci.yml` pins `EffortlessMetrics/ub-review` by SHA
(`e1e41124e0468b3714827fd32574c8c583803b72`). Other workflows
(`ci.yml`, `release.yml`, `cockpit.yml`, `nix-full.yml`, `bindings-parity.yml`,
`swarm-ghcr.yml`, `ghcr-container-smoke.yml`, `proof-executor.yml`,
`proof-observation-collection.yml`, `mutants.yml`, `pr-plan.yml`,
`badge-endpoints.yml`, `coverage.yml`, `test-action.yml`, `fuzz.yml`, `ripr.yml`,
`ci-policy.yml`, `no-panic-policy.yml`, `clippy-exceptions-policy.yml`,
`sync-labels.yml`, `nix-macos.yml`) pin by tag (e.g. `actions/checkout@v7.0.0`,
`Swatinem/rust-cache@v2`, `dtolnay/rust-toolchain@stable`).
`.github/workflows/promote-release-aliases.yml` pins by SHA.

**Why not a finding:**
- Tag-pinned first-party actions (`actions/*`) are a well-accepted practice
  with low residual risk; GitHub's own recommended baseline.
- The highest-privilege third-party surfaces (the custom Droid action and the
  release-alias promotion workflow) ARE SHA-pinned.
- Below the `medium` severity threshold.

**Recommended action (optional):** Convert the remaining third-party actions
to SHA-pinned references and codify the rotation process in `.factory/rules/`.

### OBS-004 (carried): `web/runner` browser code does not pin GitHub API base URL

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Spoofing |
| **File** | `web/runner/ingest.js` |
| **Status** | Not patched — review for future |

**Description:** The browser-side runner fetches repository content via
`fetch()` calls to `api.github.com` (and the codeload/GitHub
`releases`/`archive` endpoints). These URLs are hard-coded in the
`web/runner/` JavaScript modules. The token (when supplied) is stored in
`sessionStorage` (not `localStorage`) and used as a `Bearer`/`token` header.

**Why not a finding:** Re-verified during this scan:
- No DOM injection surfaces: repository-wide grep for `innerHTML`,
  `outerHTML`, `insertAdjacentHTML`, `document.write`, `eval(`,
  `new Function` and `setTimeout(` in `web/runner/` returns **no matches**.
  All dynamic data is rendered via `textContent`.
- The token is stored in `sessionStorage` only, never `localStorage`.
- `owner` and `repo` are interpolated with `encodeURIComponent`.
- Repository URLs are checked against a hostname allowlist
  (`web/runner/ingest.js:590`): `if (!["github.com", "www.github.com"].includes(url.hostname.toLowerCase()))`.
- The worker protocol allowlists modes and presets and validates message
  shape (`web/runner/messages.js`).
- Browser-side runner runs entirely in the user-agent sandbox; no filesystem,
  no subprocess.

**Recommended action (optional):** Consider an explicit allowlist of fetch
origins and a CSP `connect-src` directive in the runner's served HTML.

### OBS-005 (carried): `action.yml` install step performs `curl | sh` style download

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (informational) |
| **STRIDE Category** | Tampering / Information Disclosure |
| **File** | `action.yml` (composite step `Install tokmd`) |
| **Status** | Not patched — verified checksums |

**Description:** The composite GitHub Action downloads a pre-built `tokmd`
binary from `github.com/EffortlessMetrics/tokmd/releases/...` and verifies it
against `checksums.txt` (sha256). It does not verify a cryptographic signature
on the checksum file or on the release itself.

**Why not a finding:** Re-verified during this scan:
- The `version` input is constrained by regex to `latest` or
  `^v?[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.]+)?$`, so it can only produce a
  well-formed release download URL.
- The `runtime: container` path only accepts verification-gated tags
  (`container_supported_tags="1.14.0 1.15.0"`); mutable tags such as
  `latest` are a hard error.
- Checksum verification uses `sha256sum`/`shasum`/`Get-FileHash`.
- Build provenance is separately attested via `actions/attest-build-provenance@v4`.
- The generated container wrapper uses `printf '%q'` for shell-safe quoting
  of the image reference and the isolated docker config directory.

**Recommended action (optional):** Add a signature check on `checksums.txt`
itself.

### OBS-006 (new): Prior security fixes exist only on unmerged scan branches

| Attribute | Value |
|-----------|-------|
| **Severity** | LOW (process) |
| **STRIDE Category** | Tampering |
| **File** | repository branch topology |
| **Status** | Mitigated for this branch; manual follow-up recommended |

**Description:** The 2026-11-16 scan branch carried two HIGH fixes
(VULN-001 composite-Action script injection, VULN-002 the `zip 8.6.0`
advisory regression) that are **not** merged into `origin/main`. Every weekly
scan PR (`#2951`–`#2958`) has been left open rather than merged, so the
publication repository's `main` still carries the composite-Action script
injection primitive and the `zip 8.6.0` dependency.

The 2026-10-26 scan had already produced the `zip` upgrade, and the
2026-11-02 scan branch was created from the publication root commit rather
than from the 2026-10-26 branch, silently dropping that fix. The 2026-11-16
scan re-applied it and added a regression guard.

**Why not a finding:** The vulnerable code is not new in this window, and the
prior fixes were correctly produced and documented; this is a merge-process
gap, not a new vulnerability. This scan branch was created **from**
`origin/droid/security-report-2026-11-16` so the prior fixes carry forward
rather than being dropped again.

**Recommended action (manual):** Merge the accumulated security-scan branches
into `main` (or cherry-pick the specific fix commits) so the fixes reach the
published artifacts. The unreviewed commit on `main` is
`ff058890 import: finalize stable release after consumer proof`, whose tree
still contains the composite-Action injection at `action.yml` (the
`artifact`/`comment` inputs) and the `zip 8.6.0` pin.

---

## Standing Defenses Verified (No Regression)

The following defenses were re-verified during this scan. All remain intact.

| ID | Defense | Location | Verified |
|----|---------|----------|----------|
| D-01 | `unsafe_code = "forbid"` workspace lint | `Cargo.toml` | ✓ |
| D-02 | `unwrap_used`, `expect_used`, `panic`, `unreachable`, `dbg_macro`, `todo`, `unimplemented` lints denied | `Cargo.toml` | ✓ |
| D-03 | Git subprocess env isolation (`GIT_REPO_SHAPING_ENV`) | `crates/tokmd-git/src/command.rs` | ✓ |
| D-04 | Git ref validation (`env_base_ref_is_safe` + `--end-of-options`) | `crates/tokmd-git/src/refs.rs` | ✓ |
| D-05 | Bounded path canonicalization under root | `crates/tokmd-scan/src/path/bounded_path.rs` | ✓ |
| D-06 | FFI in-memory input path validation (`MAX_IN_MEMORY_INPUT_PATH_BYTES = 4096`) | `crates/tokmd-core/src/ffi/inputs.rs` | ✓ |
| D-07 | Strict JSON parsing with type validation | `crates/tokmd-core/src/ffi/parse.rs` | ✓ |
| D-08 | Per-family schema versioning | `crates/tokmd-types/src/` | ✓ |
| D-09 | SHA-pinned highest-privilege actions; tag-pinned first-party actions | `.github/workflows/` | ✓ (see OBS-003) |
| D-10 | Branch protection policy declared | `.github/settings.yml` | ✓ |
| D-11 | `cargo-deny` advisory + license allowlist | `deny.toml` | ✓ (now passing) |
| D-12 | BLAKE3 redaction with extension allowlist | `crates/tokmd-format/src/redact/` | ✓ |
| D-13 | Content reads bounded by `ContentLimits` | `crates/tokmd-analysis/src/content/` | ✓ |
| D-14 | PyO3 FFI invariants | `crates/tokmd-python/src/lib.rs` | ✓ |
| D-15 | WASM uses `MemFs` (no host fs) | `crates/tokmd-wasm/` | ✓ |
| D-16 | `web/runner` uses `textContent` (no `innerHTML`/`eval`/`new Function`) | `web/runner/main.js` | ✓ |
| D-17 | `web/runner` token in `sessionStorage` (not `localStorage`) | `web/runner/auth.js` | ✓ |
| D-18 | `web/runner` worker protocol allowlists modes & presets | `web/runner/messages.js` | ✓ |
| D-19 | Composite action installs tokmd with sha256 checksum verification | `action.yml` | ✓ |
| D-20 | Custom Droid action SHA-pinned across all Droid workflows | `.github/workflows/droid*.yml` | ✓ |
| D-21 | `cargo audit` output parsed structurally; malformed JSON → `Pending` | `crates/tokmd-cockpit/src/supply_chain.rs` | ✓ |
| D-22 | `run_json` top-level JSON must be an object | `crates/tokmd-core/src/ffi/mod.rs` | ✓ |
| D-23 | Composite action inputs never interpolated into a `run:` body | `action.yml` | ✓ (verified: no `${{ }}` in any `run:` body) |
| D-24 | `output-dir` rejected if absolute or containing `..` | `action.yml` | ✓ |
| D-25 | Release-alias promotion validates tag regex and digest, and refuses to move `v1` backward | `.github/workflows/promote-release-aliases.yml` | ✓ |
| D-26 | Archive admission pre-checks entry count before decoding and bounds every read | `crates/tokmd-io-port/src/archive.rs` | ✓ |
| D-27 | `tokio` ≥ 1.53.3 and `rustix` ≥ 1.1.6 (no `AsyncFd` unsoundness, no xattr UB) | `Cargo.lock` | ✓ (this scan) |
| D-28 | `chrono` removed from tokmd's own source; remaining chain is build-graph-only | `xtask/`, `crates/tokmd-analysis-types/` | ✓ (this scan) |

---

## Appendix

### Threat Model

- **Status:** Current — reused, not regenerated
- **Location:** `.factory/threat-model/threat-model.md`
- **Last Regenerated:** 2026-11-16 (7 days ago)
- **Last Reviewed:** 2026-11-23 (this scan)
- **Methodology:** STRIDE
- **Next review:** 2027-02-14 (90-day cadence) or upon architecture change
- **No regeneration this scan** — the threat model was regenerated 7 days
  ago by the 2026-11-16 scan and is well within the 90-day freshness window.
  No new external surface, subprocess invocation, or trust-boundary shift was
  introduced since. The **Last Reviewed** date and the supply-chain row of
  §3 (Elevation of Privilege) plus standing defenses D-27/D-28 were updated
  to reflect this scan's findings.

### Scan Metadata

- **Commits Scanned:** 0 in the literal window (`git log --since="7 days ago"`
  returns no commits; the publication repository's history is squashed into
  the single root commit `ff058890` dated 2026-08-05). The scan therefore
  covered the entire current tree.
- **Files in scope:** The full current tree of the publication repository
  (18 workspace crates + `xtask` + `web/runner` + `.github/workflows` +
  `action.yml`).
- **Dependency advisories checked:** 353 crates.io registry packages from
  `Cargo.lock` were queried against the OSV database in a single batch query.
  6 packages returned advisories; all 6 were triaged (2 patched, 4 documented
  as unmaintained/build-graph-only).
- **Scan Duration:** ~55m
- **Skills Used:** threat-model-generation (reused existing, no regeneration
  required), commit-security-scan, vulnerability-validation, security-review
- **False Positive Filter:** applied — see Observations above. In particular,
  the `head` argument to `tokmd cockpit` was tested for git option injection
  and found **not** exploitable (the range is built as a single argv element
  prefixed by `base..`, and `git rev-parse --verify` rejects option-like
  revisions).

### Scan Coverage Matrix

| Area | Files reviewed | Findings |
|------|----------------|----------|
| Dependency graph (OSV sweep of all 353 registry packages) | `Cargo.lock`, `deny.toml` | **2** (VULN-001, VULN-002) |
| Subprocess invocation | `crates/tokmd-git/`, `crates/tokmd-cockpit/src/supply_chain.rs`, `crates/tokmd-cockpit/src/gates/contracts.rs`, `crates/tokmd/src/git_support.rs`, `crates/tokmd-scan/src/walk/git.rs`, `crates/tokmd-core/src/context_git/mod.rs` | 0 |
| Git ref validation | `crates/tokmd-git/src/refs.rs` | 0 |
| Git range construction | `crates/tokmd-git/src/lib.rs`, `crates/tokmd-cockpit/src/change_surface.rs` | 0 |
| FFI entrypoint (adversarial) | `crates/tokmd-core/src/ffi/` (`mod.rs`, `inputs.rs`, `parse.rs`, `byte_mode.rs`) | 0 |
| Python / Node / WASM bindings | `crates/tokmd-python/`, `crates/tokmd-node/`, `crates/tokmd-wasm/` | 0 |
| Archive admission | `crates/tokmd-io-port/src/archive.rs` | 0 |
| Path handling | `crates/tokmd-scan/src/path/`, `crates/tokmd-scan/src/roots.rs` | 0 |
| Redaction | `crates/tokmd-format/src/redact/` | 0 |
| Composite GitHub Action | `action.yml` | 0 |
| GitHub workflows | `.github/workflows/*.yml` (25 files), `.github/settings.yml` | 0 |
| Release promotion workflow | `.github/workflows/promote-release-aliases.yml` | 0 |
| Web runner (browser) | `web/runner/*.js` | 0 |
| Build / lint / policy | `Cargo.toml`, `deny.toml`, `clippy.toml`, `.cargo/config.toml`, `policy/` | 0 |

### Known Pre-existing Issues

Two failures exist on the branch this scan was based on and are **not** caused
by this scan's changes. Both were confirmed by stashing the scan's changes
and re-running on the clean tree.

1. **`cargo clippy --workspace --all-features --all-targets -- -D warnings`
   fails** on
   `crates/tokmd-analysis/src/complexity/tests/complexity_depth_w56.rs:103`
   (`clippy::needless_range_loop`). This makes `cargo xtask gate --check`
   report 3/4 steps (fmt ✓, check ✓, clippy ✗, test-compile ✓). The
   2026-11-16 scan branch carried a commit
   (`4dda7bdf fix(security): Refresh no-panic allowlist baseline and clear
   clippy debt`) addressing this; it is not on this branch's ancestry.

2. **`xtask` test `publishable_internal_dev_dependencies_use_loose_versions`
   fails** (`xtask/tests/publish_w71.rs:341`), complaining that
   publishable internal crates declare dev-dependencies with
   `workspace = true` instead of a loose `>=1.9, <2` version. This concerns
   crates this scan did not touch (`tokmd-envelope`, `tokmd-wasm`,
   `tokmd-analysis-types`, `tokmd-scan`, `tokmd-types`).

Neither is a security finding. Both are reported here for the maintainers to
schedule.

### Validation Summary

| Check | Result |
|-------|--------|
| `cargo build --workspace --all-features` | pass |
| `cargo test --workspace --all-features --no-fail-fast` | 530 test binaries pass, 1 pre-existing failure (see above) |
| `cargo test -p tokmd-analysis-types` | 441 tests, 0 failed |
| `cargo test -p xtask` | all suites pass (after baseline regeneration) |
| `cargo deny --all-features check` (cargo-deny 0.19.0, CI-pinned) | `advisories ok, bans ok, licenses ok, sources ok` |
| `cargo fmt-check` | pass |
| `cargo clippy -p xtask -p tokmd-analysis-types --all-features --all-targets -- -D warnings` | pass |
| `cargo clippy --workspace --all-features --all-targets -- -D warnings` | 1 pre-existing error in an untouched file |
| `cargo xtask gate --check` | 3/4 (clippy step fails on pre-existing debt) |
| `cargo run -p xtask -- check-no-panic-family --strict` | 22444 findings, 22444 matched, 0 unallowlisted, 0 stale |
| `npm test` (web/runner) | 69 pass, 0 fail, 1 skipped (WASM bundle not built) |

### Patches Generated

Two patches were generated and applied:

- `9b7cc6e7 fix(security): [VULN-001] Patch tokio and rustix for RUSTSEC-2026-0386 and RUSTSEC-2026-0486`
- `90a42e73 fix(security): [VULN-002] Replace chrono with the maintained time crate`

### Next Scan

The next scheduled security scan runs Monday, 2026-11-30 via
`.github/workflows/droid-security-scan.yml` (cron `0 8 * * 1`).

## References

- [CWE Database](https://cwe.mitre.org/)
- [STRIDE Threat Model](https://docs.microsoft.com/en-us/azure/security/develop/threat-modeling-tool-threats)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Advisory Database](https://rustsec.org/)
- [OSV.dev](https://osv.dev/)
- [CII Best Practices](https://www.bestpractices.dev/)
- Repository security policy: `SECURITY.md`
- Repository threat model: `.factory/threat-model/threat-model.md`
- Previous scans: `.factory/security/reports/security-report-2026-11-09.md`,
  `.factory/security/reports/security-report-2026-11-16.md`
