# Clearer CLI Error Messages

## Context
CLI error messages, especially when providing examples or usage instructions, should be formatted for easy reading.

## Pattern
When outputting a long block of text in `bail!`, use:
- Blank lines `\n\n` to separate logical sections.
- Numbered lists `1. `, `2. ` for sequences of options or steps.
- Indentation for code examples.

## Example
Instead of:
```
"No policy or ratchet rules specified.\n\
 \n\
 Use --policy <path> for policy rules, or\n\
 --baseline <path> with --ratchet-config <path> for ratchet rules, or\n\
 add rules to [gate] in tokmd.toml."
```

Use:
```
"No policy or ratchet rules specified.\n\n\
 How to fix this:\n\
 1. Provide a policy file: `tokmd gate --policy rules.toml`\n\
 2. Or provide a ratchet baseline: `tokmd gate --baseline old.json --ratchet-config limits.toml`\n\
 3. Or configure rules in your `tokmd.toml`"
```
