# PR Glass Cockpit

## Type
- [ ] Bug fix
- [ ] New feature
- [ ] Maintenance
- [x] Documentation / Examples

## Description
Updated the `tokmd-core` README example to properly compile under the latest API which favors `lang_workflow` and `ScanSettings`/`LangSettings` over the old clap-based arg structs.

## Verification Receipts
```
$ cargo test -p tokmd-core --doc
running 6 tests
test crates/tokmd-core/src/../README.md - readme_doctests (line 19) - compile ... ok
...
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.12s
```
