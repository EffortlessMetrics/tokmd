# Friction Item: baseline_metrics_has_total_files obsolete

The Wasm / no-default-features issue causing `tokmd baseline` to output `total_files: 0` was initially targeted for a compat fix. However, the reviewer noted that the main branch already handles `total_files` correctly in `ComplexityBaseline::from_analysis`. Thus, the patch has been closed.
