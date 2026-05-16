# Sample Artifact Trees

Use these examples when you need to see the physical shape of a `tokmd`
artifact set before generating one locally.

These are small layout walkthroughs, not checked-in generated packets. They
show what to open first, what each file owns, and what not to infer from the
artifact set.

## Trees

- [Review packet tree](review-packet-tree.md)
- [Handoff tree](handoff-tree.md)
- [Proof status tree](proof-status-tree.md)
- [Browser receipt tree](browser-receipt-tree.md)
- [Publishing evidence tree](publishing-evidence-tree.md)

## Rules

- Treat generated receipts as evidence for the run that produced them, not as
  merge approval.
- Treat advisory proof, coverage, mutation, and browser output as advisory
  unless policy explicitly promotes them.
- Regenerate verifier receipts after changing packet-local files.
- Use [User paths](../user-paths.md) to choose the right workflow before using
  these examples.
