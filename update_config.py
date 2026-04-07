import re
with open('crates/tokmd-config/src/lib.rs', 'r') as f:
    text = f.read()

replacements = {
    r'format: Option<TableFormat>': 'format: Option<CliTableFormat>',
    r'format: Option<ExportFormat>': 'format: Option<CliExportFormat>',
    r'format: Option<AnalysisFormat>': 'format: Option<CliAnalysisFormat>',
    r'children: Option<ChildrenMode>': 'children: Option<CliChildrenMode>',
    r'children: Option<ChildIncludeMode>': 'children: Option<CliChildIncludeMode>',
    r'redact: Option<RedactMode>': 'redact: Option<CliRedactMode>',
    r'config: ConfigMode,': 'config: CliConfigMode,',
    r'format: Some\(TableFormat::Json\)': 'format: Some(CliTableFormat::Json)',
    r'redact: Some\(RedactMode::All\)': 'redact: Some(CliRedactMode::All)',
    r'config: ConfigMode::None,': 'config: CliConfigMode::None,',
    r'config: ConfigMode::Auto,': 'config: CliConfigMode::Auto,',
    r'g\.config,': 'g.config.into(),',
    r'default_value_t = ConfigMode::Auto': 'default_value_t = CliConfigMode::Auto',
}
for k, v in replacements.items():
    text = re.sub(k, v, text)
with open('crates/tokmd-config/src/lib.rs', 'w') as f:
    f.write(text)
