# Architecture: AI-Native Crate Design

## Why 45 Crates?

This repository uses fine-grained crate decomposition optimized for AI agent development, not human developer convenience.

### Design Rationale

**Unix Philosophy for AI Agents**
- Each crate has one focused responsibility (SRP enforced at crate boundary)
- Agents can reason about narrow scope → clearer tool selection
- Independent versioning enables precise dependency tracking

**Agent Workflow Optimization**
- Smaller context windows per crate (fits easily in agent working memory)
- Clear scope boundaries → agents know exactly which crate to use
- Composable tools → agents chain focused crates rather than navigating monolithic APIs
- Independent reasoning → "I need complexity metrics" → `tokmd-analysis-complexity`

**Tradeoffs Acknowledged**
- ✅ Benefit: Agent clarity, focused scope, composable tooling
- ⚠️ Cost: Cargo workspace overhead, dependency duplication in Cargo.toml files
- ⚠️ Cost: Human developers see "crate proliferation" as friction

This is intentional: we optimize for agentic development workflows, not traditional human-only development.

## Crate Discovery Patterns for Agents

### How Agents Select Crates

1. **By metric type**: `tokmd-analysis-*` crates are named by metric (complexity, entropy, halstead, fingerprint)
2. **By function**: `tokmd-scan` for scanning, `tokmd-walk` for traversal, `tokmd-path` for normalization
3. **By output**: `tokmd-receipts` for evidence generation, `tokmd-report` for formatting

### Crate Categories

**Analysis Crates** (tokmd-analysis-*):
- Each implements one metric algorithm
- Independent versioning (can update halstead without touching entropy)
- Agents select by metric name

**Infrastructure Crates** (tokmd-scan, tokmd-walk, tokmd-path):
- Filesystem operations
- Shared utilities
- Lower-level building blocks

**Governance Crates** (tokmd-gate, tokmd-policy, tokmd-receipts):
- Policy enforcement
- Evidence generation
- CI/CD integration

## Crate Boundary Rules

### What Belongs in a New Crate

✅ Algorithm implementation (one metric, one crate)
✅ Independent lifecycle (versioned separately)
✅ Reusable across contexts (not template-specific)
✅ Clear input/output contract

### What Stays in Existing Crates

❌ Variations of same algorithm (use feature flags)
❌ Always-versioned-together code (use modules)
❌ Template-specific logic (stays in rust-as-spec)

## Agentic Development Workflow

### Typical Agent Session

1. Agent receives task: "Add cyclomatic complexity tracking"
2. Agent searches for existing complexity crate: `tokmd-analysis-complexity`
3. Agent examines crate API (small, focused)
4. Agent implements changes with clear scope
5. Agent runs tests for that crate only (fast feedback)

### Vs Monolithic Alternative

In a monolithic `tokmd-analysis` crate:
- Agent must navigate 10x more code
- Scope unclear (where does complexity logic end?)
- Tests slower (entire crate must rebuild)
- Harder to reason about changes

## For Human Contributors

If you're a human developer reading this and thinking "this seems like over-engineering":

**You're right — for human-only development, it would be.**

This architecture is designed for a future where AI agents are primary code authors, and humans are reviewers/orchestrators. The crate boundaries serve agent reasoning, not human IDE navigation.

### Working With This Structure

1. **Don't consolidate crates** for "simplicity" — this breaks agentic workflows
2. **Do add new crates** for genuinely independent functionality
3. **Use modules within crates** for sub-structure (not new crates for everything)
4. **Question boundaries** if crates always version together (may be false granularity)

## Questions & Answers

**Q: Why not use modules instead of crates?**

A: Crates provide compilation boundaries, independent versioning, and clear API surfaces. Modules are invisible to agents as tool selection units.

**Q: Don't agents just read the whole codebase?**

A: No — context windows are limited, and focused crates reduce token usage. Agents work more effectively with narrow scope.

**Q: Will this work for human developers?**

A: Yes, but with friction. Import paths are longer, Cargo.toml has more entries. This is an intentional tradeoff.

**Q: How do I know if something needs a new crate?**

A: Ask: "Does this have independent versioning lifecycle?" and "Would an agent want to use this without the rest?" If yes → new crate.

## Related Documentation

- CONTRIBUTING.md: Development workflow
- crates/*/README.md: Individual crate documentation
- docs/agentic-development.md: Agent workflow guides
