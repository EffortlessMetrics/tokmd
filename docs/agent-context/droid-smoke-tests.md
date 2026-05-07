# Droid Smoke Test Checklist for tokmd

This document specifies the live smoke test procedures for validating Droid auto-review and manual execution in the tokmd repository.

## Smoke Test Environment

- Runs on the `main` branch after Droid workflows are deployed
- Performed on same-repo pull requests (no cross-fork execution)
- Uses draft PRs to allow early abort if issues are found
- Validates both auto-review and manual `@droid` invocation

## Pre-Smoke Test Prerequisites

- [ ] MINIMAX_API_KEY secret is set in GitHub repository settings
- [ ] FACTORY_API_KEY secret is set in GitHub repository settings
- [ ] `.github/workflows/droid-review.yml` exists and is enabled
- [ ] `.github/workflows/droid.yml` exists and is enabled
- [ ] `.github/workflows/droid-security-scan.yml` exists and is enabled
- [ ] `.factory/rules/droid-review.md` exists with review guidance
- [ ] `.factory/skills/review-guidelines/SKILL.md` exists with skill definition
- [ ] Static checks pass: `cargo xtask gate --check`

## Test Sequence

### Test 1: Auto Review on PR Creation

**Procedure**:
1. Create a new draft PR on the same-repo branch (any non-trivial change, e.g., typo fix)
2. Observe GitHub Actions tab for workflow status
3. Wait 5–10 minutes for Droid to start

**Expected Outcome**:
- [ ] Droid Auto Review workflow starts within 2 minutes
- [ ] Workflow runs with status "in progress"
- [ ] No permission errors in workflow logs

**Validation**:
- [ ] Workflow shows "Checkout repository" step completed
- [ ] "Configure MiniMax BYOK for Factory Droid" step runs without error
- [ ] "Run Droid Auto Review with MiniMax M2.7 BYOK" step invokes the safe action

**Failure Mode**: If workflow does not start:
- Check that PR head is on the same repo (not a fork)
- Check that PR title does not contain `[skip-review]`
- Check repository Settings > Actions > Workflow permissions include "Read and write permissions"

---

### Test 2: Auto Review Uses Correct Model

**Procedure**:
1. Once auto-review completes (after ~5–10 minutes from PR creation), check the workflow logs
2. Navigate to the workflow run details
3. Expand the "Run Droid Auto Review" step

**Expected Outcome**:
- [ ] Step completed successfully (green checkmark)
- [ ] Logs show: `review_model: custom:MiniMax-M2.7-0`
- [ ] Logs show: `security_model: custom:MiniMax-M2.7-0`
- [ ] No authentication errors or API key issues in logs
- [ ] MiniMax API calls are successful (check for rate limit or auth errors)

**Validation**:
- [ ] Model name is exactly `custom:MiniMax-M2.7-0` (not a different Claude model)
- [ ] No fallback to default model appears in logs
- [ ] API response time is reasonable (< 30 seconds per call)

**Failure Mode**: If model is incorrect or API calls fail:
- Check that MINIMAX_API_KEY is set correctly in GitHub secrets
- Check MiniMax dashboard for API key validity and rate limits
- Verify model name in `.github/workflows/droid-review.yml` matches documentation

---

### Test 3: No Raw Debug Artifacts Uploaded

**Procedure**:
1. After workflow completes, navigate to the Actions tab
2. Click on the Droid Auto Review run
3. Scroll to "Artifacts" section at the bottom of the page

**Expected Outcome**:
- [ ] Artifacts section shows no items OR shows only expected artifacts
- [ ] No artifact named `droid-review-debug-<run_id>` is present
- [ ] No artifact named `droid-review-debug-sanitized-<run_id>` is present (unless explicitly enabled)
- [ ] Artifacts section shows "no artifacts" or lists only CI artifacts (e.g., test reports)

**Validation**:
- [ ] `upload_debug_artifacts: false` is set in `.github/workflows/droid-review.yml`
- [ ] Workflow logs confirm: `upload_debug_artifacts: false`

**Failure Mode**: If debug artifacts are uploaded:
- Verify workflow YAML contains `upload_debug_artifacts: false` in the action `with:` block
- Check that no custom action wrapper is overriding this setting
- Verify EffortlessMetrics/droid-action-safe is being used, not Factory-AI/droid-action

---

### Test 4: Auto Review Generates Comment

**Procedure**:
1. Return to the draft PR page (Conversation tab)
2. Scroll down to see all comments
3. Look for a comment from "factory-bot" or the GitHub Actions bot

**Expected Outcome**:
- [ ] A review comment appears from an automated account
- [ ] Comment body includes findings or a clean-review explanation
- [ ] Comment is marked as a "Review" (not a regular comment)
- [ ] Timestamp shows it was generated during the workflow run

**Validation**:
- [ ] If no findings: comment includes "Inspected surfaces" section with inspection record format
- [ ] If findings present: each finding includes title, failure mode, fix direction, and validation guidance
- [ ] Comment does not include raw `droid-review-debug-<run_id>` content
- [ ] Comment formatting follows `.factory/rules/droid-review.md` standards

**Failure Mode**: If no comment appears:
- Wait another 2–3 minutes for the workflow to finish
- Check workflow logs for errors in the "Run Droid Auto Review" step
- Verify that the PR was created on the same repo (not a fork)
- Check GitHub Action permissions in repository settings

---

### Test 5: Manual @droid review Command

**Procedure**:
1. In the draft PR, navigate to the Conversation tab
2. Leave a comment with text: `@droid review`
3. Wait 2–3 minutes for the Droid Tag workflow to trigger

**Expected Outcome**:
- [ ] Droid Tag workflow starts in the Actions tab
- [ ] Workflow completes with status "success"
- [ ] A new comment appears with Droid's manual review output

**Validation**:
- [ ] Comment is from the same automated account as auto-review
- [ ] Timestamp matches or follows the manual comment timestamp
- [ ] Content follows `.factory/rules/droid-review.md` standards
- [ ] Comment body does not include raw debug artifacts

**Failure Mode**: If workflow does not trigger:
- Verify your GitHub account is an OWNER or COLLABORATOR on the repo
- Check that your comment explicitly contains `@droid review`
- Wait another 30 seconds and check Actions tab again
- Verify workflow YAML contains your role in the `author_association` check

---

### Test 6: Manual @droid security Command

**Procedure**:
1. Leave a second comment with text: `@droid security`
2. Wait 2–3 minutes for the Droid Tag workflow to trigger again

**Expected Outcome**:
- [ ] Droid Tag workflow starts in the Actions tab
- [ ] Workflow completes with status "success"
- [ ] A new comment appears with Droid's security scan output

**Validation**:
- [ ] Comment focuses on security-relevant findings (not style issues)
- [ ] If no security findings: comment includes clean-review format with inspection record
- [ ] If security findings: each is prioritized as P0/P1/P2

**Failure Mode**: Same troubleshooting as Test 5.

---

### Test 7: Scheduled Security Scan (Manual Trigger)

**Procedure**:
1. Navigate to the Actions tab of the repository
2. Find "Droid Security Scan" workflow in the left sidebar
3. Click "Droid Security Scan"
4. Click the "Run workflow" button (dropdown on the right)
5. Select branch `main` or current branch
6. Click "Run workflow"
7. Wait 5–10 minutes for the scan to complete

**Expected Outcome**:
- [ ] Workflow run appears and progresses through steps
- [ ] All steps complete successfully
- [ ] Workflow logs show security scan completed

**Validation**:
- [ ] Logs show: `security_scan_schedule: true`
- [ ] Logs show: `security_severity_threshold: medium`
- [ ] Logs show MiniMax API calls for security analysis
- [ ] Workflow creates or updates a security-focused comment/issue (if findings present)

**Failure Mode**: If workflow fails to start:
- Verify `droid-security-scan.yml` exists in `.github/workflows/`
- Check that the workflow file is syntactically valid YAML
- Verify GitHub Actions permissions include "Read and write"

---

### Test 8: MiniMax Usage Appears in Dashboard

**Procedure**:
1. Log in to your MiniMax account dashboard (https://api.minimax.io/)
2. Navigate to the API usage or logs section
3. Look for requests from your GitHub repository

**Expected Outcome**:
- [ ] Dashboard shows recent API calls from the tokmd repository
- [ ] Call timestamps align with the Droid workflow executions (Tests 2, 5, 6, 7)
- [ ] Calls are attributed to your MINIMAX_API_KEY

**Validation**:
- [ ] At least 1 call per workflow execution
- [ ] Model name in logs matches `MiniMax-M2.7`
- [ ] Response times are reasonable

**Failure Mode**: If no calls appear:
- Verify MINIMAX_API_KEY is correctly set in GitHub secrets
- Verify workflows are using the correct secret variable name: `${{ secrets.MINIMAX_API_KEY }}`
- Check MiniMax dashboard for API key validity or rate limit issues

---

## Post-Smoke Test

### Cleanup

- [ ] Close the draft PR (do not merge for smoke testing)
- [ ] Document any issues found in a GitHub issue

### Success Criteria

All 8 tests must pass:

- [x] Auto review starts on PR creation
- [x] Correct model (MiniMax-M2.7) is used
- [x] No raw debug artifacts uploaded
- [x] Auto review generates a comment with findings or clean review
- [x] Manual `@droid review` command works
- [x] Manual `@droid security` command works
- [x] Scheduled security scan can be triggered manually
- [x] MiniMax usage is visible in the MiniMax dashboard

### Next Steps

If all tests pass:
1. Mark this repo as validated in the rollout checklist
2. Proceed to Phase 2 (baseline convergence) if in Batch 1 or Batch 2
3. Monitor for any issues in production PRs

If any test fails:
1. Document the failure mode and reproduction steps
2. Open an issue in the tokmd repository
3. Do not proceed to Phase 2 until the failure is resolved

---

## Known Blind Spots

These aspects of Droid behavior are not validated by smoke tests:

1. **Deep-dive analysis on real code changes**: Smoke tests use trivial PRs (e.g., typo fixes). Real code changes may trigger different Droid behavior.
2. **False negative rate**: Smoke tests do not measure how many real bugs Droid misses.
3. **Edge case handling**: Very large diffs, many files, or special characters in paths are not tested.
4. **Performance at scale**: Droid timeout behavior on large repos is not tested.
5. **Concurrent PR handling**: Multiple simultaneous PRs are not tested.

To gather these signals, monitor production usage after Phase 2 rollout.

---

## References

- `.github/workflows/droid-review.yml` — Auto-review workflow
- `.github/workflows/droid.yml` — Manual command workflow
- `.github/workflows/droid-security-scan.yml` — Security scan workflow
- `.factory/rules/droid-review.md` — Review standards and finding formats
- `agents/shared/droid-migration.md` — Full rollout design
