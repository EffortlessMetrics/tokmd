# Friction Item: Stale Error Hints Fix

## Context
The `Specsmith` persona identified that some CLI subcommands (like `check-ignore`, `diff`, and `init`) emit "does not exist" or "Directory does not exist" instead of the standard "Path not found: <path>". This prevents the global `error_hints` machinery from providing actionable troubleshooting advice.

## Friction
A patch was prepared to standardize these error messages, but it was deferred and closed as stale because it predated the 1.15.0 release and was not considered a release blocker.

## Resolution
The PR was converted into a learning PR and closed to keep the active queue clean. The work remains in the branch history for future salvage if standardizing these error messages is still desired.
