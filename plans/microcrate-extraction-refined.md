# tokmd Microcrate Extraction Analysis (Refined)

## Executive Summary of the Critique

### What Roo Got Right

The original analysis correctly identified several structural aspects of the tokmd codebase:

- **Tier Model**: The codebase follows a well-defined tiered architecture with clear separation between foundational types, adapters, orchestration, configuration, and CLI layers.
- **CLI Crate Candidates**: The main [`tokmd`](crates/tokmd/src/lib.rs:1) crate contains substantial modules that could potentially be extracted.
- **Config Coupling**: The analysis correctly identified that [`tokmd-config`](crates/tokmd-config/src/lib.rs:1) couples strict configuration schemas with Clap CLI parsing, as documented in its own module comments.

### Key Weaknesses

The original plan suffered from several fundamental issues:

1. **Line-Count Focus**: Prioritization was based on module size (~140 lines for badge, ~1057 lines for context-pack) rather than boundary quality. Small modules with poor boundaries were prioritized over larger modules with clean boundaries.

2. **Ignoring Publishing Reality**: The plan treated all extractions as equally publishable without considering:
   - Clap dependencies make crates unusable as libraries
   - Schema coupling to tokmd-config creates tight dependencies
   - Some modules are UI affordances, not reusable algorithms

3. **Backwards Boundaries**: The plan suggested extracting UI modules (badge, progress, interactive) as crates, when these are better kept as modules because they:
   - Have no external consumers
   - Are tightly coupled to CLI-specific concerns
   - Don't represent stable contracts

### The Fundamental Shift Needed

The refined approach shifts from **"smaller files"** to **"better boundaries"**:

| Original Approach | Refined Approach |
|-------------------|------------------|
| Extract by line count | Extract by boundary quality |
| All modules become crates | Only publishable boundaries become crates |
| Focus on code organization | Focus on dependency inversion |
| Size reduction as goal | Dependency simplification as goal |

---

## Boundary-First Principles

### Core Decision Framework

Each extraction must answer four questions:

1. **Who will depend on this?**
   - External consumers (other projects, language bindings, CI tools)
   - Internal consumers (other tokmd crates)
   - No consumers (keep as module)

2. **What dependencies does this eliminate?**
   - Does it allow tokmd-core to build without clap?
   - Does it reduce feature flag complexity?
   - Does it break circular dependencies?

3. **Does it reduce feature-flag complexity?**
   - Does it eliminate the need for optional features?
   - Does it simplify feature gate logic?

4. **Does it prevent cycles?**
   - Does it create a clearer dependency hierarchy?
   - Does it eliminate reverse dependencies?

### Publication Policy

- **Start as workspace-only** (`publish = false`) until boundary proves itself
- **Promote to publishable** only when:
  - Concrete external consumer exists
  - API has stabilized through use
  - Clap-free (for library crates)
  - Schema-linked (uses tokmd-types contracts)

### Anti-Patterns to Avoid

- Don't split formatting further unless it reduces schema coupling
- Don't extract UI affordances (badges, progress bars, interactive prompts)
- Don't create crates that only have one consumer
- Don't extract without a clear semver story

---

## Publish Matrix

### Phase 0: Keystone Candidates

| Crate Name | Publish | Type | Clap-Free | Semver Risk | External Consumer | Notes |
|------------|---------|------|-----------|-------------|-------------------|-------|
| tokmd-settings | yes | Contract | yes | low | tokmd-core, language bindings | Pure config without clap |
| tokmd-cli | workspace-only | Wrapper | no | high | None | Clap parsing, not publishable |

**Dependency Inversion**: This split enables tokmd-core to depend on tokmd-settings without pulling in clap.

### Phase 1: High-Impact Library Extractions

| Crate Name | Publish | Type | Clap-Free | Semver Risk | External Consumer | Notes |
|------------|---------|------|-----------|-------------|-------------------|-------|
| tokmd-context-pack | yes | Algorithm | yes | medium | LLM tools, AI agents | LLM context window optimization |
| tokmd-git-scoring | conditional | Algorithm | yes | low | tokmd-git consumers | Could fold into tokmd-git instead |

### Phase 2: Conditional Extractions

| Crate Name | Publish | Type | Clap-Free | Semver Risk | External Consumer | Notes |
|------------|---------|------|-----------|-------------|-------------------|-------|
| tokmd-derived-metrics | conditional | Library API | yes | medium | Bindings, analysis tools | Only if external consumer exists |
| tokmd-topics | conditional | Algorithm | yes | low | Text analysis tools | Only if used outside analysis |
| tokmd-archetype | conditional | Algorithm | yes | low | Project detection tools | Only if used outside analysis |

### Phase 3: Keep as Modules

| Module | Reason to Keep as Module |
|--------|-------------------------|
| tokmd-badge | UI affordance, no external consumers |
| tokmd-progress | UI affordance, tightly coupled to CLI |
| tokmd-interactive | UI affordance, dialoguer-dependent |
| tokmd-tools-schema | Tightly coupled to clap, not a library API |

---

## Re-Ranked Extraction Roadmap

### Phase 0: Keystone (Do This First)

**tokmd-config split → tokmd-settings + tokmd-cli**

This is the dependency inversion that unlocks everything else.

**Current State:**
```
tokmd-core → tokmd-config (with clap) ❌
tokmd → tokmd-config (with clap) ✓
```

**Target State:**
```
tokmd-core → tokmd-settings (no clap) ✓
tokmd → tokmd-cli (with clap) → tokmd-settings ✓
```

**Actions:**
1. Extract clap-dependent types to [`tokmd-cli`](crates/tokmd-config/src/lib.rs:1)
2. Extract pure config schemas to [`tokmd-settings`](crates/tokmd-config/src/lib.rs:1)
3. Update tokmd-core to depend on tokmd-settings only

**Success Criteria:**
- tokmd-core builds without clap dependency
- tokmd-cli is workspace-only (`publish = false`)
- tokmd-settings is publishable
- No regression in CLI functionality

**Estimated Impact:**
- Enables tokmd-core to be used as a library without clap
- Reduces dependency surface for language bindings
- Foundation for all future extractions

---

### Phase 1: High-Impact Library Extractions

#### 1.1 tokmd-context-pack (publish)

**Module Path:** [`crates/tokmd/src/context_pack.rs`](crates/tokmd/src/context_pack.rs:1)

**Boundary Quality:** High
- Pure algorithm (greedy and spread strategies)
- Depends on tokmd-types contracts only
- No clap dependency
- Well-tested with comprehensive coverage

**External Consumer:** LLM tools, AI agents, context optimization libraries

**Dependencies:**
- tokmd-types (ContextFileRow, FileRow, FileKind)
- tokmd-settings (ContextStrategy, ValueMetric) - after Phase 0

**Success Criteria:**
- Publishable crate with clap-free API
- Reduces tokmd crate size by ~1057 lines
- Enables external use for LLM context optimization

**Decision:** Extract and publish

#### 1.2 tokmd-git-scoring (conditional)

**Module Path:** [`crates/tokmd/src/git_scoring.rs`](crates/tokmd/src/git_scoring.rs:1)

**Boundary Quality:** Medium
- Pure algorithm (hotspot and commit count computation)
- Depends on tokmd-types and tokmd-git
- No clap dependency

**Alternative:** Fold into tokmd-git crate

**Decision:** Evaluate whether to:
- Extract as separate crate (if external consumer exists)
- Fold into tokmd-git (simpler, fewer crates)

**Success Criteria:**
- Either: published crate with external consumer
- Or: integrated into tokmd-git with public API

---

### Phase 2: Conditional Extractions

Only extract if there's a concrete external consumer.

#### 2.1 tokmd-derived-metrics (conditional)

**Module Path:** [`crates/tokmd-analysis/src/derived.rs`](crates/tokmd-analysis/src/derived.rs:1)

**Boundary Quality:** Medium-High
- Depends on tokmd-analysis-types and tokmd-types
- No clap dependency
- Well-tested with property-based testing

**External Consumer:** Language bindings, analysis tools

**Decision:** Extract only if:
- tokmd-python or tokmd-node need it
- External analysis tools request it

**Success Criteria:**
- Published crate with stable API
- Reduces tokmd-analysis size by ~745 lines
- Used by at least one external consumer

#### 2.2 tokmd-topics (conditional)

**Module Path:** [`crates/tokmd-analysis/src/topics.rs`](crates/tokmd-analysis/src/topics.rs:1)

**Boundary Quality:** High
- Pure TF-IDF algorithm
- No clap dependency
- Well-tested

**External Consumer:** Text analysis tools, topic modeling libraries

**Decision:** Extract only if external consumer exists

#### 2.3 tokmd-archetype (conditional)

**Module Path:** [`crates/tokmd-analysis/src/archetype.rs`](crates/tokmd-analysis/src/archetype.rs:1)

**Boundary Quality:** Medium
- Project pattern detection
- No clap dependency
- Well-tested

**External Consumer:** Project detection tools, IDE plugins

**Decision:** Extract only if external consumer exists

---

### Phase 3: Keep as Modules

These modules should remain as modules in their current crates:

#### 3.1 tokmd-badge (keep as module)

**Module Path:** [`crates/tokmd/src/badge.rs`](crates/tokmd/src/badge.rs:1)

**Reason:**
- UI affordance, not a library API
- No external consumers
- Tightly coupled to CLI-specific concerns (BadgeMetric from tokmd-config)
- ~140 lines - not worth extracting

#### 3.2 tokmd-progress (keep as module)

**Module Path:** [`crates/tokmd/src/progress.rs`](crates/tokmd/src/progress.rs:1)

**Reason:**
- UI affordance (progress indicators)
- No external consumers
- Depends on indicatif (UI library)
- ~130 lines - not worth extracting

#### 3.3 tokmd-interactive (keep as module)

**Module Path:** [`crates/tokmd/src/interactive/`](crates/tokmd/src/interactive/)

**Reason:**
- UI affordance (interactive prompts)
- No external consumers
- Depends on dialoguer (UI library)
- CLI-specific functionality

#### 3.4 tokmd-tools-schema (keep in CLI crate)

**Module Path:** [`crates/tokmd/src/tools_schema.rs`](crates/tokmd/src/tools_schema.rs:1)

**Reason:**
- Tightly coupled to clap (CLI introspection)
- Not a library API
- ~424 lines but clap-dependent
- Could be useful for other CLI tools, but requires clap

**Alternative:** If external demand exists, extract as clap-dependent crate (non-publishable)

---

## Success Criteria per Phase

### Phase 0: Keystone Success Criteria

**Compile Time Improvements:**
- tokmd-core builds without clap dependency
- Reduced dependency graph for tokmd-core

**Dependency Graph Simplification:**
- Clear separation: tokmd-core → tokmd-settings, tokmd → tokmd-cli → tokmd-settings
- No circular dependencies introduced

**Feature Flag Reduction:**
- Eliminates need for tokmd-core to have clap-related features

**API Stability Metrics:**
- tokmd-settings has stable, clap-free API
- tokmd-cli is workspace-only (no semver concerns)

---

### Phase 1: High-Impact Success Criteria

**Compile Time Improvements:**
- tokmd crate size reduced by ~1057 lines (context-pack)
- Reduced compilation time for tokmd crate

**Dependency Graph Simplification:**
- tokmd-context-pack has minimal dependencies (tokmd-types, tokmd-settings)
- No clap dependency in extracted crates

**Feature Flag Reduction:**
- Eliminates need for git feature in tokmd (if git-scoring moved)

**API Stability Metrics:**
- Published crates have stable, documented APIs
- Clear semver story for each published crate

---

### Phase 2: Conditional Success Criteria

**Compile Time Improvements:**
- tokmd-analysis size reduced (if extractions proceed)
- Reduced compilation time for tokmd-analysis

**Dependency Graph Simplification:**
- Extracted crates have clear, minimal dependencies
- No clap dependency in extracted crates

**Feature Flag Reduction:**
- Reduced feature flag complexity in tokmd-analysis

**API Stability Metrics:**
- Published crates have stable APIs
- External consumers confirmed before publishing

---

## Publishing Policy

### Publishable Crate Criteria

A crate is publishable if it meets ALL of these criteria:

1. **Type is Contract, Library API, or Algorithm**
   - Contract: Defines stable data structures and interfaces
   - Library API: Provides reusable functionality with clear contracts
   - Algorithm: Pure computation with well-defined inputs/outputs

2. **Clap-Free**
   - No direct or transitive dependency on clap
   - CLI-specific types are in separate, non-publishable crates

3. **Feature-Stable**
   - API has been validated through use
   - Breaking changes follow semver
   - Experimental features are feature-gated

4. **Schema-Linked**
   - Uses tokmd-types contracts where appropriate
   - Doesn't define duplicate types that should be in tokmd-types

5. **Small Dependency Surface**
   - Minimal external dependencies
   - No unnecessary dependencies

### Non-Publishable Crate Types

These types should be workspace-only (`publish = false`):

1. **Wrapper**
   - Thin wrappers around external libraries
   - No significant value-add

2. **UI Affordances**
   - Progress indicators, badges, interactive prompts
   - CLI-specific UI components

3. **Glue**
   - Code that connects other crates
   - No independent value

### Publishing Process

1. **Start as workspace-only**
   - Set `publish = false` in Cargo.toml
   - Validate boundary quality through use

2. **Promote to publishable**
   - Confirm external consumer exists
   - Validate API stability
   - Add comprehensive documentation
   - Set `publish = true` in Cargo.toml

3. **Maintain stability**
   - Follow semver for breaking changes
   - Document deprecation policy
   - Add stability sections to READMEs

---

## Implementation Guardrails

### 1. Use publish = false Initially

All new crates start as workspace-only:

```toml
[package]
name = "tokmd-new-crate"
publish = false  # Start as workspace-only
```

Promote to publishable only after validation.

### 2. Add Semver Checks for Publishable Crates

For publishable crates, add semver enforcement:

```toml
[package]
publish = true

[package.metadata.release]
pre-release-replacements = []
```

Use cargo-semver-checks in CI:

```bash
cargo semver-checks check-release
```

### 3. Maintain Stability Sections in READMEs

Each publishable crate should have a stability section:

```markdown
## API Stability

This crate follows semantic versioning. Breaking changes will increment the major version.

### Stable API

- `pack_greedy()` - Context packing algorithm
- `pack_spread()` - Spread strategy for context packing

### Experimental

- `pack_custom()` - Custom packing strategies (may change)
```

### 4. Use Unstable Feature Flags for Experimental Surfaces

For APIs that may change, use unstable feature flags:

```toml
[features]
default = []
unstable-custom-packing = []
```

```rust
#[cfg(feature = "unstable-custom-packing")]
pub fn pack_custom(...) {
    // Experimental API
}
```

### 5. Dependency Inversion Validation

Before extracting, validate that the extraction enables dependency inversion:

```
Before:
tokmd-core → tokmd-config (with clap) ❌

After:
tokmd-core → tokmd-settings (no clap) ✓
```

### 6. Feature Flag Audit

After extraction, audit feature flags:

- Eliminate unnecessary feature flags
- Consolidate related features
- Document feature flag behavior

### 7. Circular Dependency Prevention

Before extracting, analyze dependencies:

```
Use cargo-depgraph:
cargo depgraph --workspace | dot -Tpng > deps.png
```

Ensure no circular dependencies are introduced.

### 8. Test Coverage Requirements

Extracted crates must maintain test coverage:

- Move existing tests with the module
- Add integration tests to parent crate
- Maintain property-based tests where applicable

---

## Conclusion

The refined microcrate extraction analysis shifts focus from **"smaller files"** to **"better boundaries"**. The key insights are:

1. **Phase 0 is the keystone**: Splitting tokmd-config into tokmd-settings and tokmd-cli enables all other extractions by allowing tokmd-core to build without clap.

2. **Publishability matters**: Not all modules should become crates. Only publishable boundaries (clap-free, feature-stable, schema-linked) should be published.

3. **External consumers drive extraction**: Conditional extractions (derived-metrics, topics, archetype) should only proceed if there's a concrete external consumer.

4. **UI affordances stay as modules**: badge, progress, interactive, and tools-schema should remain as modules because they have no external consumers and are tightly coupled to CLI concerns.

5. **Boundary quality over line count**: tokmd-context-pack (~1057 lines) is a better extraction candidate than tokmd-badge (~140 lines) because it has a clean boundary and external consumers.

By following this boundary-first approach, tokmd can achieve:
- Cleaner dependency graphs
- Reduced feature flag complexity
- Better separation of library and CLI concerns
- More maintainable codebase
- Reusable components for external use
