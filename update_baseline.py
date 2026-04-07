import sys

content = open('crates/tokmd-analysis-types/src/lib.rs', 'r').read()

search_else = """\
            (metrics, files, Some(complexity_section))
        } else {
            let mut fallback_metrics = BaselineMetrics::default();
            fallback_metrics.total_code_lines = total_code_lines;
            fallback_metrics.total_files = total_files;
            (fallback_metrics, Vec::new(), None)
        };"""

replace_else = """\
            (metrics, files, Some(complexity_section))
        } else {
            let mut fallback_metrics = BaselineMetrics::default();
            fallback_metrics.total_code_lines = total_code_lines;
            fallback_metrics.total_files = total_files;
            (fallback_metrics, Vec::new(), None)
        };"""

# I need to use the derived totals without instantiating full DerivedReport in the test
