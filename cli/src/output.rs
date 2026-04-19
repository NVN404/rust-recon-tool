use std::path::Path;
use anyhow::Result;
use std::fs;
use crate::types::{Scope, Facts, Summary};

pub fn write_outputs(project_root: &Path, scope: &Scope, facts: &Facts, summary: &Summary) -> Result<()> {
    let out_dir = project_root.join(".rust-recon"); // Updated folder name
    if !out_dir.exists() {
        fs::create_dir_all(&out_dir)?;
    }
    
    let scope_json = serde_json::to_string_pretty(scope)?;
    fs::write(out_dir.join("scope.json"), scope_json)?;
    
    let facts_json = serde_json::to_string_pretty(facts)?;
    fs::write(out_dir.join("facts.json"), facts_json)?;
    
    let sum_json = serde_json::to_string_pretty(summary)?;
    fs::write(out_dir.join("summary.json"), sum_json)?;
    
    Ok(())
}
