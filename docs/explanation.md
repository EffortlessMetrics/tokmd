# Explanation: The Philosophy of Receipts

`tokmd` is designed as a **sensor** for agentic workflows. Its goal is to shift context left: from messy, chatty exploration to structured, deterministic artifacts.

## The Context: Agentic Swarms

In a gated multi-agent workflow, the first step is **Explore → Inventory + Plan**. `tokmd` provides the inventory artifact.

It addresses specific failure modes in AI-native development:

1.  **Context Starvation**: Giving an LLM a cheap, structured map of the repo so it can steer without dumping the whole codebase into the prompt.
2.  **Process Confabulation**: The tendency to "narrate" work (e.g., "I checked the files") without evidence. `tokmd` receipts provide the evidence that crosses the **trust boundary**.

## The Trust Boundary

Text is untrusted; artifacts are trusted.

- **Chats** are ephemeral and hallucination-prone.
- **Receipts** are deterministic, versioned, and machine-verifiable.

By generating a `tokmd` receipt, you create a stable artifact that can be diffed, cached, and validated by pipelines. This enables the shift from "chatting with code" to "reasoning about artifacts."

## Safety: "If you wouldn't email it, don't paste"

`tokmd` adopts a "boring compliance" posture for AI safety.

- **Redaction**: Support for hashing paths and module names allows you to share the *shape* of your repo (architecture, complexity, distribution) with a hosted LLM without leaking sensitive internal identifiers.
- **Model Copies**: We prefer working with structural models (lists, stats) over raw data to minimize surface area.

## Why "Shape, Not Grade"?

`tokmd` explicitly rejects the use of LOC as a productivity metric. In AI-native repos, velocity is easy to game. `tokmd` is for understanding the **shape** of the codebase:

- Where is the complexity accumulating?
- What are we actually vending vs. writing?
- How does the repo look to a machine?
