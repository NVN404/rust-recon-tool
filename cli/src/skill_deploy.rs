use std::path::Path;
use anyhow::{Result, Context};
use std::fs;

/// Default global path where the rust-recon skill repo is cloned
const SKILL_REPO_DIR: &str = ".rust-recon-skill";

/// Finds the skill repo directory. Checks:
/// 1. Environment variable RUST_RECON_SKILL_PATH (override)
/// 2. ~/.rust-recon-skill (canonical install location)
fn find_skill_repo() -> Result<std::path::PathBuf> {
    // Check env override first
    if let Ok(custom_path) = std::env::var("RUST_RECON_SKILL_PATH") {
        let p = std::path::PathBuf::from(&custom_path);
        if p.join("skill").join("core.md").exists() {
            return Ok(p);
        }
        eprintln!("Warning: RUST_RECON_SKILL_PATH={} does not contain skill/core.md", custom_path);
    }
    
    // Check canonical location
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let canonical = home.join(SKILL_REPO_DIR);
    if canonical.join("skill").join("core.md").exists() {
        return Ok(canonical);
    }
    
    anyhow::bail!(
        "rust-recon skill repo not found.\n\
         Expected at: ~/.rust-recon-skill/\n\
         Install with: git clone https://github.com/NVN404/rust-recon ~/.rust-recon-skill\n\
         Or set RUST_RECON_SKILL_PATH environment variable."
    )
}

/// Deploy a minimal AI agent pointer into the target Anchor project.
/// This copies the actual skill instructions into .rust-recon/skill/ (hidden from the user's project root)
/// so that heavily sandboxed AI agents can read them locally without failing on absolute paths.
pub fn deploy_skill_configs(project_root: &Path) -> Result<()> {
    let skill_repo = find_skill_repo()?;
    let global_skill_dir = skill_repo.join("skill");
    
    // Create the local .rust-recon/skill/ directory in the workspace
    let local_recon_dir = project_root.join(".rust-recon");
    let local_skill_dir = local_recon_dir.join("skill");
    
    fs::create_dir_all(&local_skill_dir)?;
    
    // Copy all skill files into the local workspace so the sandboxed AI can read them
    let files_to_copy = vec![
        "core.md",
        "guardrails.md",
        "SKILLS.md",
        "references/audit-patterns.md",
        "references/cpi-rules.md",
        "references/facts-schema.md",
        "references/section-specs.md",
    ];
    
    fs::create_dir_all(local_skill_dir.join("references"))?;
    
    for file in files_to_copy {
        let src = global_skill_dir.join(file);
        let dst = local_skill_dir.join(file);
        if src.exists() {
            fs::copy(&src, &dst)?;
        }
    }
    
    // Use relative paths for the pointers since the files are now in the workspace
     let pointer_content = r#"# rust-recon Skill Pointer (One-Command Recon)

<critical_directives>
1. YOU ARE FORBIDDEN FROM WRITING PYTHON, BASH, OR NODE SCRIPTS TO PARSE JSON.
2. YOU MUST USE YOUR NATIVE FILE-READING TOOLS.
3. YOU MUST OUTPUT THE REPORT DIRECTLY USING NATIVE REASONING.
4. THIS POINTER DEFINES A NON-INTERACTIVE ORCHESTRATION FLOW.
</critical_directives>

## Trigger Contract

When the user invokes any recon trigger (`/recon`, `/recon detailed`, `/recon condensed`, `@rust-recon`, `@rust-recon detailed`, `@rust-recon condensed`) or asks for a recon report in natural language:

- Start orchestration immediately.
- Default format is `detailed` unless `condensed` is explicitly requested.
- Do NOT ask for confirmation to write the report.
- Do NOT stop after JSON extraction.

## CRITICAL RULE: ZERO DATA LOSS

- EVERY instruction gets a full Section 2 subsection (2a-2f).
- EVERY param, account, error, step, and mutation MUST appear in the report.
- NEVER write "omitted for brevity". NEVER skip instructions.
- "Not extracted - verify manually." is ONLY for genuinely empty JSON arrays.

## Mandatory Cognitive Forcing Step

Before any report synthesis, you MUST output this exact line:
"I am beginning a native reasoning pass over facts.json. I will not use temporary scripts. I will ensure zero data loss."

## One-Command Orchestration Sequence (Mandatory)

1. Read skill instructions from these files IN ORDER:
   - `.rust-recon/skill/core.md`
   - `.rust-recon/skill/references/facts-schema.md`
   - `.rust-recon/skill/references/section-specs.md`
   - `.rust-recon/skill/references/audit-patterns.md`
   - `.rust-recon/skill/references/cpi-rules.md`
   - `.rust-recon/skill/guardrails.md`

2. Run extraction tools:
   - `rust-recon scope`
   - `rust-recon facts`

3. Read extracted data in this exact order:
   - `.rust-recon/scope.json`
   - `.rust-recon/global_facts.json`
   - `.rust-recon/facts/index.json`
   - Every file listed in `.rust-recon/facts/index.json`, in ascending `order`
   - `.rust-recon/summary.json`
   - Fallback only if split facts are missing: `.rust-recon/facts.json`

4. Generate the report as the mandatory next step:
    - Write `recon.md` at project root.
    - If the user explicitly requests `RECON_REPORT.md`, write that filename instead.
   - Follow section-specs exactly.
   - Verify: instruction count in report == instruction count in extracted facts.

5. Fail-closed cleanup as the mandatory final step:
   - Run `rust-recon clean` immediately after successful report write.
   - If cleanup fails, attempt `rm -rf .rust-recon`.
   - If `.rust-recon` still exists after fallback, return a failure state (do not claim success).

## Completion Rules

- Recon execution is successful only if all three complete: extraction, report write, cleanup.
- Never end the run after extraction only.
- Never ask "Should I write the file?" once a recon trigger is received.
"#;

    // 1. Deploy CLAUDE.md into .rust-recon/
    fs::write(local_recon_dir.join("CLAUDE.md"), pointer_content)?;
    
    // 2. Deploy .cursorrules into .rust-recon/
    fs::write(local_recon_dir.join(".cursorrules"), pointer_content)?;
    
    // 3. Deploy Copilot instructions into .rust-recon/
    let github_dir = local_recon_dir.join(".github");
    fs::create_dir_all(&github_dir)?;
    fs::write(github_dir.join("copilot-instructions.md"), pointer_content)?;
    
    println!("✅ Skill pointers deployed (Sandboxed Mode).");
    println!("   Instructions copied to  → .rust-recon/skill/ (hidden from root)");
    println!("   CLAUDE.md               → .rust-recon/CLAUDE.md");
    println!("   .cursorrules            → .rust-recon/.cursorrules");
    println!("   .github/copilot...      → .rust-recon/.github/copilot-instructions.md");
    
    Ok(())
}

/// Cleans up the deployed skill configs and pointer files
pub fn cleanup_skill_configs(project_root: &Path) -> Result<()> {
    let local_recon_dir = project_root.join(".rust-recon");
    if local_recon_dir.exists() {
        // Delete skill directory
        let skill_dir = local_recon_dir.join("skill");
        if skill_dir.exists() {
            fs::remove_dir_all(skill_dir)?;
        }
        
        // Delete pointer files inside .rust-recon
        let _ = fs::remove_file(local_recon_dir.join("CLAUDE.md"));
        let _ = fs::remove_file(local_recon_dir.join(".cursorrules"));
        let _ = fs::remove_dir_all(local_recon_dir.join(".github"));
        
        // Also cleanup legacy root files if they exist from older versions
        let _ = fs::remove_file(project_root.join("CLAUDE.md"));
        let _ = fs::remove_file(project_root.join(".cursorrules"));
        // Only delete .github in root if it's strictly ours, but since users might have actions, let's just delete the file
        let _ = fs::remove_file(project_root.join(".github").join("copilot-instructions.md"));
        
        println!("✅ Workspace cleaned up successfully.");
    } else {
        println!("No .rust-recon directory found to clean up.");
    }
    
    Ok(())
}
