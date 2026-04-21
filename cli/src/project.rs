use std::path::{Path, PathBuf};
use anyhow::Result;
use std::fs;
use toml::Value;

pub fn find_anchor_project() -> Result<PathBuf> {
    let mut current_dir = std::env::current_dir()?;
    
    // Walk up directories to find Anchor.toml
    loop {
        let anchor_toml = current_dir.join("Anchor.toml");
        if anchor_toml.exists() {
            eprintln!("✓ rust-recon: Found Anchor.toml at: {}", current_dir.display());
            return Ok(current_dir);
        }
        
        if !current_dir.pop() {
            break;
        }
    }
    
    anyhow::bail!("Anchor.toml not found in current directory or any parent. Are you in an Anchor project directory?");
}

pub fn detect_programs(project_root: &Path) -> Result<Vec<(String, PathBuf)>> {
    let anchor_toml_path = project_root.join("Anchor.toml");
    let content = fs::read_to_string(&anchor_toml_path)?;
    let toml: Value = toml::from_str(&content)?;
    
    let mut results = Vec::new();
    
    // Try workspace.members first
    if let Some(members) = toml.get("workspace").and_then(|w| w.get("members")).and_then(|m| m.as_array()) {
        for m in members {
            if let Some(m_str) = m.as_str() {
                let path = project_root.join(m_str);
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                if path.exists() {
                    eprintln!("  ✓ Program '{}' at: {}", name, path.display());
                    results.push((name, path));
                } else {
                    eprintln!("  ⚠ Workspace member path does not exist: {}", path.display());
                }
            }
        }
    }
    
    // Fallback: scan programs/ directory
    if results.is_empty() {
        let programs_dir = project_root.join("programs");
        if programs_dir.exists() {
            for entry in fs::read_dir(&programs_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    // Check if this looks like a program (has src/ or Cargo.toml)
                    let has_src = path.join("src").exists();
                    let has_cargo = path.join("Cargo.toml").exists();
                    if has_src || has_cargo {
                        if let Some(name) = path.file_name() {
                            let name_str = name.to_string_lossy().to_string();
                            eprintln!("  ✓ Program '{}' at: {}", name_str, path.display());
                            results.push((name_str, path));
                        }
                    }
                }
            }
        }
    }
    
    if results.is_empty() {
        eprintln!("  ⚠ No programs detected. Check Anchor.toml workspace.members or programs/ directory.");
    }
    
    Ok(results)
}

