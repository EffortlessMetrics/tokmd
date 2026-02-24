//! Tool-schema generation for AI agent tool use.
//!
//! This crate introspects a clap `Command` tree and produces schema output in
//! formats commonly consumed by AI tooling.

use anyhow::Result;
use clap::{Arg, ArgAction, Command};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// Output format for rendered tool schemas.
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ToolSchemaFormat {
    /// OpenAI function calling format.
    Openai,
    /// Anthropic tool use format.
    Anthropic,
    /// JSON Schema Draft 7 format.
    #[default]
    Jsonschema,
    /// Raw clap structure dump.
    Clap,
}

/// Schema version for tool definitions.
pub const TOOL_SCHEMA_VERSION: u32 = 1;

/// Top-level schema output with envelope metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchemaOutput {
    /// Schema version.
    pub schema_version: u32,

    /// Tool name.
    pub name: String,

    /// Tool version.
    pub version: String,

    /// Tool description.
    pub description: String,

    /// Available commands/tools.
    pub tools: Vec<ToolDefinition>,
}

/// Definition of a single command/tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Command name.
    pub name: String,

    /// Command description.
    pub description: String,

    /// Parameters/arguments.
    pub parameters: Vec<ParameterSchema>,
}

/// Schema for a single parameter/argument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    /// Parameter name.
    pub name: String,

    /// Parameter description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Parameter type.
    #[serde(rename = "type")]
    pub param_type: String,

    /// Whether the parameter is required.
    pub required: bool,

    /// Default value if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Enum values if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
}

/// Build the tool schema from a clap `Command`.
pub fn build_tool_schema(cmd: &Command) -> ToolSchemaOutput {
    let mut tools = Vec::new();

    // Add the root command as a tool (for default lang mode).
    tools.push(build_tool_definition(cmd, None));

    // Add all subcommands.
    for subcmd in cmd.get_subcommands() {
        // Skip generated help subcommand.
        let name = subcmd.get_name();
        if name == "help" {
            continue;
        }
        tools.push(build_tool_definition(subcmd, Some(name)));
    }

    ToolSchemaOutput {
        schema_version: TOOL_SCHEMA_VERSION,
        name: cmd.get_name().to_string(),
        version: cmd.get_version().unwrap_or("unknown").to_string(),
        description: cmd.get_about().map(|s| s.to_string()).unwrap_or_default(),
        tools,
    }
}

/// Build a tool definition from a command.
fn build_tool_definition(cmd: &Command, name_override: Option<&str>) -> ToolDefinition {
    let name = name_override.unwrap_or(cmd.get_name()).to_string();
    let description = cmd.get_about().map(|s| s.to_string()).unwrap_or_default();

    let mut parameters = Vec::new();

    // Add arguments.
    for arg in cmd.get_arguments() {
        // Skip generated args.
        if arg.get_id() == "help" || arg.get_id() == "version" {
            continue;
        }
        parameters.push(build_parameter_schema(arg));
    }

    ToolDefinition {
        name,
        description,
        parameters,
    }
}

/// Build a parameter schema from a clap `Arg`.
fn build_parameter_schema(arg: &Arg) -> ParameterSchema {
    let name = arg.get_id().to_string();
    let description = arg.get_help().map(|s| s.to_string());

    // Determine type based on action and value hints.
    let param_type = determine_param_type(arg);

    // Check if required.
    let required = arg.is_required_set();

    // Get default value.
    let default = arg
        .get_default_values()
        .first()
        .map(|v| v.to_string_lossy().to_string());

    // Get enum values if applicable.
    let enum_values = arg
        .get_possible_values()
        .iter()
        .map(|v| v.get_name().to_string())
        .collect::<Vec<_>>();
    let enum_values = if enum_values.is_empty() {
        None
    } else {
        Some(enum_values)
    };

    ParameterSchema {
        name,
        description,
        param_type,
        required,
        default,
        enum_values,
    }
}

/// Determine the parameter type from a clap `Arg`.
fn determine_param_type(arg: &Arg) -> String {
    match arg.get_action() {
        ArgAction::SetTrue | ArgAction::SetFalse => "boolean".to_string(),
        ArgAction::Count => "integer".to_string(),
        ArgAction::Append => "array".to_string(),
        _ => "string".to_string(),
    }
}

/// Render the schema output in the specified format.
pub fn render_output(
    schema: &ToolSchemaOutput,
    format: ToolSchemaFormat,
    pretty: bool,
) -> Result<String> {
    match format {
        ToolSchemaFormat::Jsonschema => render_jsonschema(schema, pretty),
        ToolSchemaFormat::Openai => render_openai(schema, pretty),
        ToolSchemaFormat::Anthropic => render_anthropic(schema, pretty),
        ToolSchemaFormat::Clap => render_clap(schema, pretty),
    }
}

/// Render as JSON Schema format.
fn render_jsonschema(schema: &ToolSchemaOutput, pretty: bool) -> Result<String> {
    let tools_schema: Vec<Value> = schema
        .tools
        .iter()
        .map(|tool| {
            let properties: BTreeMap<String, Value> = tool
                .parameters
                .iter()
                .map(|p| {
                    let mut prop = json!({
                        "type": p.param_type,
                    });

                    if let Some(desc) = &p.description {
                        prop["description"] = json!(desc);
                    }
                    if let Some(def) = &p.default {
                        prop["default"] = json!(def);
                    }
                    if let Some(enums) = &p.enum_values {
                        prop["enum"] = json!(enums);
                    }

                    (p.name.clone(), prop)
                })
                .collect();

            let required: Vec<&str> = tool
                .parameters
                .iter()
                .filter(|p| p.required)
                .map(|p| p.name.as_str())
                .collect();

            json!({
                "name": tool.name,
                "description": tool.description,
                "parameters": {
                    "type": "object",
                    "properties": properties,
                    "required": required,
                }
            })
        })
        .collect();

    let output = json!({
        "$schema": "https://json-schema.org/draft-07/schema#",
        "schema_version": schema.schema_version,
        "name": schema.name,
        "version": schema.version,
        "description": schema.description,
        "tools": tools_schema,
    });

    if pretty {
        Ok(serde_json::to_string_pretty(&output)?)
    } else {
        Ok(serde_json::to_string(&output)?)
    }
}

/// Render in OpenAI function calling format.
fn render_openai(schema: &ToolSchemaOutput, pretty: bool) -> Result<String> {
    let functions: Vec<Value> = schema
        .tools
        .iter()
        .map(|tool| {
            let properties: BTreeMap<String, Value> = tool
                .parameters
                .iter()
                .map(|p| {
                    let mut prop = json!({
                        "type": p.param_type,
                    });

                    if let Some(desc) = &p.description {
                        prop["description"] = json!(desc);
                    }
                    if let Some(enums) = &p.enum_values {
                        prop["enum"] = json!(enums);
                    }

                    (p.name.clone(), prop)
                })
                .collect();

            let required: Vec<&str> = tool
                .parameters
                .iter()
                .filter(|p| p.required)
                .map(|p| p.name.as_str())
                .collect();

            json!({
                "name": tool.name,
                "description": tool.description,
                "parameters": {
                    "type": "object",
                    "properties": properties,
                    "required": required,
                }
            })
        })
        .collect();

    let output = json!({
        "functions": functions,
    });

    if pretty {
        Ok(serde_json::to_string_pretty(&output)?)
    } else {
        Ok(serde_json::to_string(&output)?)
    }
}

/// Render in Anthropic tool use format.
fn render_anthropic(schema: &ToolSchemaOutput, pretty: bool) -> Result<String> {
    let tools: Vec<Value> = schema
        .tools
        .iter()
        .map(|tool| {
            let properties: BTreeMap<String, Value> = tool
                .parameters
                .iter()
                .map(|p| {
                    let mut prop = json!({
                        "type": p.param_type,
                    });

                    if let Some(desc) = &p.description {
                        prop["description"] = json!(desc);
                    }
                    if let Some(enums) = &p.enum_values {
                        prop["enum"] = json!(enums);
                    }

                    (p.name.clone(), prop)
                })
                .collect();

            let required: Vec<&str> = tool
                .parameters
                .iter()
                .filter(|p| p.required)
                .map(|p| p.name.as_str())
                .collect();

            json!({
                "name": tool.name,
                "description": tool.description,
                "input_schema": {
                    "type": "object",
                    "properties": properties,
                    "required": required,
                }
            })
        })
        .collect();

    let output = json!({
        "tools": tools,
    });

    if pretty {
        Ok(serde_json::to_string_pretty(&output)?)
    } else {
        Ok(serde_json::to_string(&output)?)
    }
}

/// Render raw clap structure (for debugging).
fn render_clap(schema: &ToolSchemaOutput, pretty: bool) -> Result<String> {
    if pretty {
        Ok(serde_json::to_string_pretty(schema)?)
    } else {
        Ok(serde_json::to_string(schema)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_cmd() -> Command {
        Command::new("test")
            .version("1.0.0")
            .about("Test command")
            .subcommand(
                Command::new("sub")
                    .about("Subcommand")
                    .arg(Arg::new("flag").long("flag").action(ArgAction::SetTrue))
                    .arg(
                        Arg::new("value")
                            .long("value")
                            .required(true)
                            .help("A value"),
                    ),
            )
    }

    #[test]
    fn build_schema_includes_subcommands() {
        let cmd = make_test_cmd();
        let schema = build_tool_schema(&cmd);

        assert_eq!(schema.name, "test");
        assert_eq!(schema.version, "1.0.0");
        assert!(!schema.tools.is_empty());

        let sub = schema
            .tools
            .iter()
            .find(|tool| tool.name == "sub")
            .expect("subcommand should exist");
        assert_eq!(sub.parameters.len(), 2);
    }

    #[test]
    fn render_openai_has_functions_key() {
        let cmd = make_test_cmd();
        let schema = build_tool_schema(&cmd);
        let output = render_output(&schema, ToolSchemaFormat::Openai, false).unwrap();

        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.get("functions").is_some());
    }

    #[test]
    fn render_anthropic_has_input_schema() {
        let cmd = make_test_cmd();
        let schema = build_tool_schema(&cmd);
        let output = render_output(&schema, ToolSchemaFormat::Anthropic, false).unwrap();

        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.get("tools").is_some());
        let tools = parsed["tools"].as_array().unwrap();
        assert!(tools.iter().any(|tool| tool.get("input_schema").is_some()));
    }
}
