# Decision

## Target
Contract drift around `TOOL_SCHEMA_VERSION = 1` inside `crates/tokmd-tool-schema/`

## Options
- **Option A**: Fix the schema documentation to include `TOOL_SCHEMA_VERSION = 1`, align the definitions in `docs/schema.json` with the rust struct implementation of `ToolSchemaOutput` and add it to `docs/SCHEMA.md` and `docs/architecture.md`.
- **Option B**: Produce a learning PR because the contract fix was already applied on main.

## Choice
**Option B**. The original fix was found to already be present on main. Generating a learning PR to record the friction item.
