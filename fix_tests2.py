import re
import os

for root, _, files in os.walk('crates/tokmd-config/tests'):
    for f in files:
        if not f.endswith('.rs'): continue
        path = os.path.join(root, f)
        with open(path, 'r') as fp:
            text = fp.read()

        text = text.replace('tokmd_config::RedactMode', 'tokmd_config::CliRedactMode')
        text = text.replace('tokmd_config::TableFormat', 'tokmd_config::CliTableFormat')
        text = text.replace('tokmd_config::ChildrenMode', 'tokmd_config::CliChildrenMode')
        text = text.replace('tokmd_config::ExportFormat', 'tokmd_config::CliExportFormat')
        text = text.replace('tokmd_config::AnalysisFormat', 'tokmd_config::CliAnalysisFormat')
        text = text.replace('tokmd_config::ChildIncludeMode', 'tokmd_config::CliChildIncludeMode')
        text = text.replace('tokmd_config::ConfigMode', 'tokmd_config::CliConfigMode')

        text = text.replace('use tokmd_config::{', 'use tokmd_config::{')

        text = re.sub(r'\bRedactMode\b', 'CliRedactMode', text)
        text = re.sub(r'\bTableFormat\b', 'CliTableFormat', text)
        text = re.sub(r'\bChildrenMode\b', 'CliChildrenMode', text)
        text = re.sub(r'\bExportFormat\b', 'CliExportFormat', text)
        text = re.sub(r'\bAnalysisFormat\b', 'CliAnalysisFormat', text)
        text = re.sub(r'\bChildIncludeMode\b', 'CliChildIncludeMode', text)
        text = re.sub(r'\bConfigMode\b', 'CliConfigMode', text)

        # undo any accidental tokmd_config::CliCli changes
        text = text.replace('CliCli', 'Cli')
        text = text.replace('tokmd_types::Cli', 'tokmd_types::')

        with open(path, 'w') as fp:
            fp.write(text)

with open('crates/tokmd-config/tests/config_depth_w60.rs', 'r') as f:
    text = f.read()
text = text.replace('opts.config, CliConfigMode::None', 'opts.config, tokmd_types::ConfigMode::None')
with open('crates/tokmd-config/tests/config_depth_w60.rs', 'w') as f:
    f.write(text)

with open('crates/tokmd-config/tests/deep.rs', 'r') as f:
    text = f.read()
text = text.replace('tokmd_config::CliConfigMode::None', 'tokmd_types::ConfigMode::None')
text = text.replace('config: tokmd_types::ConfigMode::None,', 'config: tokmd_config::CliConfigMode::None,')
with open('crates/tokmd-config/tests/deep.rs', 'w') as f:
    f.write(text)

with open('crates/tokmd-config/tests/deep_config_round2_w52.rs', 'r') as f:
    text = f.read()
text = text.replace('tokmd_settings::Cli', 'tokmd_settings::')
with open('crates/tokmd-config/tests/deep_config_round2_w52.rs', 'w') as f:
    f.write(text)

with open('crates/tokmd-config/tests/config_deep_w75.rs', 'r') as f:
    text = f.read()
text = text.replace('opts.config, CliConfigMode::None', 'opts.config, tokmd_types::ConfigMode::None')
with open('crates/tokmd-config/tests/config_deep_w75.rs', 'w') as f:
    f.write(text)

with open('crates/tokmd-config/tests/deep_config_w47.rs', 'r') as f:
    text = f.read()
text = text.replace('config: CliConfigMode::Auto', 'config: tokmd_types::ConfigMode::Auto')
with open('crates/tokmd-config/tests/deep_config_w47.rs', 'w') as f:
    f.write(text)

with open('crates/tokmd-config/tests/scenarios.rs', 'r') as f:
    text = f.read()
text = text.replace('opts.config, CliConfigMode::None', 'opts.config, tokmd_types::ConfigMode::None')
text = text.replace('opts2.config, CliConfigMode::Auto', 'opts2.config, tokmd_types::ConfigMode::Auto')
with open('crates/tokmd-config/tests/scenarios.rs', 'w') as f:
    f.write(text)

with open('crates/tokmd-config/tests/bdd.rs', 'r') as f:
    text = f.read()
text = text.replace('opts.config, CliConfigMode::None', 'opts.config, tokmd_types::ConfigMode::None')
with open('crates/tokmd-config/tests/bdd.rs', 'w') as f:
    f.write(text)

with open('crates/tokmd-config/tests/deep_w66.rs', 'r') as f:
    text = f.read()
text = text.replace('ConfigMode::Auto', 'tokmd_config::CliConfigMode::Auto')
with open('crates/tokmd-config/tests/deep_w66.rs', 'w') as f:
    f.write(text)

with open('crates/tokmd-config/tests/proptest_expansion_w50.rs', 'r') as f:
    text = f.read()
text = text.replace('tokmd_config::RedactMode::', 'tokmd_config::CliRedactMode::')
with open('crates/tokmd-config/tests/proptest_expansion_w50.rs', 'w') as f:
    f.write(text)
