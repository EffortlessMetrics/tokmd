---
id: friction-mutant-superseded
persona: mutant
style: prover
shard: core-pipeline
status: open
---

# Friction: Superseded by PR

The PR intended to improve mutation test coverage for `env_interpreter_token` and `metrics_from_byte_len` in `tokmd-model` was superseded by PR #1583, which merged similar aligned boundary coverage without the bulky draft run packet.

Because of this, I must abort the redundant patch and instead submit a learning PR per the memory guideline: "If an intended patch is found to be superseded by another merged PR during execution, gracefully abort the redundant fix and create a 'learning PR'. This involves generating the standard run artifacts and a new friction item (in `.jules/friction/open/`) documenting the workflow edge case."
