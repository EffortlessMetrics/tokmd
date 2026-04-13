# Persona Duplication

When reviewing persona files (e.g., `README.md` in `.jules/personas/*/`), do not attempt to deduplicate shared boilerplate like the `## Notes` section into a central file like `.jules/README.md`.

Even though the text is identical across 16 files, it is **intentionally duplicated**. Agents often only receive their specific persona file in their context window. If the instruction is moved to a central file, the agent might not see it. Prompt-critical guidance must stay in the individual persona files.
