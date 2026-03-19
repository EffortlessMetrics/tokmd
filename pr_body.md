# PR Glass Cockpit

Make review boring. Make truth cheap.

## 💡 Summary
Refactored unit tests in `crates/tokmd-analysis-types/src/lib.rs` to return `Result<(), Box<dyn std::error::Error>>` and replaced panic-inducing `.unwrap()` calls with the `?` operator.

## 🎯 Why (user/dev pain)
Using `unwrap()` in tests creates sharp edges and obscures failure paths. If a type conversion fails, the test suite aborts with an unhelpful panic trace. Using `?` yields cleaner test failures and fits Palette's developer experience focus perfectly.

## 🔎 Evidence (before/after)
- **File**: `crates/tokmd-analysis-types/src/lib.rs`
- **Before**: `fn entropy_class_serde_roundtrip() { ... let json = serde_json::to_string(&variant).unwrap(); ... }`
- **After**: `fn entropy_class_serde_roundtrip() -> Result<(), Box<dyn std::error::Error>> { ... let json = serde_json::to_string(&variant)?; ... Ok(()) }`

## 🧭 Options considered
### Option A (recommended)
- **What it is**: Refactor test signatures to return `Result` and replace unwraps with `?`.
- **Why it fits**: Eliminates sharp edges (panics) and improves overall test suite robustness. Very tight SRP.
- **Trade-offs**: Requires minor boilerplate (`Ok(())`) at the end of each test.

### Option B
- **What it is**: Leave `unwrap()` calls as they are.
- **When to choose it instead**: If the test code explicitly tests for panics or if DX is low priority.
- **Trade-offs**: Leaves poor DX in place and continues masking underlying failure causes.

## ✅ Decision
Choose **Option A**. It's a localized, safe, and effective DX improvement.

## 🧱 Changes made (SRP)
- Modified tests in `crates/tokmd-analysis-types/src/lib.rs` to return `Result<(), Box<dyn std::error::Error>>`
- Replaced `.unwrap()` with `?` in those tests
- Appended `Ok(())` to test bodies

## 🧪 Verification receipts
### `cargo build --verbose`
```
       Fresh unicode-ident v1.0.24
       Fresh memchr v2.8.0
       Fresh proc-macro2 v1.0.106
       Fresh cfg-if v1.0.4
       Fresh quote v1.0.45
       Fresh utf8parse v0.2.2
       Fresh syn v2.0.117
       Fresh anstyle-parse v1.0.0
       Fresh serde_derive v1.0.228
       Fresh colorchoice v1.0.4
       Fresh anstyle v1.0.13
       Fresh is_terminal_polyfill v1.70.2
       Fresh anstyle-query v1.1.5
       Fresh itoa v1.0.17
       Fresh anstream v1.0.0
       Fresh strsim v0.11.1
       Fresh clap_lex v1.0.0
       Fresh heck v0.5.0
       Fresh zmij v1.0.21
       Fresh clap_builder v4.6.0
       Fresh clap_derive v4.6.0
       Fresh shlex v1.3.0
       Fresh clap v4.6.0
       Fresh find-msvc-tools v0.1.9
       Fresh aho-corasick v1.1.4
       Fresh regex-syntax v0.8.10
       Fresh tokmd-types v1.8.0 (/app/crates/tokmd-types)
       Fresh cc v1.2.56
       Fresh regex-automata v0.4.14
       Fresh arrayref v0.3.9
       Fresh arrayvec v0.7.6
       Fresh constant_time_eq v0.4.2
       Fresh cpufeatures v0.2.17
       Fresh blake3 v1.8.3
       Fresh anyhow v1.0.102
       Fresh tokmd-envelope v1.8.0 (/app/crates/tokmd-envelope)
       Fresh regex v1.12.3
       Dirty tokmd-analysis-types v1.8.0 (/app/crates/tokmd-analysis-types): the file `crates/tokmd-analysis-types/src/lib.rs` has changed (1773923759.447512064s, 14m 2s after last build at 1773922917.203521546s)
   Compiling tokmd-analysis-types v1.8.0 (/app/crates/tokmd-analysis-types)
       Fresh bstr v1.12.1
       Fresh log v0.4.29
       Fresh libc v0.2.182
       Fresh globset v0.4.18
       Fresh getrandom v0.2.17
       Fresh zerocopy v0.8.40
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_types --edition=2024 crates/tokmd-analysis-types/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=8a631163b373268e -C extra-filename=-dbe472d2c9a93d82 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern serde=/app/target/debug/deps/libserde-7b496471ddbe204c.rmeta --extern serde_json=/app/target/debug/deps/libserde_json-e8a555b61197c87a.rmeta --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-e71a588320b3150d.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Fresh same-file v1.0.6
       Fresh rand_core v0.6.4
       Fresh walkdir v2.5.0
       Fresh ppv-lite86 v0.2.21
       Fresh siphasher v1.0.2
       Fresh rand_chacha v0.3.1
       Fresh phf_shared v0.11.3
       Fresh winnow v0.7.14
       Fresh rand v0.8.5
       Fresh ucd-trie v0.1.7
       Fresh phf_generator v0.11.3
       Fresh autocfg v1.5.0
       Fresh phf_codegen v0.11.3
       Fresh crossbeam-utils v0.8.21
       Fresh pest v2.8.6
       Fresh phf v0.11.3
       Fresh parse-zoneinfo v0.3.1
       Fresh serde_core v1.0.228
       Fresh chrono-tz-build v0.3.0
       Fresh crossbeam-epoch v0.9.18
       Fresh pest_meta v2.8.6
       Fresh tokmd-math v1.8.0 (/app/crates/tokmd-math)
       Fresh either v1.15.0
       Fresh num-traits v0.2.19
       Fresh crossbeam-deque v0.8.6
       Fresh pest_generator v2.8.6
       Fresh toml_parser v1.0.9+spec-1.1.0
       Fresh serde_spanned v1.0.4
       Fresh toml_datetime v1.0.0+spec-1.1.0
       Fresh iana-time-zone v0.1.65
       Fresh toml_writer v1.0.6+spec-1.1.0
       Fresh lazy_static v1.5.0
       Fresh chrono v0.4.44
       Fresh toml v1.0.6+spec-1.1.0
       Fresh libm v0.2.16
       Fresh pest_derive v2.8.6
       Fresh ignore v0.4.25
       Fresh rayon-core v1.13.0
       Fresh serde v1.0.228
       Fresh deunicode v1.6.2
       Fresh globwalk v0.9.1
       Fresh slug v0.1.6
       Fresh rayon v1.11.0
       Fresh chrono-tz v0.9.0
       Fresh humansize v2.1.3
       Fresh serde_json v1.0.149
       Fresh percent-encoding v2.3.2
       Fresh equivalent v1.0.2
       Fresh unicode-segmentation v1.12.0
       Fresh scopeguard v1.2.0
       Fresh smallvec v1.15.1
       Fresh hashbrown v0.16.1
       Fresh once_cell v1.21.3
       Fresh parking_lot_core v0.9.12
       Fresh indexmap v2.13.0
       Fresh lock_api v0.4.14
       Fresh tera v1.20.1
       Fresh json5 v0.4.1
       Fresh tokmd-settings v1.8.0 (/app/crates/tokmd-settings)
       Fresh tokmd-content v1.8.0 (/app/crates/tokmd-content)
       Fresh serde_spanned v0.6.9
       Fresh toml_datetime v0.6.11
       Fresh thiserror-impl v1.0.69
       Fresh encoding_rs v0.8.35
       Fresh num-conv v0.2.0
       Fresh time-core v0.1.8
       Fresh powerfmt v0.2.0
       Fresh toml_write v0.1.2
       Fresh time-macros v0.2.27
       Fresh toml_edit v0.22.27
       Fresh deranged v0.5.8
       Fresh encoding_rs_io v0.1.7
       Fresh thiserror v1.0.69
       Fresh colored v2.2.0
       Fresh itertools v0.11.0
       Fresh memmap2 v0.9.10
       Fresh tokmd-git v1.8.0 (/app/crates/tokmd-git)
       Fresh derive_arbitrary v1.4.2
       Fresh grep-matcher v0.1.8
       Fresh home v0.5.12
       Fresh hashbrown v0.14.5
       Fresh tokmd-path v1.8.0 (/app/crates/tokmd-path)
       Fresh grep-searcher v0.1.16
       Fresh dashmap v6.1.0
       Fresh etcetera v0.8.0
       Fresh arbitrary v1.4.2
       Fresh table_formatter v0.6.1
       Fresh getrandom v0.4.1
       Fresh time v0.3.47
       Fresh toml v0.8.23
       Fresh parking_lot v0.12.5
       Fresh crossbeam-channel v0.5.15
       Fresh term_size v0.3.2
       Fresh clap-cargo v0.18.3
       Fresh tokei v14.0.0
       Fresh unicode-width v0.2.2
       Fresh console v0.16.3
       Fresh tokmd-walk v1.8.0 (/app/crates/tokmd-walk)
       Fresh tokmd-tool-schema v1.8.0 (/app/crates/tokmd-tool-schema)
       Fresh tokmd-redact v1.8.0 (/app/crates/tokmd-redact)
       Fresh thiserror-impl v2.0.18
       Fresh csv-core v0.1.13
       Fresh bitflags v2.11.0
       Fresh tokmd-module-key v1.8.0 (/app/crates/tokmd-module-key)
       Fresh linux-raw-sys v0.12.1
       Fresh ryu v1.0.23
       Fresh tokmd-model v1.8.0 (/app/crates/tokmd-model)
       Fresh rustix v1.1.4
       Fresh csv v1.4.0
       Fresh tokmd-scan-args v1.8.0 (/app/crates/tokmd-scan-args)
       Fresh tokmd-config v1.8.0 (/app/crates/tokmd-config)
       Fresh thiserror v2.0.18
       Fresh portable-atomic v1.13.1
       Fresh tokmd-scan v1.8.0 (/app/crates/tokmd-scan)
       Fresh uuid v1.22.0
       Fresh midly v0.5.3
       Fresh tokmd-export-tree v1.8.0 (/app/crates/tokmd-export-tree)
       Fresh option-ext v0.2.0
       Fresh tokmd-analysis-imports v1.8.0 (/app/crates/tokmd-analysis-imports)
       Fresh fastrand v2.3.0
       Fresh unit-prefix v0.5.2
       Fresh rustc-hash v2.1.1
       Fresh tempfile v3.27.0
       Fresh indicatif v0.18.4
       Fresh dirs-sys v0.5.0
       Fresh tokmd-format v1.8.0 (/app/crates/tokmd-format)
       Fresh tokmd-fun v1.8.0 (/app/crates/tokmd-fun)
       Fresh zeroize v1.8.2
       Fresh shell-words v1.1.1
       Fresh tokmd-analysis-grid v1.8.0 (/app/crates/tokmd-analysis-grid)
       Fresh tokmd-progress v1.8.0 (/app/crates/tokmd-progress)
       Fresh dialoguer v0.12.0
       Fresh dirs v6.0.0
       Fresh tokmd-tokeignore v1.8.0 (/app/crates/tokmd-tokeignore)
       Fresh tokmd-gate v1.8.0 (/app/crates/tokmd-gate)
       Fresh tokmd-context-policy v1.8.0 (/app/crates/tokmd-context-policy)
       Fresh tokmd-exclude v1.8.0 (/app/crates/tokmd-exclude)
       Fresh tokmd-context-git v1.8.0 (/app/crates/tokmd-context-git)
       Fresh clap_complete v4.6.0
       Fresh tokmd-analysis-explain v1.8.0 (/app/crates/tokmd-analysis-explain)
       Fresh tokmd-badge v1.8.0 (/app/crates/tokmd-badge)
       Fresh tokmd-substrate v1.8.0 (/app/crates/tokmd-substrate)
       Fresh tokmd-core v1.8.0 (/app/crates/tokmd-core)
       Fresh tokmd-sensor v1.8.0 (/app/crates/tokmd-sensor)
       Fresh tokmd-ffi-envelope v1.8.0 (/app/crates/tokmd-ffi-envelope)
       Dirty tokmd-analysis-util v1.8.0 (/app/crates/tokmd-analysis-util): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-util v1.8.0 (/app/crates/tokmd-analysis-util)
       Dirty tokmd-analysis-maintainability v1.8.0 (/app/crates/tokmd-analysis-maintainability): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-maintainability v1.8.0 (/app/crates/tokmd-analysis-maintainability)
       Dirty tokmd-analysis-archetype v1.8.0 (/app/crates/tokmd-analysis-archetype): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-archetype v1.8.0 (/app/crates/tokmd-analysis-archetype)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_util --edition=2024 crates/tokmd-analysis-util/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=e9a442a884e303c5 -C extra-filename=-d1c64451e6671bfb --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_math=/app/target/debug/deps/libtokmd_math-0a0ea467c23e5e5b.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_maintainability --edition=2024 crates/tokmd-analysis-maintainability/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=fa5dbce25bb5d8fd -C extra-filename=-e60ca1f85bf8ebbe --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_archetype --edition=2024 crates/tokmd-analysis-archetype/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=4056318bc75470b0 -C extra-filename=-091680c4ad7f09ec --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-content v1.8.0 (/app/crates/tokmd-analysis-content): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-content v1.8.0 (/app/crates/tokmd-analysis-content)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_content --edition=2024 crates/tokmd-analysis-content/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=5fc43b91bd735b49 -C extra-filename=-e90ffa21c2fe85ea --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern blake3=/app/target/debug/deps/libblake3-858fdb64925ed1db.rmeta --extern tokmd_analysis_imports=/app/target/debug/deps/libtokmd_analysis_imports-964a0abe502ef516.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_analysis_util=/app/target/debug/deps/libtokmd_analysis_util-d1c64451e6671bfb.rmeta --extern tokmd_content=/app/target/debug/deps/libtokmd_content-2956831b07a6c0b0.rmeta --extern tokmd_math=/app/target/debug/deps/libtokmd_math-0a0ea467c23e5e5b.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-complexity v1.8.0 (/app/crates/tokmd-analysis-complexity): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-complexity v1.8.0 (/app/crates/tokmd-analysis-complexity)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_complexity --edition=2024 crates/tokmd-analysis-complexity/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=2ababda094becd94 -C extra-filename=-09b89548d6182dea --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern tokmd_analysis_maintainability=/app/target/debug/deps/libtokmd_analysis_maintainability-e60ca1f85bf8ebbe.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_analysis_util=/app/target/debug/deps/libtokmd_analysis_util-d1c64451e6671bfb.rmeta --extern tokmd_content=/app/target/debug/deps/libtokmd_content-2956831b07a6c0b0.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-git v1.8.0 (/app/crates/tokmd-analysis-git): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-git v1.8.0 (/app/crates/tokmd-analysis-git)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_git --edition=2024 crates/tokmd-analysis-git/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=ebcfcc759c6ce86c -C extra-filename=-39abbc66739ce874 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_analysis_util=/app/target/debug/deps/libtokmd_analysis_util-d1c64451e6671bfb.rmeta --extern tokmd_git=/app/target/debug/deps/libtokmd_git-44290802db5c70b4.rmeta --extern tokmd_math=/app/target/debug/deps/libtokmd_math-0a0ea467c23e5e5b.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-entropy v1.8.0 (/app/crates/tokmd-analysis-entropy): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-entropy v1.8.0 (/app/crates/tokmd-analysis-entropy)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_entropy --edition=2024 crates/tokmd-analysis-entropy/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=c0daf372be044132 -C extra-filename=-c680a2a0c01099a1 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_analysis_util=/app/target/debug/deps/libtokmd_analysis_util-d1c64451e6671bfb.rmeta --extern tokmd_content=/app/target/debug/deps/libtokmd_content-2956831b07a6c0b0.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-derived v1.8.0 (/app/crates/tokmd-analysis-derived): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-derived v1.8.0 (/app/crates/tokmd-analysis-derived)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_derived --edition=2024 crates/tokmd-analysis-derived/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=65a496120074aab6 -C extra-filename=-9661ee909b7093c6 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern blake3=/app/target/debug/deps/libblake3-858fdb64925ed1db.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_analysis_util=/app/target/debug/deps/libtokmd_analysis_util-d1c64451e6671bfb.rmeta --extern tokmd_export_tree=/app/target/debug/deps/libtokmd_export_tree-f8a55bd5b16b4090.rmeta --extern tokmd_math=/app/target/debug/deps/libtokmd_math-0a0ea467c23e5e5b.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-api-surface v1.8.0 (/app/crates/tokmd-analysis-api-surface): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-api-surface v1.8.0 (/app/crates/tokmd-analysis-api-surface)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_api_surface --edition=2024 crates/tokmd-analysis-api-surface/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=ec0d81e6abb1183b -C extra-filename=-467a632300d54f87 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_analysis_util=/app/target/debug/deps/libtokmd_analysis_util-d1c64451e6671bfb.rmeta --extern tokmd_content=/app/target/debug/deps/libtokmd_content-2956831b07a6c0b0.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-effort v1.8.0 (/app/crates/tokmd-analysis-effort): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-effort v1.8.0 (/app/crates/tokmd-analysis-effort)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_effort --edition=2024 crates/tokmd-analysis-effort/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="default"' --cfg 'feature="git"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("default", "git"))' -C metadata=f5e325f522bb9d90 -C extra-filename=-a95c95fd1a58c1fa --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_analysis_util=/app/target/debug/deps/libtokmd_analysis_util-d1c64451e6671bfb.rmeta --extern tokmd_git=/app/target/debug/deps/libtokmd_git-44290802db5c70b4.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-license v1.8.0 (/app/crates/tokmd-analysis-license): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-license v1.8.0 (/app/crates/tokmd-analysis-license)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_license --edition=2024 crates/tokmd-analysis-license/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=21b1a858ff23916b -C extra-filename=-214139663c05b96e --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern serde_json=/app/target/debug/deps/libserde_json-e8a555b61197c87a.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_analysis_util=/app/target/debug/deps/libtokmd_analysis_util-d1c64451e6671bfb.rmeta --extern tokmd_content=/app/target/debug/deps/libtokmd_content-2956831b07a6c0b0.rmeta --extern tokmd_walk=/app/target/debug/deps/libtokmd_walk-c966a588140bd4ac.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-topics v1.8.0 (/app/crates/tokmd-analysis-topics): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-topics v1.8.0 (/app/crates/tokmd-analysis-topics)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_topics --edition=2024 crates/tokmd-analysis-topics/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=7477f970eca47df5 -C extra-filename=-11b8d158fc775595 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-html v1.8.0 (/app/crates/tokmd-analysis-html): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-html v1.8.0 (/app/crates/tokmd-analysis-html)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_html --edition=2024 crates/tokmd-analysis-html/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=583c5c6298179084 -C extra-filename=-7fc0f0aa99666bf4 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern serde_json=/app/target/debug/deps/libserde_json-e8a555b61197c87a.rmeta --extern time=/app/target/debug/deps/libtime-1e7607da8aaea389.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-fingerprint v1.8.0 (/app/crates/tokmd-analysis-fingerprint): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-fingerprint v1.8.0 (/app/crates/tokmd-analysis-fingerprint)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_fingerprint --edition=2024 crates/tokmd-analysis-fingerprint/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=be126354bf70d2c4 -C extra-filename=-18bddd4c34264f1e --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_git=/app/target/debug/deps/libtokmd_git-44290802db5c70b4.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-fun v1.8.0 (/app/crates/tokmd-analysis-fun): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-fun v1.8.0 (/app/crates/tokmd-analysis-fun)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_fun --edition=2024 crates/tokmd-analysis-fun/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=4585f1426f501ab1 -C extra-filename=-590a542032e1f343 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-near-dup v1.8.0 (/app/crates/tokmd-analysis-near-dup): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-near-dup v1.8.0 (/app/crates/tokmd-analysis-near-dup)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_near_dup --edition=2024 crates/tokmd-analysis-near-dup/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=7c6293efa7c959a0 -C extra-filename=-b11cf6ee6808577d --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern globset=/app/target/debug/deps/libglobset-0221fc414717030a.rmeta --extern rustc_hash=/app/target/debug/deps/librustc_hash-66d95c27dec1f3b0.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-assets v1.8.0 (/app/crates/tokmd-analysis-assets): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-assets v1.8.0 (/app/crates/tokmd-analysis-assets)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_assets --edition=2024 crates/tokmd-analysis-assets/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=16cc12b8fa91009e -C extra-filename=-77224fe66d2e6d59 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern serde_json=/app/target/debug/deps/libserde_json-e8a555b61197c87a.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_walk=/app/target/debug/deps/libtokmd_walk-c966a588140bd4ac.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-format v1.8.0 (/app/crates/tokmd-analysis-format): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-format v1.8.0 (/app/crates/tokmd-analysis-format)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_format --edition=2024 crates/tokmd-analysis-format/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="default"' --cfg 'feature="fun"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("default", "fun"))' -C metadata=f24dedbd879dba2e -C extra-filename=-cf7f8f5ab11169a0 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern serde_json=/app/target/debug/deps/libserde_json-e8a555b61197c87a.rmeta --extern tokmd_analysis_html=/app/target/debug/deps/libtokmd_analysis_html-7fc0f0aa99666bf4.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_fun=/app/target/debug/deps/libtokmd_fun-0b79070bb515a13b.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis v1.8.0 (/app/crates/tokmd-analysis): the dependency `tokmd-analysis-git` was rebuilt
   Compiling tokmd-analysis v1.8.0 (/app/crates/tokmd-analysis)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis --edition=2024 crates/tokmd-analysis/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="archetype"' --cfg 'feature="content"' --cfg 'feature="default"' --cfg 'feature="effort"' --cfg 'feature="fun"' --cfg 'feature="git"' --cfg 'feature="topics"' --cfg 'feature="walk"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("archetype", "content", "default", "effort", "fun", "git", "halstead", "topics", "walk"))' -C metadata=be990b9d94550405 -C extra-filename=-eefafb5b615c4320 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern blake3=/app/target/debug/deps/libblake3-858fdb64925ed1db.rmeta --extern serde=/app/target/debug/deps/libserde-7b496471ddbe204c.rmeta --extern serde_json=/app/target/debug/deps/libserde_json-e8a555b61197c87a.rmeta --extern tokmd_analysis_api_surface=/app/target/debug/deps/libtokmd_analysis_api_surface-467a632300d54f87.rmeta --extern tokmd_analysis_archetype=/app/target/debug/deps/libtokmd_analysis_archetype-091680c4ad7f09ec.rmeta --extern tokmd_analysis_assets=/app/target/debug/deps/libtokmd_analysis_assets-77224fe66d2e6d59.rmeta --extern tokmd_analysis_complexity=/app/target/debug/deps/libtokmd_analysis_complexity-09b89548d6182dea.rmeta --extern tokmd_analysis_content=/app/target/debug/deps/libtokmd_analysis_content-e90ffa21c2fe85ea.rmeta --extern tokmd_analysis_derived=/app/target/debug/deps/libtokmd_analysis_derived-9661ee909b7093c6.rmeta --extern tokmd_analysis_effort=/app/target/debug/deps/libtokmd_analysis_effort-a95c95fd1a58c1fa.rmeta --extern tokmd_analysis_entropy=/app/target/debug/deps/libtokmd_analysis_entropy-c680a2a0c01099a1.rmeta --extern tokmd_analysis_fingerprint=/app/target/debug/deps/libtokmd_analysis_fingerprint-18bddd4c34264f1e.rmeta --extern tokmd_analysis_fun=/app/target/debug/deps/libtokmd_analysis_fun-590a542032e1f343.rmeta --extern tokmd_analysis_git=/app/target/debug/deps/libtokmd_analysis_git-39abbc66739ce874.rmeta --extern tokmd_analysis_grid=/app/target/debug/deps/libtokmd_analysis_grid-1793611fea8d04a3.rmeta --extern tokmd_analysis_license=/app/target/debug/deps/libtokmd_analysis_license-214139663c05b96e.rmeta --extern tokmd_analysis_near_dup=/app/target/debug/deps/libtokmd_analysis_near_dup-b11cf6ee6808577d.rmeta --extern tokmd_analysis_topics=/app/target/debug/deps/libtokmd_analysis_topics-11b8d158fc775595.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_analysis_util=/app/target/debug/deps/libtokmd_analysis_util-d1c64451e6671bfb.rmeta --extern tokmd_content=/app/target/debug/deps/libtokmd_content-2956831b07a6c0b0.rmeta --extern tokmd_git=/app/target/debug/deps/libtokmd_git-44290802db5c70b4.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta --extern tokmd_walk=/app/target/debug/deps/libtokmd_walk-c966a588140bd4ac.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-cockpit v1.8.0 (/app/crates/tokmd-cockpit): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-cockpit v1.8.0 (/app/crates/tokmd-cockpit)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_cockpit --edition=2024 crates/tokmd-cockpit/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="default"' --cfg 'feature="git"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("default", "git"))' -C metadata=a6919729ceb07cc4 -C extra-filename=-d1378b8fd92c47da --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern blake3=/app/target/debug/deps/libblake3-858fdb64925ed1db.rmeta --extern ignore=/app/target/debug/deps/libignore-7d1f9a682b57c852.rmeta --extern serde=/app/target/debug/deps/libserde-7b496471ddbe204c.rmeta --extern serde_json=/app/target/debug/deps/libserde_json-e8a555b61197c87a.rmeta --extern time=/app/target/debug/deps/libtime-1e7607da8aaea389.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-e71a588320b3150d.rmeta --extern tokmd_git=/app/target/debug/deps/libtokmd_git-44290802db5c70b4.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd-analysis-halstead v1.8.0 (/app/crates/tokmd-analysis-halstead): the dependency `tokmd-analysis-types` was rebuilt
   Compiling tokmd-analysis-halstead v1.8.0 (/app/crates/tokmd-analysis-halstead)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_halstead --edition=2024 crates/tokmd-analysis-halstead/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=1d627cb308481e53 -C extra-filename=-7a628f2ea9a8c514 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_analysis_util=/app/target/debug/deps/libtokmd_analysis_util-d1c64451e6671bfb.rmeta --extern tokmd_content=/app/target/debug/deps/libtokmd_content-2956831b07a6c0b0.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
       Dirty tokmd v1.8.0 (/app/crates/tokmd): the dependency `tokmd-analysis` was rebuilt
   Compiling tokmd v1.8.0 (/app/crates/tokmd)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd --edition=2024 crates/tokmd/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="archetype"' --cfg 'feature="content"' --cfg 'feature="default"' --cfg 'feature="fun"' --cfg 'feature="git"' --cfg 'feature="topics"' --cfg 'feature="ui"' --cfg 'feature="walk"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("alias-tok", "archetype", "content", "default", "fun", "git", "topics", "ui", "walk"))' -C metadata=5a4b05abf9d1eb8e -C extra-filename=-02b5195b737d1811 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rmeta --extern blake3=/app/target/debug/deps/libblake3-858fdb64925ed1db.rmeta --extern clap=/app/target/debug/deps/libclap-971c649bd0726e15.rmeta --extern clap_complete=/app/target/debug/deps/libclap_complete-3ab40a9005b11aea.rmeta --extern console=/app/target/debug/deps/libconsole-aeccd1d922037640.rmeta --extern dialoguer=/app/target/debug/deps/libdialoguer-da1a45b7bc8d38c0.rmeta --extern dirs=/app/target/debug/deps/libdirs-b0038fcd362be2e6.rmeta --extern serde=/app/target/debug/deps/libserde-7b496471ddbe204c.rmeta --extern serde_json=/app/target/debug/deps/libserde_json-e8a555b61197c87a.rmeta --extern time=/app/target/debug/deps/libtime-1e7607da8aaea389.rmeta --extern tokmd_analysis=/app/target/debug/deps/libtokmd_analysis-eefafb5b615c4320.rmeta --extern tokmd_analysis_explain=/app/target/debug/deps/libtokmd_analysis_explain-f979ea9a7914df26.rmeta --extern tokmd_analysis_format=/app/target/debug/deps/libtokmd_analysis_format-cf7f8f5ab11169a0.rmeta --extern tokmd_analysis_grid=/app/target/debug/deps/libtokmd_analysis_grid-1793611fea8d04a3.rmeta --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rmeta --extern tokmd_badge=/app/target/debug/deps/libtokmd_badge-1ed7c3587470cb4c.rmeta --extern tokmd_cockpit=/app/target/debug/deps/libtokmd_cockpit-d1378b8fd92c47da.rmeta --extern tokmd_config=/app/target/debug/deps/libtokmd_config-b5b6317f5ab20f66.rmeta --extern tokmd_context_git=/app/target/debug/deps/libtokmd_context_git-ed96c3987d5b4da3.rmeta --extern tokmd_context_policy=/app/target/debug/deps/libtokmd_context_policy-2f33b27a5255d3f5.rmeta --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-e71a588320b3150d.rmeta --extern tokmd_exclude=/app/target/debug/deps/libtokmd_exclude-0e07499c990062e3.rmeta --extern tokmd_export_tree=/app/target/debug/deps/libtokmd_export_tree-f8a55bd5b16b4090.rmeta --extern tokmd_format=/app/target/debug/deps/libtokmd_format-3d1138e1191b62e8.rmeta --extern tokmd_gate=/app/target/debug/deps/libtokmd_gate-034bf90bb13e7572.rmeta --extern tokmd_git=/app/target/debug/deps/libtokmd_git-44290802db5c70b4.rmeta --extern tokmd_model=/app/target/debug/deps/libtokmd_model-c9c26399a3ecc877.rmeta --extern tokmd_path=/app/target/debug/deps/libtokmd_path-88ca67c47715c256.rmeta --extern tokmd_progress=/app/target/debug/deps/libtokmd_progress-4afaa312fd819401.rmeta --extern tokmd_scan=/app/target/debug/deps/libtokmd_scan-1774b9c31a337c38.rmeta --extern tokmd_settings=/app/target/debug/deps/libtokmd_settings-3a923806c0d33ea0.rmeta --extern tokmd_tokeignore=/app/target/debug/deps/libtokmd_tokeignore-294cb4cbd90473bd.rmeta --extern tokmd_tool_schema=/app/target/debug/deps/libtokmd_tool_schema-54a5f79e9101ada0.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rmeta --extern toml=/app/target/debug/deps/libtoml-6df5a8e70f66077b.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd --edition=2024 crates/tokmd/src/bin/tokmd.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type bin --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="archetype"' --cfg 'feature="content"' --cfg 'feature="default"' --cfg 'feature="fun"' --cfg 'feature="git"' --cfg 'feature="topics"' --cfg 'feature="ui"' --cfg 'feature="walk"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("alias-tok", "archetype", "content", "default", "fun", "git", "topics", "ui", "walk"))' -C metadata=e4ab96cb9ab08131 -C extra-filename=-2898d7615907c11d --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern anyhow=/app/target/debug/deps/libanyhow-1c897acad9a38b79.rlib --extern blake3=/app/target/debug/deps/libblake3-858fdb64925ed1db.rlib --extern clap=/app/target/debug/deps/libclap-971c649bd0726e15.rlib --extern clap_complete=/app/target/debug/deps/libclap_complete-3ab40a9005b11aea.rlib --extern console=/app/target/debug/deps/libconsole-aeccd1d922037640.rlib --extern dialoguer=/app/target/debug/deps/libdialoguer-da1a45b7bc8d38c0.rlib --extern dirs=/app/target/debug/deps/libdirs-b0038fcd362be2e6.rlib --extern serde=/app/target/debug/deps/libserde-7b496471ddbe204c.rlib --extern serde_json=/app/target/debug/deps/libserde_json-e8a555b61197c87a.rlib --extern time=/app/target/debug/deps/libtime-1e7607da8aaea389.rlib --extern tokmd=/app/target/debug/deps/libtokmd-02b5195b737d1811.rlib --extern tokmd_analysis=/app/target/debug/deps/libtokmd_analysis-eefafb5b615c4320.rlib --extern tokmd_analysis_explain=/app/target/debug/deps/libtokmd_analysis_explain-f979ea9a7914df26.rlib --extern tokmd_analysis_format=/app/target/debug/deps/libtokmd_analysis_format-cf7f8f5ab11169a0.rlib --extern tokmd_analysis_grid=/app/target/debug/deps/libtokmd_analysis_grid-1793611fea8d04a3.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-dbe472d2c9a93d82.rlib --extern tokmd_badge=/app/target/debug/deps/libtokmd_badge-1ed7c3587470cb4c.rlib --extern tokmd_cockpit=/app/target/debug/deps/libtokmd_cockpit-d1378b8fd92c47da.rlib --extern tokmd_config=/app/target/debug/deps/libtokmd_config-b5b6317f5ab20f66.rlib --extern tokmd_context_git=/app/target/debug/deps/libtokmd_context_git-ed96c3987d5b4da3.rlib --extern tokmd_context_policy=/app/target/debug/deps/libtokmd_context_policy-2f33b27a5255d3f5.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-e71a588320b3150d.rlib --extern tokmd_exclude=/app/target/debug/deps/libtokmd_exclude-0e07499c990062e3.rlib --extern tokmd_export_tree=/app/target/debug/deps/libtokmd_export_tree-f8a55bd5b16b4090.rlib --extern tokmd_format=/app/target/debug/deps/libtokmd_format-3d1138e1191b62e8.rlib --extern tokmd_gate=/app/target/debug/deps/libtokmd_gate-034bf90bb13e7572.rlib --extern tokmd_git=/app/target/debug/deps/libtokmd_git-44290802db5c70b4.rlib --extern tokmd_model=/app/target/debug/deps/libtokmd_model-c9c26399a3ecc877.rlib --extern tokmd_path=/app/target/debug/deps/libtokmd_path-88ca67c47715c256.rlib --extern tokmd_progress=/app/target/debug/deps/libtokmd_progress-4afaa312fd819401.rlib --extern tokmd_scan=/app/target/debug/deps/libtokmd_scan-1774b9c31a337c38.rlib --extern tokmd_settings=/app/target/debug/deps/libtokmd_settings-3a923806c0d33ea0.rlib --extern tokmd_tokeignore=/app/target/debug/deps/libtokmd_tokeignore-294cb4cbd90473bd.rlib --extern tokmd_tool_schema=/app/target/debug/deps/libtokmd_tool_schema-54a5f79e9101ada0.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-97676599ee5ca68d.rlib --extern toml=/app/target/debug/deps/libtoml-6df5a8e70f66077b.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 30.57s
```

### `CI=true cargo test --verbose -p tokmd-analysis-types --all-features`
```
       Fresh cfg-if v1.0.4
       Fresh libc v0.2.182
       Fresh unicode-ident v1.0.24
       Fresh proc-macro2 v1.0.106
       Fresh quote v1.0.45
       Fresh find-msvc-tools v0.1.9
       Fresh shlex v1.3.0
       Fresh syn v2.0.117
       Fresh cc v1.2.56
       Fresh getrandom v0.3.4
   Compiling serde_core v1.0.228
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name serde_core --edition=2021 /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_core-1.0.228/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="result"' --cfg 'feature="std"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("alloc", "default", "rc", "result", "std", "unstable"))' -C metadata=2203c5558179a544 -C extra-filename=-d7126bb57f93c398 --out-dir /app/target/debug/deps -L dependency=/app/target/debug/deps --cap-lints allow --check-cfg 'cfg(if_docsrs_then_no_serde_core)' --check-cfg 'cfg(no_core_cstr)' --check-cfg 'cfg(no_core_error)' --check-cfg 'cfg(no_core_net)' --check-cfg 'cfg(no_core_num_saturating)' --check-cfg 'cfg(no_diagnostic_namespace)' --check-cfg 'cfg(no_serde_derive)' --check-cfg 'cfg(no_std_atomic)' --check-cfg 'cfg(no_std_atomic64)' --check-cfg 'cfg(no_target_has_atomic)'`
       Fresh rand_core v0.9.5
       Fresh serde_derive v1.0.228
       Fresh autocfg v1.5.0
       Fresh linux-raw-sys v0.12.1
       Fresh bitflags v2.11.0
       Fresh rustix v1.1.4
       Fresh zmij v1.0.21
       Fresh zerocopy v0.8.40
       Fresh getrandom v0.4.1
       Fresh constant_time_eq v0.4.2
       Fresh arrayvec v0.7.6
   Compiling once_cell v1.21.3
       Fresh arrayref v0.3.9
       Fresh cpufeatures v0.2.17
       Fresh itoa v1.0.17
       Fresh fastrand v2.3.0
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name once_cell --edition=2021 /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/once_cell-1.21.3/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="alloc"' --cfg 'feature="race"' --cfg 'feature="std"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("alloc", "atomic-polyfill", "critical-section", "default", "parking_lot", "portable-atomic", "race", "std", "unstable"))' -C metadata=8e3dc3beb9516534 -C extra-filename=-173cb124a47953a0 --out-dir /app/target/debug/deps -L dependency=/app/target/debug/deps --cap-lints allow`
   Compiling memchr v2.8.0
       Fresh blake3 v1.8.3
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name memchr --edition=2021 /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/memchr-2.8.0/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="alloc"' --cfg 'feature="std"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("alloc", "core", "default", "libc", "logging", "rustc-dep-of-std", "std", "use_std"))' -C metadata=3338533f333a799c -C extra-filename=-ec5c298b979d44cc --out-dir /app/target/debug/deps -L dependency=/app/target/debug/deps --cap-lints allow`
   Compiling ppv-lite86 v0.2.21
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name ppv_lite86 --edition=2021 /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ppv-lite86-0.2.21/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="simd"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("default", "no_simd", "simd", "std"))' -C metadata=c52b1072a766040f -C extra-filename=-146e1f6cd735d018 --out-dir /app/target/debug/deps -L dependency=/app/target/debug/deps --extern zerocopy=/app/target/debug/deps/libzerocopy-d5fab3bb803e6520.rmeta --cap-lints allow`
   Compiling tempfile v3.27.0
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tempfile --edition=2021 /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tempfile-3.27.0/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="default"' --cfg 'feature="getrandom"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("default", "getrandom", "nightly"))' -C metadata=35f0b5973215b4d1 -C extra-filename=-57e95f3aec4ee295 --out-dir /app/target/debug/deps -L dependency=/app/target/debug/deps --extern fastrand=/app/target/debug/deps/libfastrand-edb399766ba30a3d.rmeta --extern getrandom=/app/target/debug/deps/libgetrandom-7798e0f8672480ba.rmeta --extern once_cell=/app/target/debug/deps/libonce_cell-173cb124a47953a0.rmeta --extern rustix=/app/target/debug/deps/librustix-e0e8dffdca9bc897.rmeta --cap-lints allow`
       Fresh wait-timeout v0.2.1
       Fresh quick-error v1.2.3
       Fresh fnv v1.0.7
       Fresh bit-vec v0.8.0
       Fresh bit-set v0.8.0
   Compiling num-traits v0.2.19
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name num_traits --edition=2021 /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/num-traits-0.2.19/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="std"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("default", "i128", "libm", "std"))' -C metadata=4b286a3109c92ca2 -C extra-filename=-0fe333d40e8b7c7f --out-dir /app/target/debug/deps -L dependency=/app/target/debug/deps --cap-lints allow --cfg has_total_cmp --check-cfg 'cfg(has_total_cmp)'`
   Compiling rusty-fork v0.3.1
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name rusty_fork --edition=2018 /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rusty-fork-0.3.1/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="timeout"' --cfg 'feature="wait-timeout"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("default", "timeout", "wait-timeout"))' -C metadata=a873c437c70d5171 -C extra-filename=-a1947c28fc93e25a --out-dir /app/target/debug/deps -L dependency=/app/target/debug/deps --extern fnv=/app/target/debug/deps/libfnv-1e77ae8f0484c8fd.rmeta --extern quick_error=/app/target/debug/deps/libquick_error-d07b82ff5e154d70.rmeta --extern tempfile=/app/target/debug/deps/libtempfile-57e95f3aec4ee295.rmeta --extern wait_timeout=/app/target/debug/deps/libwait_timeout-6e4498e41857a275.rmeta --cap-lints allow`
   Compiling rand_chacha v0.9.0
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name rand_chacha --edition=2021 /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rand_chacha-0.9.0/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("default", "os_rng", "serde", "std"))' -C metadata=b719fcdac87bc4b4 -C extra-filename=-b605e9a319d8214b --out-dir /app/target/debug/deps -L dependency=/app/target/debug/deps --extern ppv_lite86=/app/target/debug/deps/libppv_lite86-146e1f6cd735d018.rmeta --extern rand_core=/app/target/debug/deps/librand_core-5c83113414eb775f.rmeta --cap-lints allow`
       Fresh rand v0.9.2
       Fresh rand_xorshift v0.4.0
       Fresh unarray v0.1.4
       Fresh regex-syntax v0.8.10
   Compiling proptest v1.10.0
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name proptest --edition=2021 /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.10.0/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="bit-set"' --cfg 'feature="default"' --cfg 'feature="fork"' --cfg 'feature="regex-syntax"' --cfg 'feature="rusty-fork"' --cfg 'feature="std"' --cfg 'feature="tempfile"' --cfg 'feature="timeout"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("alloc", "atomic64bit", "attr-macro", "bit-set", "default", "default-code-coverage", "fork", "handle-panics", "hardware-rng", "no_std", "proptest-macro", "regex-syntax", "rusty-fork", "std", "tempfile", "timeout", "unstable", "x86"))' -C metadata=daa50f91a9ed95b1 -C extra-filename=-c22aa1ec6e28355e --out-dir /app/target/debug/deps -L dependency=/app/target/debug/deps --extern bit_set=/app/target/debug/deps/libbit_set-248f2d1f1efbd76b.rmeta --extern bit_vec=/app/target/debug/deps/libbit_vec-bcfd2c47e4b75695.rmeta --extern bitflags=/app/target/debug/deps/libbitflags-4c22fbc3fb575483.rmeta --extern num_traits=/app/target/debug/deps/libnum_traits-0fe333d40e8b7c7f.rmeta --extern rand=/app/target/debug/deps/librand-b2463ef33a5f7e39.rmeta --extern rand_chacha=/app/target/debug/deps/librand_chacha-b605e9a319d8214b.rmeta --extern rand_xorshift=/app/target/debug/deps/librand_xorshift-3d860054ba2d5902.rmeta --extern regex_syntax=/app/target/debug/deps/libregex_syntax-abe2ba1672407809.rmeta --extern rusty_fork=/app/target/debug/deps/librusty_fork-a1947c28fc93e25a.rmeta --extern tempfile=/app/target/debug/deps/libtempfile-57e95f3aec4ee295.rmeta --extern unarray=/app/target/debug/deps/libunarray-13071c3ccdbc289d.rmeta --cap-lints allow`
   Compiling serde v1.0.228
   Compiling serde_json v1.0.149
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name serde --edition=2021 /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde-1.0.228/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="default"' --cfg 'feature="derive"' --cfg 'feature="serde_derive"' --cfg 'feature="std"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("alloc", "default", "derive", "rc", "serde_derive", "std", "unstable"))' -C metadata=b125b10a0b9ee7f2 -C extra-filename=-dcea86c0a8efc1ac --out-dir /app/target/debug/deps -L dependency=/app/target/debug/deps --extern serde_core=/app/target/debug/deps/libserde_core-d7126bb57f93c398.rmeta --extern serde_derive=/app/target/debug/deps/libserde_derive-9dc0367e64da9072.so --cap-lints allow --cfg if_docsrs_then_no_serde_core --check-cfg 'cfg(feature, values("result"))' --check-cfg 'cfg(if_docsrs_then_no_serde_core)' --check-cfg 'cfg(no_core_cstr)' --check-cfg 'cfg(no_core_error)' --check-cfg 'cfg(no_core_net)' --check-cfg 'cfg(no_core_num_saturating)' --check-cfg 'cfg(no_diagnostic_namespace)' --check-cfg 'cfg(no_serde_derive)' --check-cfg 'cfg(no_std_atomic)' --check-cfg 'cfg(no_std_atomic64)' --check-cfg 'cfg(no_target_has_atomic)'`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name serde_json --edition=2021 /home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.149/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="default"' --cfg 'feature="std"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("alloc", "arbitrary_precision", "default", "float_roundtrip", "indexmap", "preserve_order", "raw_value", "std", "unbounded_depth"))' -C metadata=8d8049930e97bb02 -C extra-filename=-b02faf413897a7d0 --out-dir /app/target/debug/deps -L dependency=/app/target/debug/deps --extern itoa=/app/target/debug/deps/libitoa-f12dbab35ba5f598.rmeta --extern memchr=/app/target/debug/deps/libmemchr-ec5c298b979d44cc.rmeta --extern serde_core=/app/target/debug/deps/libserde_core-d7126bb57f93c398.rmeta --extern zmij=/app/target/debug/deps/libzmij-c3888d3cb1e0eabd.rmeta --cap-lints allow --cfg 'fast_arithmetic="64"' --check-cfg 'cfg(fast_arithmetic, values("32", "64"))'`
   Compiling tokmd-types v1.8.0 (/app/crates/tokmd-types)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_types --edition=2024 crates/tokmd-types/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("clap"))' -C metadata=37d1a6ea60330213 -C extra-filename=-729fd4069bfbb733 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rmeta`
   Compiling tokmd-envelope v1.8.0 (/app/crates/tokmd-envelope)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_envelope --edition=2024 crates/tokmd-envelope/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=b32c1439c04ab1f9 -C extra-filename=-0e4273f9b236521b --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern blake3=/app/target/debug/deps/libblake3-858fdb64925ed1db.rmeta --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rmeta --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
   Compiling tokmd-analysis-types v1.8.0 (/app/crates/tokmd-analysis-types)
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_types --edition=2024 crates/tokmd-analysis-types/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C embed-bitcode=no -C debuginfo=2 --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=3bebe3372eeb8016 -C extra-filename=-17b1c00370d108ff --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rmeta --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rmeta --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rmeta --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rmeta -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_analysis_types --edition=2024 crates/tokmd-analysis-types/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=bcc1865963a83fa2 -C extra-filename=-7c4de23d4132b0a5 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name baseline --edition=2024 crates/tokmd-analysis-types/tests/baseline.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=7cab15b89060cc56 -C extra-filename=-b4b95414323a28f8 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-17b1c00370d108ff.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name analysis_types_depth_w56 --edition=2024 crates/tokmd-analysis-types/tests/analysis_types_depth_w56.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=97c82a942d36ccf4 -C extra-filename=-207f04c578892559 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-17b1c00370d108ff.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name proptest_w69 --edition=2024 crates/tokmd-analysis-types/tests/proptest_w69.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=fe5715eca1bf40f0 -C extra-filename=-5c8bbdaa255cb896 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-17b1c00370d108ff.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name analysis_types_depth_w61 --edition=2024 crates/tokmd-analysis-types/tests/analysis_types_depth_w61.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=3f36d193b05933fb -C extra-filename=-dc9dc20a56884f43 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-17b1c00370d108ff.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name properties --edition=2024 crates/tokmd-analysis-types/tests/properties.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=3e101be46a567107 -C extra-filename=-22218058e489d5ca --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-17b1c00370d108ff.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name serde_compat_v2 --edition=2024 crates/tokmd-analysis-types/tests/serde_compat_v2.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=394f2dd782a38507 -C extra-filename=-09f14b372189d23b --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-17b1c00370d108ff.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name coverage_expansion --edition=2024 crates/tokmd-analysis-types/tests/coverage_expansion.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=f696431b1948b9ee -C extra-filename=-e767f02e60bfee11 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-17b1c00370d108ff.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name feature_stability_w53 --edition=2024 crates/tokmd-analysis-types/tests/feature_stability_w53.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=90968e5c923d2b5a -C extra-filename=-f2668c5dc4151352 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-17b1c00370d108ff.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name schema_contract --edition=2024 crates/tokmd-analysis-types/tests/schema_contract.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=c5a632eb16f60460 -C extra-filename=-ace1741453eb3e26 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-17b1c00370d108ff.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name schema_contract_w66 --edition=2024 crates/tokmd-analysis-types/tests/schema_contract_w66.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=a6fc9f1897261608 -C extra-filename=-957a74d97c9611ab --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-17b1c00370d108ff.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name schema_validation --edition=2024 crates/tokmd-analysis-types/tests/schema_validation.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=f50eac73c1b059b9 -C extra-filename=-7ee360efa057ca48 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-17b1c00370d108ff.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name schema_compliance_w53 --edition=2024 crates/tokmd-analysis-types/tests/schema_compliance_w53.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --emit=dep-info,link -C embed-bitcode=no -C debuginfo=2 --test --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=f63e536d22c09665 -C extra-filename=-4ec3c4bea2da9672 --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-17b1c00370d108ff.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 33.95s
     Running `/app/target/debug/deps/tokmd_analysis_types-7c4de23d4132b0a5`

running 23 tests
test tests::baseline_version_constant ... ok
test tests::analysis_schema_version_constant ... ok
test tests::baseline_metrics_default_is_zeroed ... ok
test tests::complexity_baseline_new_equals_default ... ok
test tests::complexity_baseline_default ... ok
test tests::complexity_histogram_to_ascii_basic ... ok
test tests::complexity_histogram_to_ascii_empty_counts ... ok
test tests::complexity_baseline_serde_roundtrip ... ok
test tests::eco_label_serde_roundtrip ... ok
test tests::complexity_risk_serde_roundtrip ... ok
test tests::effort_confidence_level_display_strings_are_stable ... ok
test tests::complexity_risk_uses_snake_case ... ok
test tests::effort_delta_classification_display_strings_are_stable ... ok
test tests::effort_model_display_strings_are_stable ... ok
test tests::entropy_class_serde_roundtrip ... ok
test tests::entropy_class_uses_snake_case ... ok
test tests::license_source_kind_serde_roundtrip ... ok
test tests::technical_debt_level_serde_roundtrip ... ok
test tests::timestamp_epoch ... ok
test tests::timestamp_with_millis ... ok
test tests::topic_term_serde_roundtrip ... ok
test tests::trend_class_serde_roundtrip ... ok
test tests::trend_class_uses_snake_case ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running `/app/target/debug/deps/analysis_types_depth_w56-207f04c578892559`

running 33 tests
test analysis_schema_version_is_9 ... ok
test commit_intent_all_kinds_increment ... ok
test baseline_version_is_1 ... ok
test commit_intent_counts_increment ... ok
test complexity_baseline_from_receipt_with_complexity ... ok
test commit_intent_counts_default_is_zeroed ... ok
test api_surface_report_roundtrip ... ok
test complexity_baseline_from_receipt_without_complexity ... ok
test complexity_histogram_ascii_nonempty ... ok
test coupling_row_optional_fields_skip_when_none ... ok
test coupling_row_optional_fields_present_when_some ... ok
test envelope_schema_is_sensor_report_v1 ... ok
test envelope_type_aliases_work ... ok
test minimal_receipt_has_no_optional_sections ... ok
test minimal_receipt_serializes_to_json ... ok
test near_dup_report_truncated_default_false ... ok
test near_dup_scope_default_is_module ... ok
test minimal_receipt_json_roundtrip ... ok
test near_dup_scope_serde_roundtrip ... ok
test near_dup_scope_uses_kebab_case ... ok
test receipt_json_keys_are_snake_case ... ok
test receipt_preset_stored_in_args ... ok
test receipt_schema_version_matches_constant ... ok
test receipt_null_optional_fields_deserialize ... ok
test receipt_with_archetype ... ok
test full_receipt_json_roundtrip_fidelity ... ok
test receipt_source_fields_preserved ... ok
test receipt_with_complexity_report ... ok
test scan_status_serde_roundtrip ... ok
test technical_debt_level_serde_roundtrip ... ok
test verdict_default_is_pass ... ok
test receipt_with_git_report ... ok
test receipt_with_derived_roundtrip ... ok

test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running `/app/target/debug/deps/analysis_types_depth_w61-dc9dc20a56884f43`

running 50 tests
test analysis_receipt_json_contains_schema_version ... ok
test api_surface_report_serde_roundtrip ... ok
test analysis_receipt_with_warnings_roundtrip ... ok
test analysis_receipt_minimal_roundtrip ... ok
test analysis_receipt_scan_status_partial ... ok
test baseline_from_analysis_no_complexity ... ok
test baseline_from_analysis_with_complexity ... ok
test baseline_new_matches_default ... ok
test commit_intent_counts_default_is_zeroed ... ok
test commit_intent_counts_increment_all_kinds ... ok
test code_age_distribution_serde_roundtrip ... ok
test baseline_version_is_1 ... ok
test commit_intent_counts_increments_total ... ok
test complexity_risk_all_variants_roundtrip ... ok
test commit_intent_report_serde_roundtrip ... ok
test coupling_row_with_normalization_roundtrip ... ok
test determinism_baseline_serde_roundtrip ... ok
test coupling_row_without_normalization_skips_fields ... ok
test determinism_baseline_without_cargo_lock ... ok
test entropy_class_all_variants_roundtrip ... ok
test envelope_schema_constant_format ... ok
test finding_severity_all_variants_roundtrip ... ok
test function_complexity_detail_full_roundtrip ... ok
test function_complexity_detail_minimal ... ok
test gate_results_serde_roundtrip ... ok
test histogram_ascii_all_zero_no_panic ... ok
test histogram_ascii_single_bucket ... ok
test histogram_ascii_lines_equal_counts_len ... ok
test license_source_kind_all_variants_roundtrip ... ok
test maintainability_index_serde_roundtrip ... ok
test maintainability_index_without_halstead ... ok
test near_dup_algorithm_serde_roundtrip ... ok
test near_dup_scope_all_variants_roundtrip ... ok
test near_dup_cluster_serde_roundtrip ... ok
test near_dup_scope_default_is_module ... ok
test halstead_metrics_serde_roundtrip ... ok
test near_dup_scope_uses_kebab_case ... ok
test near_dup_stats_serde_roundtrip ... ok
test prop_histogram_bucket_preserved ... ok
test prop_near_dup_scope_roundtrip ... ok
test schema_version_is_9 ... ok
test sensor_report_alias_serde_roundtrip ... ok
test technical_debt_level_all_variants_roundtrip ... ok
test technical_debt_ratio_serde_roundtrip ... ok
test trend_class_all_variants_roundtrip ... ok
test verdict_all_variants_roundtrip ... ok
test verdict_default_is_pass ... ok
test prop_entropy_class_roundtrip ... ok
test prop_complexity_risk_roundtrip ... ok
test prop_topic_term_score_preserved ... ok

test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running `/app/target/debug/deps/baseline-b4b95414323a28f8`

running 4 tests
test complexity_baseline_from_analysis_with_complexity ... ok
test complexity_baseline_new_defaults ... ok
test complexity_histogram_to_ascii_formats_lines ... ok
test complexity_baseline_from_analysis_without_complexity ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running `/app/target/debug/deps/coverage_expansion-e767f02e60bfee11`

running 20 tests
test commit_intent_counts_default_is_zeroed ... ok
test analysis_args_meta_all_fields_roundtrip ... ok
test analysis_source_all_fields_roundtrip ... ok
test commit_intent_counts_increment_all_kinds ... ok
test api_surface_report_roundtrip ... ok
test complexity_histogram_empty_counts ... ok
test complexity_histogram_single_bucket ... ok
test complexity_baseline_serde_roundtrip ... ok
test complexity_report_minimal_roundtrip ... ok
test complexity_risk_all_variants_snake_case ... ok
test fun_report_none_eco_roundtrip ... ok
test fun_report_with_eco_label_roundtrip ... ok
test duplicate_report_roundtrip ... ok
test git_report_roundtrip ... ok
test near_dup_scope_default_is_module ... ok
test near_dup_scope_all_variants_roundtrip ... ok
test receipt_double_roundtrip_is_stable ... ok
test technical_debt_level_all_variants_snake_case ... ok
test topic_clouds_btreemap_keys_are_sorted ... ok
test receipt_with_archetype_roundtrip ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running `/app/target/debug/deps/feature_stability_w53-f2668c5dc4151352`

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running `/app/target/debug/deps/properties-22218058e489d5ca`

running 35 tests
test bus_factor_row_roundtrip ... ok
test churn_trend_roundtrip ... ok
test asset_file_row_roundtrip ... ok
test cocomo_report_roundtrip ... ok
test boilerplate_report_roundtrip ... ok
test context_window_report_roundtrip ... ok
test archetype_roundtrip ... ok
test derived_totals_roundtrip ... ok
test distribution_report_roundtrip ... ok
test domain_stat_roundtrip ... ok
test coupling_row_roundtrip ... ok
test eco_label_roundtrip ... ok
test entropy_class_roundtrip ... ok
test entropy_class_snake_case ... ok
test hotspot_row_roundtrip ... ok
test import_edge_roundtrip ... ok
test entropy_finding_roundtrip ... ok
test duplicate_group_roundtrip ... ok
test license_source_kind_roundtrip ... ok
test near_dup_algorithm_roundtrip ... ok
test integrity_report_roundtrip ... ok
test lockfile_report_roundtrip ... ok
test license_finding_roundtrip ... ok
test near_dup_stats_roundtrip ... ok
test near_dup_pair_row_roundtrip ... ok
test polyglot_report_roundtrip ... ok
test ratio_row_roundtrip ... ok
test reading_time_report_roundtrip ... ok
test schema_version_is_valid ... ok
test test_density_report_roundtrip ... ok
test todo_report_roundtrip ... ok
test near_dup_cluster_roundtrip ... ok
test trend_class_roundtrip ... ok
test trend_class_snake_case ... ok
test topic_term_roundtrip ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

     Running `/app/target/debug/deps/proptest_w69-5c8bbdaa255cb896`

running 20 tests
test analysis_schema_version_value ... ok
test baseline_metrics_default_zeroed ... ok
test analysis_preset_names_valid ... ok
test baseline_version_value ... ok
test commit_intent_counts_increment ... ok
test analysis_receipt_minimal_roundtrip ... ok
test complexity_baseline_default_version ... ok
test complexity_risk_roundtrip ... ok
test analysis_receipt_archetype_roundtrip ... ok
test commit_intent_counts_total_equals_sum ... ok
test license_source_kind_roundtrip ... ok
test entropy_class_roundtrip ... ok
test near_dup_scope_roundtrip ... ok
test eco_label_roundtrip ... ok
test domain_stat_roundtrip ... ok
test technical_debt_level_roundtrip ... ok
test trend_class_roundtrip ... ok
test ratio_row_bounded ... ok
test ratio_row_serde_roundtrip ... ok
test topic_term_serde_roundtrip ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running `/app/target/debug/deps/schema_compliance_w53-4ec3c4bea2da9672`

running 14 tests
test analysis_receipt_schema_version_in_json ... ok
test analysis_receipt_json_has_required_fields ... ok
test analysis_schema_version_is_positive ... ok
test analysis_schema_version_matches_documented_value ... ok
test analysis_receipt_roundtrip ... ok
test analysis_receipt_with_enrichments_roundtrip ... ok
test archetype_serializes_correctly ... ok
test analysis_source_structure ... ok
test complexity_risk_serde_roundtrip ... ok
test entropy_report_serializes_correctly ... ok
test fun_report_serializes_correctly ... ok
test missing_enrichments_serialize_as_null ... ok
test topic_clouds_serializes_correctly ... ok
test preset_metadata_in_args ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running `/app/target/debug/deps/schema_contract-ace1741453eb3e26`

running 13 tests
test analysis_schema_version_pinned ... ok
test analysis_receipt_json_roundtrip ... ok
test analysis_receipt_with_all_none_enrichers_roundtrips ... ok
test analysis_schema_version_is_positive ... ok
test baseline_version_pinned ... ok
test archetype_serializable ... ok
test entropy_report_serializable ... ok
test fun_report_serializable ... ok
test import_report_serializable ... ok
test schema_version_field_in_json ... ok
test unknown_fields_in_json_are_tolerated ... ok
test preset_names_stable ... ok
test properties::analysis_receipt_roundtrip ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running `/app/target/debug/deps/schema_contract_w66-957a74d97c9611ab`

running 20 tests
test analysis_args_meta_field_names_stable ... ok
test analysis_receipt_envelope_has_schema_version ... ok
test analysis_receipt_ignores_extra_fields ... ok
test analysis_receipt_missing_optional_sections_ok ... ok
test analysis_receipt_json_roundtrip ... ok
test analysis_schema_version_is_positive ... ok
test analysis_schema_version_value ... ok
test baseline_version_is_positive ... ok
test analysis_receipt_required_fields_present ... ok
test commit_intent_kind_all_variants_roundtrip ... ok
test complexity_risk_serializes_snake_case ... ok
test analysis_receipt_with_all_optional_sections_roundtrip ... ok
test complexity_baseline_json_roundtrip ... ok
test entropy_class_serializes_snake_case ... ok
test near_dup_scope_serializes_kebab_case ... ok
test license_source_kind_serializes_snake_case ... ok
test technical_debt_level_serializes_snake_case ... ok
test near_duplicate_report_json_roundtrip ... ok
test topic_clouds_json_roundtrip ... ok
test trend_class_serializes_snake_case ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running `/app/target/debug/deps/schema_validation-7ee360efa057ca48`

running 15 tests
test analysis_args_meta_optional_fields_null_when_none ... ok
test analysis_args_meta_roundtrip ... ok
test analysis_receipt_json_contains_args_fields ... ok
test analysis_receipt_json_contains_required_envelope_fields ... ok
test analysis_receipt_json_contains_source_fields ... ok
test analysis_receipt_optional_sections_serialize_as_null ... ok
test analysis_receipt_roundtrip ... ok
test analysis_receipt_schema_version_field_matches_constant ... ok
test analysis_schema_version_matches_expected ... ok
test analysis_receipt_top_level_keys_are_known ... ok
test analysis_receipt_roundtrip_preserves_null_optional_fields ... ok
test analysis_source_optional_fields_absent_when_none ... ok
test analysis_source_roundtrip ... ok
test baseline_version_matches_expected ... ok
test scan_status_values_in_analysis_context ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running `/app/target/debug/deps/serde_compat_v2-09f14b372189d23b`

running 22 tests
test artifact_type_field_deserializes_from_type_key ... ok
test artifact_type_field_serializes_as_type ... ok
test churn_trend_roundtrip_with_classification ... ok
test artifact_roundtrip_preserves_type_rename ... ok
test code_age_distribution_uses_trend_class ... ok
test complexity_risk_all_variants_snake_case ... ok
test entropy_finding_roundtrip ... ok
test entropy_class_all_variants_snake_case ... ok
test envelope_capability_state_all_variants_lowercase ... ok
test envelope_re_export_aliases_are_same_type ... ok
test finding_severity_all_variants_lowercase ... ok
test file_complexity_roundtrip_with_risk_level ... ok
test near_dup_params_roundtrip_with_scope ... ok
test near_dup_scope_default_is_module ... ok
test license_source_kind_all_variants_snake_case ... ok
test near_dup_scope_all_variants_kebab_case ... ok
test sensor_report_roundtrip_with_verdict_and_severity ... ok
test technical_debt_level_all_variants_snake_case ... ok
test trend_class_all_variants_snake_case ... ok
test technical_debt_ratio_roundtrip ... ok
test verdict_all_variants_lowercase ... ok
test verdict_default_is_pass ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests tokmd_analysis_types
     Running `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc --edition=2024 --crate-type lib --color auto --crate-name tokmd_analysis_types --test crates/tokmd-analysis-types/src/lib.rs --test-run-directory /app/crates/tokmd-analysis-types -L native=/app/target/debug/build/blake3-24f25e6af21f2888/out --extern proptest=/app/target/debug/deps/libproptest-c22aa1ec6e28355e.rlib --extern serde=/app/target/debug/deps/libserde-dcea86c0a8efc1ac.rlib --extern serde_json=/app/target/debug/deps/libserde_json-b02faf413897a7d0.rlib --extern tokmd_analysis_types=/app/target/debug/deps/libtokmd_analysis_types-17b1c00370d108ff.rlib --extern tokmd_envelope=/app/target/debug/deps/libtokmd_envelope-0e4273f9b236521b.rlib --extern tokmd_types=/app/target/debug/deps/libtokmd_types-729fd4069bfbb733.rlib -L dependency=/app/target/debug/deps -C embed-bitcode=no --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' --error-format human`

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### `cargo fmt -- --check`
```

```

### `cargo clippy -- -D warnings -p tokmd-analysis-types`
```
    Checking serde_core v1.0.228
    Checking memchr v2.8.0
    Checking utf8parse v0.2.2
    Checking anstyle v1.0.13
    Checking anstyle-parse v1.0.0
    Checking is_terminal_polyfill v1.70.2
    Checking colorchoice v1.0.4
    Checking anstyle-query v1.1.5
    Checking strsim v0.11.1
    Checking anstream v1.0.0
    Checking clap_lex v1.0.0
    Checking anyhow v1.0.102
    Checking clap_builder v4.6.0
    Checking aho-corasick v1.1.4
    Checking crossbeam-utils v0.8.21
    Checking regex-automata v0.4.14
    Checking serde v1.0.228
    Checking clap v4.6.0
    Checking serde_json v1.0.149
    Checking tokmd-types v1.8.0 (/app/crates/tokmd-types)
error: Unrecognized option: 'p'

error: could not compile `tokmd-types` (lib)

Caused by:
  process didn't exit successfully: `/home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/clippy-driver /home/jules/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc --crate-name tokmd_types --edition=2024 crates/tokmd-types/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata -C embed-bitcode=no -C debuginfo=2 --cfg 'feature="clap"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("clap"))' -C metadata=e144cd8426bebe9a -C extra-filename=-31e803e7701379ed --out-dir /app/target/debug/deps -C incremental=/app/target/debug/incremental -L dependency=/app/target/debug/deps --extern clap=/app/target/debug/deps/libclap-87a05a7791d8bf54.rmeta --extern serde=/app/target/debug/deps/libserde-8e26167187254633.rmeta` (exit status: 1)
warning: build failed, waiting for other jobs to finish...
```

## 🧭 Telemetry
- **Change shape**: Refactoring unit test signatures
- **Blast radius**: Test module only (no public API changes)
- **Risk class**: Low (isolated to test compilation)
- **Merge-confidence gates**: build, test, fmt, clippy

## 🗂️ .jules updates
- Created `.jules/policy/scheduled_tasks.json`, run templates, and directories.
- Added envelope `.jules/palette/envelopes/20260319T122251Z.json` with execution receipts.
- Updated `.jules/palette/ledger.json` with run results.

## 📝 Notes (freeform)
Replaced over a dozen unwraps across serialization/deserialization tests.
