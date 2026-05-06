# Friction Item: Superseded PR
The performance improvement for `build_integrity_report` (replacing `format!` with custom string buffer logic) was superseded by #1608.
The duplicate logic collided with another generated Bolt provenance path.
