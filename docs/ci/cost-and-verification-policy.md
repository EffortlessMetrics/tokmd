# tokmd verification economics policy

We are not reducing CI because we want less verification.

tokmd exists to produce deterministic receipts, analysis artifacts, review
reports, and CI gates. That makes verification central to the product. The
goal is to keep that verification strong while making **ordinary PR
verification cheap, deterministic, and scoped to the change.**

Agentic development increases PR volume and integration pressure.
Verification demand rises faster than generation cost. OpenClaw's published
Blacksmith runner spend of roughly $511k maps directionally to about $20 per
commit since February, with the squash-merge caveat. We read that as
evidence that serious agentic workflows need *more* verification with
*better* economics, not less verification.

- Rust makes local checks fast.
- Clippy catches bad local code shapes.
- TOML policy ledgers make exceptions reviewable.
- ripr gives mutation-testing-lite static oracle-gap signal.
- LEM budgeting makes cost visible.
- CI routing spends expensive lanes only where they buy proof.

## Operating principle

```text
Default PR:
  cheap proof that the changed risk surface is sound.
Main / nightly / label:
  expensive confirmation that the whole product surface still holds.
```

## Verification ladder

From cheapest to most expensive:

1. `cargo check` / Clippy — local code shape.
2. Unit / oracle tests — deterministic proof.
3. `ripr` — static mutation-shaped oracle-gap signal.
4. Property tests — randomized invariant proof.
5. Coverage — surfaces what is and isn't exercised.
6. Runtime mutation testing — confirms tests kill concrete mutants.
7. Crossval, hardware, packaging — last-mile, environment-bound proof.

Each rung answers a different question. Spending on a higher rung does not
substitute for the rung below it; spending on a higher rung *only* makes
sense once the lower rungs are clean and the marginal proof is worth the
marginal cost.

## Default PR target

| Tier | LEM band | Outcome |
|------|----------|---------|
| Frontdoor (ordinary) | 0–35 | Green by default. Sub-$0.50 wherever possible. |
| Elevated | 36–75 | Warning. Justified by risk pack hit or label. |
| High-cost | 76–125 | Strong warning. Requires explicit label. |
| Override | >125 | Blocked unless `full-ci` or `ci-budget-override`. |

See `docs/ci/lem-budgeting.md` for the LEM definition and worked examples.

## What this is *not*

- Not a reduction in proof for what reaches main.
- Not a relaxation of strict-lint posture.
- Not a removal of mutation testing — only a change in *when* it runs.
- Not a soft-gate on advisory signal until calibration data exists.
