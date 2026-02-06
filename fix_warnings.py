import re

file_path = 'crates/tokmd/src/commands/cockpit.rs'

with open(file_path, 'r') as f:
    content = f.read()

# 1. Fix args warning
# Replace `fn handle(args: cli::CockpitArgs` with `fn handle(#[cfg_attr(not(feature = "git"), allow(unused))] args: cli::CockpitArgs`
# Or just suppress unused variables on the function
content = content.replace(
    'pub(crate) fn handle(args: cli::CockpitArgs',
    'pub(crate) fn handle(#[cfg_attr(not(feature = "git"), allow(unused))] args: cli::CockpitArgs'
)

# 2. Fix Context warning
# Replace `use anyhow::{Context, Result, bail};`
# with split imports
content = content.replace(
    'use anyhow::{Context, Result, bail};',
    'use anyhow::{Result, bail};\n#[cfg(feature = "git")]\nuse anyhow::Context;'
)

with open(file_path, 'w') as f:
    f.write(content)

print("Warnings fixed.")
