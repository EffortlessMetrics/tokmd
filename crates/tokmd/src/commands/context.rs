use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use tokmd_config as cli;
use tokmd_model as model;
use tokmd_scan as scan;
use tokmd_types::{ContextReceipt, ToolInfo};

use crate::context_pack;

pub(crate) fn handle(args: cli::CliContextArgs, global: &cli::GlobalArgs) -> Result<()> {
    let paths = args.paths.clone().unwrap_or_else(|| vec![PathBuf::from(".")]);

    // Parse budget
    let budget = context_pack::parse_budget(&args.budget)?;

    // Scan and create export data
    let languages = scan::scan(&paths, global)?;
    let module_roots = args.module_roots.clone().unwrap_or_default();
    let module_depth = args.module_depth.unwrap_or(2);

    let export = model::create_export_data(
        &languages,
        &module_roots,
        module_depth,
        cli::ChildIncludeMode::ParentsOnly,
        None,
        0,    // no min_code filter
        0,    // no max_rows limit
    );

    // Select files based on strategy
    let selected = context_pack::select_files(
        &export.rows,
        budget,
        args.strategy,
        args.rank_by,
    );

    let used_tokens: usize = selected.iter().map(|f| f.tokens).sum();
    let utilization = if budget > 0 {
        (used_tokens as f64 / budget as f64) * 100.0
    } else {
        0.0
    };

    match args.output {
        cli::ContextOutput::List => {
            println!("# Context Pack");
            println!();
            println!("Budget: {} tokens", budget);
            println!("Used: {} tokens ({:.1}%)", used_tokens, utilization);
            println!("Files: {}", selected.len());
            println!("Strategy: {:?}", args.strategy);
            println!();
            println!("|Path|Module|Lang|Tokens|Code|");
            println!("|---|---|---|---:|---:|");
            for file in &selected {
                println!("|{}|{}|{}|{}|{}|", file.path, file.module, file.lang, file.tokens, file.code);
            }
        }
        cli::ContextOutput::Bundle => {
            for file in &selected {
                // Read file and output
                let path = PathBuf::from(&file.path);
                if path.exists() {
                    println!("// === {} ===", file.path);
                    if args.compress {
                        // Strip comments and blank lines (simple heuristic)
                        if let Ok(f) = fs::File::open(&path) {
                            let reader = BufReader::new(f);
                            for line in reader.lines().map_while(Result::ok) {
                                let trimmed = line.trim();
                                // Skip blank lines
                                if trimmed.is_empty() {
                                    continue;
                                }
                                // Skip common comment patterns (simple heuristic)
                                if trimmed.starts_with("//")
                                    || trimmed.starts_with('#')
                                    || trimmed.starts_with("/*")
                                    || trimmed.starts_with('*')
                                    || trimmed.starts_with("'''")
                                    || trimmed.starts_with("\"\"\"")
                                {
                                    continue;
                                }
                                println!("{}", line);
                            }
                        }
                    } else if let Ok(content) = fs::read_to_string(&path) {
                        print!("{}", content);
                        if !content.ends_with('\n') {
                            println!();
                        }
                    }
                    println!();
                }
            }
        }
        cli::ContextOutput::Json => {
            let receipt = ContextReceipt {
                schema_version: tokmd_types::SCHEMA_VERSION,
                generated_at_ms: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis(),
                tool: ToolInfo::current(),
                mode: "context".to_string(),
                budget_tokens: budget,
                used_tokens,
                utilization_pct: utilization,
                strategy: format!("{:?}", args.strategy).to_lowercase(),
                rank_by: format!("{:?}", args.rank_by).to_lowercase(),
                file_count: selected.len(),
                files: selected,
            };
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
    }

    Ok(())
}
