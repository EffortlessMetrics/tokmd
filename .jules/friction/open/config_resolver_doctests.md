# Config Resolver Doctest Coverage
While investigating the `crates/tokmd/src/config/resolve/` interfaces (such as `resolve_lang`, `resolve_export`, and `resolve_module`), I discovered that comprehensive and passing executable `rust` doctests are already in place for all `resolve_*` and `resolve_*_with_config` functions.

Because `Librarian` rules require factual drift, missing executable coverage, or a clearly misleading omission to justify a patch, and because there was no drift or missing coverage on this targeted public interface, a code change here would have forced a fake fix.

Instead of forcing a patch, this run was pivoted into a learning PR to document that the public CLI config resolving interfaces are already well-covered and protected against silent drift.
