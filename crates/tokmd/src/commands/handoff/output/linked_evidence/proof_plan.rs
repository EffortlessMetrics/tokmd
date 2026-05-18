//! Proof plan report summary.

use serde_json::Value;

pub(in crate::commands::handoff) struct ProofPlanSummary {
    pub(in crate::commands::handoff) commands: usize,
    pub(in crate::commands::handoff) required: usize,
    pub(in crate::commands::handoff) advisory: usize,
    pub(in crate::commands::handoff) first_commands: Vec<String>,
}

pub(super) fn summarize(value: &Value) -> ProofPlanSummary {
    let Some(commands) = value.get("commands").and_then(Value::as_array) else {
        return ProofPlanSummary {
            commands: 0,
            required: 0,
            advisory: 0,
            first_commands: Vec::new(),
        };
    };
    let required = commands
        .iter()
        .filter(|command| command.get("required").and_then(Value::as_bool) == Some(true))
        .count();
    let advisory = commands.len().saturating_sub(required);
    let first_commands = commands
        .iter()
        .filter_map(|command| command.get("command").and_then(Value::as_str))
        .take(5)
        .map(str::to_string)
        .collect();

    ProofPlanSummary {
        commands: commands.len(),
        required,
        advisory,
        first_commands,
    }
}

pub(super) fn render(out: &mut String, proof_plan: &ProofPlanSummary) {
    out.push_str(&format!(
        "- Proof plan: {} command(s), {} required, {} advisory\n",
        proof_plan.commands, proof_plan.required, proof_plan.advisory
    ));
    if !proof_plan.first_commands.is_empty() {
        out.push_str("  - First commands:\n");
        for command in &proof_plan.first_commands {
            out.push_str(&format!("    - `{command}`\n"));
        }
        if proof_plan.commands > proof_plan.first_commands.len() {
            out.push_str(&format!(
                "    - ... {} more command(s); open the proof plan for the full list.\n",
                proof_plan.commands - proof_plan.first_commands.len()
            ));
        }
    }
    out.push_str("  - A proof plan is planned evidence, not execution proof.\n");
}
