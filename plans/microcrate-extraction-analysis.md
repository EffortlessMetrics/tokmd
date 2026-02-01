# tokmd Microcrate Extraction Analysis

## Executive Summary

This document provides a comprehensive analysis of the tokmd Rust codebase to identify opportunities for breaking it out into microcrates. The analysis examines the existing crate structure, dependency relationships, and module organization to identify practical, achievable microcrate extractions that would improve maintainability.

---

## 1. Current Codebase Structure Overview

### Workspace Members (18 crates)

The tokmd workspace follows a tiered architecture:

```
Tier 1 (Foundational - Stable)
├── tokmd-types          - Core data types and contracts
├── tokmd-model          - Deterministic aggregation and receipt modeling
└── tokmd-scan           - Source code scanning adapter (Tokei wrapper)

Tier 2 (Adapters)
├── tokmd-git            - Streaming git log adapter
├── tokmd-content        - Content scanning helpers (TODOs, duplicates, imports)
├── tokmd-walk          - File listing and asset discovery utilities
├── tokmd-redact        - Redaction utilities for privacy
├── tokmd-tokeignore    - Template generation for .tokeignore files
└── tokmd-fun           - Fun renderers (obj, midi, eco labels)

Tier 3 (Orchestration)
├── tokmd-analysis       - Analysis logic and enrichers
├── tokmd-analysis-types - Analysis receipt contracts
├── tokmd-analysis-format - Formatting and rendering for analysis receipts
└── tokmd-gate          - Policy evaluation engine for CI gating

Tier 4 (Configuration/Facade)
├── tokmd-config        - Configuration schemas and CLI parsing
├── tokmd-core          - High-level API façade for library usage
└── tokmd-format        - Output formatting and serialization

Tier 5 (CLI)
└── tokmd               - Entry point for tokmd CLI application

Language Bindings
├── tokmd-node          - Node.js bindings
└── tokmd-python        - Python bindings
```

### Dependency Graph Summary

```
tokmd (CLI)
├── tokmd-analysis
│   ├── tokmd-analysis-types
│   │   └── tokmd-types
│   ├── tokmd-git (optional)
│   ├── tokmd-walk (optional)
│   └── tokmd-content (optional)
├── tokmd-analysis-format
│   ├── tokmd-analysis-types
│   ├── tokmd-config
│   └── tokmd-fun (optional)
├── tokmd-config
│   └── tokmd-types
├── tokmd-model
│   ├── tokmd-config
│   └── tokmd-types
├── tokmd-scan
│   ├── tokmd-config
│   └── tokmd-types
├── tokmd-format
│   ├── tokmd-config
│   ├── tokmd-redact
│   └── tokmd-types
├── tokmd-tokeignore
│   └── tokmd-config
└── tokmd-gate
    └── tokmd-analysis-types
```

### Key Observations

1. **Well-structured tiering**: The codebase already follows a clear tiered architecture with good separation of concerns.

2. **Feature-gated dependencies**: Many crates use optional features to control dependencies (e.g., tokmd-analysis has optional git, walk, content features).

3. **Large modules in tokmd**: The main CLI crate contains several substantial modules that could be standalone.

4. **Large modules in tokmd-analysis**: The analysis crate contains multiple large modules that could be extracted.

5. **Config coupling**: tokmd-config couples CLI parsing (clap) with configuration schemas, as noted in its own documentation.

---

## 2. Specific Microcrate Extraction Opportunities

### HIGH PRIORITY OPPORTUNITIES

#### 2.1 tokmd-badge

**Module/File Path**: [`crates/tokmd/src/badge.rs`](crates/tokmd/src/badge.rs)

**Reasoning**:
- Completely self-contained with no external dependencies on tokmd internals
- Only depends on tokmd-config for the BadgeMetric enum
- Generates SVG badges for metrics - a distinct, reusable concern
- Could be useful as a standalone library for other tools that need badge generation
- ~140 lines with simple string formatting logic

**Estimated Complexity**: Low

**Dependencies to Manage**:
- tokmd-config (BadgeMetric enum only - could be redefined locally)

**Proposed Crate Structure**:
```
tokmd-badge/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   └── badge.rs
└── README.md
```

**Potential Impact**:
- Reduces tokmd crate size
- Provides a reusable badge generation library
- Minimal refactoring required

---

#### 2.2 tokmd-progress

**Module/File Path**: [`crates/tokmd/src/progress.rs`](crates/tokmd/src/progress.rs)

**Reasoning**:
- Provides a clean abstraction over indicatif with feature-gated UI support
- Completely self-contained with no tokmd-specific logic
- Could be useful for other CLI tools that need progress indicators
- ~130 lines with simple wrapper logic

**Estimated Complexity**: Low

**Dependencies to Manage**:
- indicatif (optional feature)

**Proposed Crate Structure**:
```
tokmd-progress/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   └── progress.rs
└── README.md
```

**Potential Impact**:
- Reduces tokmd crate size
- Provides a reusable progress indicator library
- Minimal refactoring required

---

#### 2.3 tokmd-context-pack

**Module/File Path**: [`crates/tokmd/src/context_pack.rs`](crates/tokmd/src/context_pack.rs)

**Reasoning**:
- Substantial module (~1057 lines) implementing LLM context window optimization
- Contains multiple algorithms (greedy, spread strategies) for file selection
- Distinct concern that could be useful for other LLM-related tools
- Well-tested with comprehensive test coverage

**Estimated Complexity**: Medium

**Dependencies to Manage**:
- tokmd-config (ContextStrategy, ValueMetric)
- tokmd-types (ContextFileRow, FileRow)

**Proposed Crate Structure**:
```
tokmd-context-pack/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── pack.rs      (greedy and spread strategies)
│   └── scoring.rs   (value metrics)
└── README.md
```

**Potential Impact**:
- Significant reduction in tokmd crate size
- Provides a reusable context packing library for LLM workflows
- Requires defining local types or depending on tokmd-types

---

#### 2.4 tokmd-git-scoring

**Module/File Path**: [`crates/tokmd/src/git_scoring.rs`](crates/tokmd/src/git_scoring.rs)

**Reasoning**:
- Git scoring for file ranking is a distinct concern (~386 lines)
- Computes hotspots and commit counts from git history
- Could be useful for other tools that need to rank files by git activity
- Well-tested with comprehensive test coverage

**Estimated Complexity**: Medium

**Dependencies to Manage**:
- tokmd-git
- tokmd-types (FileKind, FileRow)

**Proposed Crate Structure**:
```
tokmd-git-scoring/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── scoring.rs   (hotspot and commit count computation)
│   └── normalize.rs (path normalization utilities)
└── README.md
```

**Potential Impact**:
- Significant reduction in tokmd crate size
- Provides a reusable git scoring library
- Requires careful dependency management

---

#### 2.5 tokmd-tools-schema

**Module/File Path**: [`crates/tokmd/src/tools_schema.rs`](crates/tokmd/src/tools_schema.rs)

**Reasoning**:
- Introspects CLI commands and generates schema output for AI agents
- Distinct concern (~424 lines) that could be useful for other CLI tools
- Supports multiple output formats (jsonschema, openai, anthropic, clap)
- Well-structured with clear separation of concerns

**Estimated Complexity**: Medium

**Dependencies to Manage**:
- clap
- tokmd-config (ToolSchemaFormat)

**Proposed Crate Structure**:
```
tokmd-tools-schema/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── introspect.rs (clap introspection)
│   ├── schema.rs     (schema types)
│   └── render.rs     (output format rendering)
└── README.md
```

**Potential Impact**:
- Significant reduction in tokmd crate size
- Provides a reusable CLI schema generation library
- Requires defining local types or depending on tokmd-config

---

#### 2.6 tokmd-derived-metrics

**Module/File Path**: [`crates/tokmd-analysis/src/derived.rs`](crates/tokmd-analysis/src/derived.rs)

**Reasoning**:
- Substantial module (~745 lines) computing various derived metrics
- Contains multiple metric calculations (ratios, rates, distributions, etc.)
- Distinct concern that could be useful as a standalone library
- Well-tested with property-based testing

**Estimated Complexity**: High

**Dependencies to Manage**:
- tokmd-analysis-types
- tokmd-types

**Proposed Crate Structure**:
```
tokmd-derived-metrics/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── ratio.rs      (ratio calculations)
│   ├── rate.rs       (rate calculations)
│   ├── stats.rs      (statistical calculations)
│   ├── distribution.rs (distribution metrics)
│   └── cocomo.rs     (COCOMO estimation)
└── README.md
```

**Potential Impact**:
- Significant reduction in tokmd-analysis crate size
- Provides a reusable derived metrics library
- Requires careful refactoring to extract shared utilities

---

#### 2.7 tokmd-topics

**Module/File Path**: [`crates/tokmd-analysis/src/topics.rs`](crates/tokmd-analysis/src/topics.rs)

**Reasoning**:
- Builds topic clouds using TF-IDF scoring (~237 lines)
- Distinct concern that could be useful as a standalone library
- Well-tested with comprehensive test coverage
- Could be useful for other text analysis tools

**Estimated Complexity**: Medium

**Dependencies to Manage**:
- tokmd-analysis-types
- tokmd-types

**Proposed Crate Structure**:
```
tokmd-topics/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── tfidf.rs      (TF-IDF scoring)
│   ├── tokenize.rs   (path tokenization)
│   └── stopwords.rs  (stopword handling)
└── README.md
```

**Potential Impact**:
- Moderate reduction in tokmd-analysis crate size
- Provides a reusable topic extraction library
- Requires defining local types or depending on tokmd-analysis-types

---

#### 2.8 tokmd-archetype

**Module/File Path**: [`crates/tokmd-analysis/src/archetype.rs`](crates/tokmd-analysis/src/archetype.rs)

**Reasoning**:
- Identifies project archetypes (~482 lines)
- Distinct concern that could be useful as a standalone library
- Contains multiple archetype detection functions (Rust workspace, Next.js, etc.)
- Well-tested with comprehensive test coverage

**Estimated Complexity**: Medium

**Dependencies to Manage**:
- tokmd-analysis-types
- tokmd-types

**Proposed Crate Structure**:
```
tokmd-archetype/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── detect.rs     (archetype detection)
│   └── patterns.rs   (project pattern matching)
└── README.md
```

**Potential Impact**:
- Moderate reduction in tokmd-analysis crate size
- Provides a reusable project archetype detection library
- Requires defining local types or depending on tokmd-analysis-types

---

### MEDIUM PRIORITY OPPORTUNITIES

#### 2.9 tokmd-analysis-utils

**Module/File Path**: [`crates/tokmd/src/analysis_utils.rs`](crates/tokmd/src/analysis_utils.rs)

**Reasoning**:
- Provides mapping functions between CLI types and analysis types (~293 lines)
- Distinct concern that bridges different type systems
- Could be useful as a standalone library for type conversions
- Well-tested with comprehensive test coverage

**Estimated Complexity**: Low

**Dependencies to Manage**:
- tokmd-analysis
- tokmd-analysis-format
- tokmd-analysis-types
- tokmd-config

**Proposed Crate Structure**:
```
tokmd-analysis-utils/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── convert.rs    (type conversions)
│   └── output.rs     (output handling)
└── README.md
```

**Potential Impact**:
- Moderate reduction in tokmd crate size
- Provides reusable type conversion utilities
- Requires managing multiple dependencies

---

#### 2.10 tokmd-export-bundle

**Module/File Path**: [`crates/tokmd/src/export_bundle.rs`](crates/tokmd/src/export_bundle.rs)

**Reasoning**:
- Handles loading export data from multiple sources (~250 lines)
- Distinct concern that could be useful as a standalone library
- Supports multiple input formats (receipts, JSONL, JSON)
- Well-tested with comprehensive test coverage

**Estimated Complexity**: Medium

**Dependencies to Manage**:
- tokmd-model
- tokmd-scan
- tokmd-types

**Proposed Crate Structure**:
```
tokmd-export-bundle/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── load.rs       (loading strategies)
│   └── meta.rs       (metadata handling)
└── README.md
```

**Potential Impact**:
- Moderate reduction in tokmd crate size
- Provides a reusable export bundle loading library
- Requires managing multiple dependencies

---

#### 2.11 tokmd-math

**Module/File Path**: [`crates/tokmd-analysis/src/util.rs`](crates/tokmd-analysis/src/util.rs)

**Reasoning**:
- Contains shared utility functions like percentile, gini coefficient, and path helpers (~371 lines)
- Mathematical functions could be extracted into a standalone library
- Well-tested with property-based testing
- Could be useful for other analysis tools

**Estimated Complexity**: Medium

**Dependencies to Manage**:
- tokmd-analysis-types

**Proposed Crate Structure**:
```
tokmd-math/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── stats.rs      (statistical functions)
│   ├── percentile.rs (percentile calculation)
│   ├── gini.rs      (gini coefficient)
│   └── path.rs      (path utilities)
└── README.md
```

**Potential Impact**:
- Moderate reduction in tokmd-analysis crate size
- Provides a reusable mathematical utilities library
- Requires extracting only the mathematical functions

---

#### 2.12 tokmd-config-split

**Module/File Path**: [`crates/tokmd-config/src/lib.rs`](crates/tokmd-config/src/lib.rs)

**Reasoning**:
- The config crate couples strict configuration schemas with Clap CLI parsing (~1031 lines)
- The file comment suggests splitting into tokmd-settings (pure config) and tokmd-cli (Clap parsing)
- Would improve separation of concerns and testability

**Estimated Complexity**: High

**Dependencies to Manage**:
- clap (move to tokmd-cli)
- serde, toml (keep in tokmd-settings)
- tokmd-types (keep in both)

**Proposed Crate Structure**:
```
tokmd-settings/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── profile.rs    (profile types)
│   └── toml.rs      (TOML config handling)
└── README.md

tokmd-cli/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── args.rs       (Clap argument types)
│   └── commands.rs   (command definitions)
└── README.md
```

**Potential Impact**:
- Significant improvement in separation of concerns
- Better testability of configuration parsing
- Requires careful refactoring and dependency management

---

### LOW PRIORITY OPPORTUNITIES

#### 2.13 tokmd-interactive

**Module/File Path**: [`crates/tokmd/src/interactive/`](crates/tokmd/src/interactive/)

**Reasoning**:
- Provides interactive prompts and wizards for CLI
- Distinct concern but relatively small
- Could be useful for other CLI tools

**Estimated Complexity**: Low

**Dependencies to Manage**:
- dialoguer
- indicatif
- console (optional feature)

**Proposed Crate Structure**:
```
tokmd-interactive/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── tty.rs        (TTY utilities)
│   └── wizard.rs     (wizard prompts)
└── README.md
```

**Potential Impact**:
- Minor reduction in tokmd crate size
- Provides reusable interactive CLI utilities
- Minimal refactoring required

---

#### 2.14 tokmd-analysis-split

**Module/File Path**: [`crates/tokmd-analysis/src/`](crates/tokmd-analysis/src/)

**Reasoning**:
- The analysis crate contains multiple enrichers that could be split into separate crates
- However, this would be a significant refactoring effort
- The current structure is already well-organized

**Estimated Complexity**: Very High

**Dependencies to Manage**:
- All dependencies of tokmd-analysis
- Would require creating many new crates

**Proposed Crate Structure**:
```
tokmd-enrichers/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── archetype.rs
│   ├── topics.rs
│   ├── churn.rs
│   ├── complexity.rs
│   ├── entropy.rs
│   ├── license.rs
│   └── ...
└── README.md
```

**Potential Impact**:
- Very high refactoring effort
- May not provide significant benefits
- Current structure is already well-organized

---

## 3. Prioritized Recommendations

### High Priority (Immediate Action)

1. **tokmd-badge** - Low complexity, high reusability, minimal refactoring
2. **tokmd-progress** - Low complexity, high reusability, minimal refactoring
3. **tokmd-context-pack** - Medium complexity, high reusability, significant size reduction
4. **tokmd-git-scoring** - Medium complexity, high reusability, significant size reduction
5. **tokmd-tools-schema** - Medium complexity, high reusability, significant size reduction

### Medium Priority (Consider After High Priority)

6. **tokmd-derived-metrics** - High complexity, high reusability, significant size reduction
7. **tokmd-topics** - Medium complexity, high reusability, moderate size reduction
8. **tokmd-archetype** - Medium complexity, high reusability, moderate size reduction
9. **tokmd-analysis-utils** - Low complexity, moderate reusability, moderate size reduction
10. **tokmd-export-bundle** - Medium complexity, moderate reusability, moderate size reduction
11. **tokmd-math** - Medium complexity, moderate reusability, moderate size reduction
12. **tokmd-config-split** - High complexity, high reusability, significant architectural improvement

### Low Priority (Future Consideration)

13. **tokmd-interactive** - Low complexity, moderate reusability, minor size reduction
14. **tokmd-analysis-split** - Very high complexity, questionable reusability, minimal benefits

---

## 4. Potential Challenges and Considerations

### 4.1 Type System Coupling

Many modules depend on types defined in tokmd-config or tokmd-types. Extracting these modules requires:

1. **Defining local types** - This may lead to duplication
2. **Maintaining dependencies** - Keeping dependencies on existing crates
3. **Type conversion** - Adding conversion layers between type systems

**Recommendation**: For high-priority extractions, maintain dependencies on existing crates initially. Consider creating shared type crates if multiple extractions need the same types.

### 4.2 Test Coverage

Extracted modules must maintain test coverage. Consider:

1. **Moving tests with the module** - Tests should be extracted with the code
2. **Integration tests** - May need to be duplicated or refactored
3. **Test utilities** - May need to be shared across crates

**Recommendation**: Extract tests with the modules and add integration tests to the parent crate.

### 4.3 Feature Flags

Many modules use feature flags to control dependencies. Extracting these modules requires:

1. **Preserving feature flags** - Maintain feature-gated behavior
2. **Conditional dependencies** - Manage optional dependencies properly
3. **Documentation** - Document feature behavior clearly

**Recommendation**: Preserve existing feature flags and add new ones as needed for the extracted crate.

### 4.4 API Stability

Extracted crates should have stable APIs for external use. Consider:

1. **Semantic versioning** - Follow semantic versioning for public APIs
2. **Documentation** - Provide comprehensive API documentation
3. **Deprecation policy** - Define deprecation policy for API changes

**Recommendation**: Start with version 0.1.0 and follow semantic versioning.

### 4.5 Workspace Management

Adding new crates to the workspace requires:

1. **Updating Cargo.toml** - Add new crates to workspace members
2. **Version management** - Use workspace versions consistently
3. **Publishing** - Update publishing scripts for new crates

**Recommendation**: Add new crates to the workspace and update publishing scripts.

### 4.6 Circular Dependencies

Extracting modules may introduce circular dependencies. Consider:

1. **Dependency analysis** - Analyze dependencies before extraction
2. **Refactoring** - Refactor to break circular dependencies
3. **Interface crates** - Create interface crates if necessary

**Recommendation**: Analyze dependencies carefully and refactor as needed.

---

## 5. Implementation Roadmap

### Phase 1: Quick Wins (1-2 weeks)

Extract low-complexity, high-reusability modules:

1. Extract tokmd-badge
2. Extract tokmd-progress
3. Update workspace and documentation
4. Test and validate

### Phase 2: Medium Complexity (2-4 weeks)

Extract medium-complexity, high-reusability modules:

1. Extract tokmd-context-pack
2. Extract tokmd-git-scoring
3. Extract tokmd-tools-schema
4. Update workspace and documentation
5. Test and validate

### Phase 3: High Complexity (4-8 weeks)

Extract high-complexity, high-reusability modules:

1. Extract tokmd-derived-metrics
2. Extract tokmd-topics
3. Extract tokmd-archetype
4. Update workspace and documentation
5. Test and validate

### Phase 4: Architectural Improvements (8-12 weeks)

Focus on architectural improvements:

1. Split tokmd-config into tokmd-settings and tokmd-cli
2. Extract tokmd-math
3. Extract tokmd-analysis-utils
4. Extract tokmd-export-bundle
5. Update workspace and documentation
6. Test and validate

### Phase 5: Future Consideration (Ongoing)

Evaluate low-priority opportunities as needed:

1. Extract tokmd-interactive
2. Evaluate tokmd-analysis-split
3. Continue monitoring for new opportunities

---

## 6. Conclusion

The tokmd codebase is well-organized with a clear tiered architecture. However, there are significant opportunities for microcrate extraction that would improve maintainability, reusability, and modularity.

The highest priority extractions are:
- tokmd-badge
- tokmd-progress
- tokmd-context-pack
- tokmd-git-scoring
- tokmd-tools-schema

These extractions are low to medium complexity, high reusability, and would provide significant size reduction to the main tokmd crate.

The medium priority extractions are:
- tokmd-derived-metrics
- tokmd-topics
- tokmd-archetype
- tokmd-analysis-utils
- tokmd-export-bundle
- tokmd-math
- tokmd-config-split

These extractions are medium to high complexity, high reusability, and would provide significant architectural improvements.

The low priority extractions are:
- tokmd-interactive
- tokmd-analysis-split

These extractions are low complexity but provide minimal benefits or require significant refactoring.

Overall, the tokmd codebase would benefit from a phased approach to microcrate extraction, starting with quick wins and moving to more complex extractions over time.
