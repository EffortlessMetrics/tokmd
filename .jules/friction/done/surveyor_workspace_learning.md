# Friction Item: Unused dependencies in Fuzz crate

**Status:** done

**Resolution:** The stale `tokmd-config` fuzz dependency is no longer present after the config retirement work; this is no longer an active workspace friction item.

During a surveyor workspace review, we observed that `tokmd-fuzz` contains an unused dependency on `tokmd-config`.
While this is minor, it is not a structural defect within the crate boundaries of `tokmd-analysis` and `tokmd-core`, and thus does not meet the high bar for a surveyor architectural seam fix.

Additionally, `cargo machete` is not installed by default in the execution environment, which caused some friction during the workspace scan.
