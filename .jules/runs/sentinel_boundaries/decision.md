# Decision

## Option A (recommended)
Produce a learning PR. The instances of bare `Command::new("git")` found in the `interfaces` shard (specifically `tokmd-core` and tests) are contained entirely within test setups. Hardening these does not provide a production security improvement and forcing a patch would be a "fake fix". We will record this finding as a friction item instead.

## Option B
Refactor the test code to use `tokmd_git::git_cmd()`. This is low-value and misrepresents a test refactor as a security fix.

## Decision
I choose **Option A**. The prompt strictly forbids forcing a fake fix if no honest patch is justified. We will fall back to a learning PR and output the full per-run packet alongside a friction item.
