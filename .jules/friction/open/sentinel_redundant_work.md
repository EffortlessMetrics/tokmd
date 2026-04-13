# Friction Item

**Date:** 2024-04-07
**Persona:** Sentinel

## Context
I was tasked with finding a security-significant boundary hardening fix within `core-pipeline` around `RedactMode::All`. I successfully discovered that `module_roots` arrays leaked structure inside formatting payloads.

## Issue
The exact target behavior (redacting `module_roots` under `RedactMode::All`) had already landed in `main` via another PR before this task concluded. Attempting to add it became duplicate carrier work.

## Implication
I had to abort the code patch and pivot to a learning PR.
