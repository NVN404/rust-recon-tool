use std::path::Path;
use anyhow::Result;
use std::fs;
use crate::types::ScopeInstruction;

pub fn read_idl_if_exists(project_root: &Path, program: &str) -> Result<Option<serde_json::Value>> {
    let idl_path = project_root.join(format!("target/idl/{}.json", program));
    if !idl_path.exists() {
        return Ok(None);
    }
    
    let contents = fs::read_to_string(idl_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&contents)?;
    Ok(Some(parsed))
}

pub fn extract_instructions_from_idl(idl: &serde_json::Value) -> Vec<ScopeInstruction> {
    let mut instrs = Vec::new();
    
    if let Some(instructions) = idl.get("instructions").and_then(|v| v.as_array()) {
        for ix in instructions {
            if let Some(name) = ix.get("name").and_then(|v| v.as_str()) {
                // Heuristic: capitalise first letter for context
                let mut chars = name.chars();
                let context = match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                };
                
                instrs.push(ScopeInstruction {
                    ix: name.to_string(),
                    context,
                });
            }
        }
    }
    
    instrs
}
