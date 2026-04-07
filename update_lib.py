import sys

content = open('crates/tokmd-analysis-types/src/lib.rs', 'r').read()

search = """\
            let mut fallback_metrics = BaselineMetrics::default();
            fallback_metrics.total_code_lines = total_code_lines;
            fallback_metrics.total_files = total_files;"""

replace = """\
            let fallback_metrics = BaselineMetrics {
                total_code_lines,
                total_files,
                ..Default::default()
            };"""

if search in content:
    content = content.replace(search, replace)
    with open('crates/tokmd-analysis-types/src/lib.rs', 'w') as f:
        f.write(content)
    print("Updated lib.rs.")
else:
    print("Could not find search string.")
