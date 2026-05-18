# Browser Receipt Tree

Use this when your job is:

```text
Try tokmd in the browser before installing native tooling.
```

Run first: open the browser runner, then load a GitHub repository or local
files.

Sample layout after download:

```text
downloads/
  tokmd-browser-receipt.json
  tokmd-browser-summary.md
```

Open first:

1. Browser UI summary
2. `tokmd-browser-summary.md`
3. `tokmd-browser-receipt.json`

What each file owns:

| File | Owns |
| --- | --- |
| Browser UI summary | Immediate no-install interpretation. |
| `tokmd-browser-summary.md` | Saved human-readable browser-safe summary, when downloaded. |
| `tokmd-browser-receipt.json` | Machine-readable browser-safe receipt, when downloaded. |

What not to infer:

- Browser mode is not native mode.
- Browser receipts do not include native filesystem behavior.
- Browser receipts do not include git-history enrichers, cockpit packets,
  gates, handoff bundles, or AST shadow evidence.
- A browser receipt is not CI proof.

Next action:

- Move to native `tokmd cockpit` for real PR review.
- Move to native `tokmd handoff` for coding-agent context.
- Check the browser capability matrix before relying on a browser result.
- Use [Browser to native](../browser-to-native.md) for the shortest bridge from
  browser output to native review, handoff, and CI evidence workflows.
