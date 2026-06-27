# Friction Item: Superseded Work

The work to harden the FFI JSON boundary by adding `is_object()` checks to `parse_*_settings` was superseded by PR #2713, which used a `nested_arg_object` helper that also preserved `None | null -> default` semantics. My patch would have unintentionally turned `{"lang": null}` into an error.
