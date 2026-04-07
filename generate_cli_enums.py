import re

with open('crates/tokmd-types/src/lib.rs', 'r') as f:
    text = f.read()

enums_to_extract = [
    'TableFormat', 'ExportFormat', 'ConfigMode', 'ChildrenMode', 'ChildIncludeMode', 'RedactMode', 'AnalysisFormat'
]

out = """use clap::ValueEnum;
use serde::{Deserialize, Serialize};

"""

for enum_name in enums_to_extract:
    match = re.search(r'pub enum ' + enum_name + r' \{(.*?)\}', text, re.DOTALL)
    if not match:
        print("Failed to find", enum_name)
        continue
    body = match.group(1)

    variants = []
    lines = body.split('\n')
    for line in lines:
        line = line.strip()
        if not line or line.startswith('//') or line.startswith('///'):
            continue
        if line.startswith('#'):
            continue
        variant = line.split(',')[0].strip()
        if variant:
            variants.append(variant)

    is_default = "ConfigMode" in enum_name
    default_attr = ", Default" if is_default else ""

    out += f"#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum{default_attr})]\n"
    out += f"#[serde(rename_all = \"kebab-case\")]\n"
    out += f"pub enum Cli{enum_name} {{\n"
    for v in variants:
        if is_default and v == "Auto":
            out += f"    #[default]\n"
        out += f"    {v},\n"
    out += f"}}\n\n"

    out += f"impl From<Cli{enum_name}> for tokmd_types::{enum_name} {{\n"
    out += f"    fn from(v: Cli{enum_name}) -> Self {{\n"
    out += f"        match v {{\n"
    for v in variants:
        out += f"            Cli{enum_name}::{v} => tokmd_types::{enum_name}::{v},\n"
    out += f"        }}\n"
    out += f"    }}\n"
    out += f"}}\n\n"

with open('crates/tokmd-config/src/cli_enums.rs', 'w') as f:
    f.write(out)
