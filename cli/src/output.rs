use std::path::Path;
use anyhow::Result;
use std::fs;
use serde::Serialize;
use crate::types::{Scope, Facts, Summary, FactInstruction};

#[derive(Debug, Serialize)]
struct GlobalFacts<'a> {
    program: &'a str,
    program_id: &'a Option<String>,
    accounts: &'a [crate::types::FactAccount],
    data_structs: &'a [crate::types::FactDataStruct],
    errors: &'a [crate::types::FactError],
    cpi_calls: &'a [crate::types::CpiCall],
    risk_signals: &'a [crate::types::RiskSignal],
    flags: &'a [crate::types::RiskFlag],
    instruction_count: usize,
}

#[derive(Debug, Serialize)]
struct InstructionFactsFile<'a> {
    order: usize,
    #[serde(flatten)]
    instruction: &'a FactInstruction,
}

#[derive(Debug, Serialize)]
struct InstructionFactsIndex {
    program: String,
    instruction_count: usize,
    files: Vec<InstructionFactsIndexEntry>,
}

#[derive(Debug, Serialize)]
struct InstructionFactsIndexEntry {
    order: usize,
    instruction: String,
    context: String,
    file: String,
}

fn sanitize_instruction_name(name: &str) -> String {
    let mut out = String::new();
    let mut last_was_underscore = false;

    for ch in name.chars() {
        let safe = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            '_'
        };

        if safe == '_' {
            if !last_was_underscore {
                out.push('_');
            }
            last_was_underscore = true;
        } else {
            out.push(safe);
            last_was_underscore = false;
        }
    }

    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "instruction".to_string()
    } else {
        trimmed.to_string()
    }
}

fn write_split_facts(out_dir: &Path, facts: &Facts) -> Result<()> {
    let global = GlobalFacts {
        program: &facts.program,
        program_id: &facts.program_id,
        accounts: &facts.accounts,
        data_structs: &facts.data_structs,
        errors: &facts.errors,
        cpi_calls: &facts.cpi_calls,
        risk_signals: &facts.risk_signals,
        flags: &facts.flags,
        instruction_count: facts.instructions.len(),
    };

    let global_json = serde_json::to_string_pretty(&global)?;
    fs::write(out_dir.join("global_facts.json"), global_json)?;

    let facts_dir = out_dir.join("facts");
    if facts_dir.exists() {
        fs::remove_dir_all(&facts_dir)?;
    }
    fs::create_dir_all(&facts_dir)?;

    let width = std::cmp::max(2, facts.instructions.len().to_string().len());
    let mut index_entries = Vec::with_capacity(facts.instructions.len());

    for (idx, instruction) in facts.instructions.iter().enumerate() {
        let order = idx + 1;
        let safe_name = sanitize_instruction_name(&instruction.name);
        let file_name = format!("{:0width$}_{}.json", order, safe_name, width = width);

        let instruction_payload = InstructionFactsFile {
            order,
            instruction,
        };
        let instruction_json = serde_json::to_string_pretty(&instruction_payload)?;
        fs::write(facts_dir.join(&file_name), instruction_json)?;

        index_entries.push(InstructionFactsIndexEntry {
            order,
            instruction: instruction.name.clone(),
            context: instruction.context.clone(),
            file: file_name,
        });
    }

    let index = InstructionFactsIndex {
        program: facts.program.clone(),
        instruction_count: facts.instructions.len(),
        files: index_entries,
    };
    let index_json = serde_json::to_string_pretty(&index)?;
    fs::write(facts_dir.join("index.json"), index_json)?;

    Ok(())
}

pub fn write_outputs(project_root: &Path, scope: &Scope, facts: &Facts, summary: &Summary) -> Result<()> {
    let out_dir = project_root.join(".rust-recon"); // Updated folder name
    if !out_dir.exists() {
        fs::create_dir_all(&out_dir)?;
    }
    
    let scope_json = serde_json::to_string_pretty(scope)?;
    fs::write(out_dir.join("scope.json"), scope_json)?;
    
    let facts_json = serde_json::to_string_pretty(facts)?;
    fs::write(out_dir.join("facts.json"), facts_json)?;

    write_split_facts(&out_dir, facts)?;
    
    let sum_json = serde_json::to_string_pretty(summary)?;
    fs::write(out_dir.join("summary.json"), sum_json)?;
    
    Ok(())
}
