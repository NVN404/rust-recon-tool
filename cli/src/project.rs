use std::path::{Path, PathBuf};
use anyhow::Result;
use std::fs;
use toml::Value;

pub fn find_anchor_project() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let anchor_toml = current_dir.join("Anchor.toml");
    if anchor_toml.exists() {
        return Ok(current_dir);
    }
    
    anyhow::bail!("Anchor.toml not found. Are you in an Anchor project directory?");
}

pub fn detect_programs(project_root: &Path) -> Result<Vec<(String, PathBuf)>> {
    let anchor_toml_path = project_root.join("Anchor.toml");
    let content = fs::read_to_string(&anchor_toml_path)?;
    let toml: Value = toml::from_str(&content)?;
    
    let mut results = Vec::new();
    
    if let Some(members) = toml.get("workspace").and_then(|w| w.get("members")).and_then(|m| m.as_array()) {
        for m in members {
            if let Some(m_str) = m.as_str() {
                let path = project_root.join(m_str);
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                results.push((name, path));
            }
        }
    } else {
        let programs_dir = project_root.join("programs");
        if programs_dir.exists() {
            for entry in fs::read_dir(programs_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        results.push((name.to_string_lossy().to_string(), path));
                    }
                }
            }
        }
    }
    
    Ok(results)
}
