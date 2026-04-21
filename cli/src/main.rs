mod types;
mod project;
mod idl;
mod rust_parser;
mod merger;
mod output;
mod aggregate;
mod flow_extractor;
mod skill_deploy;

use clap::{Parser, Subcommand};
use types::{Scope, ScopeInstruction, Facts, Summary};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rust-recon")]
#[command(about = "Recon tool for Anchor programs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Scope {
        #[arg(short, long)]
        program: Option<String>,
    },
    Facts {
        #[arg(short, long)]
        program: Option<String>,
        /// Skip deploying skill configs after extraction
        #[arg(long, default_value_t = false)]
        no_setup: bool,
    },
    /// Deploy AI agent skill configs into the current project.
    /// This copies instructions into .rust-recon/skill/ so sandboxed AI agents can read them.
    Setup,
    /// Clean up deployed skill configs and pointer files.
    Clean,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let root = project::find_anchor_project()?;
    let programs_with_paths = project::detect_programs(&root)?;
    
    if programs_with_paths.is_empty() && !matches!(cli.command, Commands::Setup) {
        println!("No programs found in the project.");
        return Ok(());
    }
    
    let default_prog = if !programs_with_paths.is_empty() {
        Some(&programs_with_paths[0])
    } else {
        None
    };
    
    match &cli.command {
        Commands::Scope { program } => {
            let default = default_prog.unwrap();
            let (p, _path) = if let Some(name) = program {
                programs_with_paths.iter().find(|(n, _)| n == name).unwrap_or(default)
            } else {
                default
            };
            
            let idl_data = idl::read_idl_if_exists(&root, p).unwrap_or(None);
            
            let mut instructions = Vec::new();
            if let Some(idl_val) = idl_data.as_ref() {
                instructions = idl::extract_instructions_from_idl(idl_val);
            }
            
            let scope = Scope {
                program: p.clone(),
                generated_at: chrono::Utc::now().to_rfc3339(),
                instructions,
            };
            
            let facts = Facts::default();
            let sum = Summary::default();
            
            output::write_outputs(&root, &scope, &facts, &sum)?;
            println!("Scope generated for program: {}", p);
        }
        Commands::Facts { program, no_setup } => {
            let default = default_prog.unwrap();
            let (p, path) = if let Some(name) = program {
                programs_with_paths.iter().find(|(n, _)| n == name).unwrap_or(default)
            } else {
                default
            };
            
            let idl_data = idl::read_idl_if_exists(&root, p).unwrap_or(None);
            let mut facts = rust_parser::process_rust_code(path, p)?;
            merger::merge_idl_and_rust(idl_data, &mut facts);
            merger::deduplicate_instructions(&mut facts);
            aggregate::aggregate_flags(&mut facts);
            let summary = merger::generate_summary(&facts);
            
            let scope_instructions = facts.instructions.iter().map(|ix| ScopeInstruction {
                ix: ix.name.clone(),
                context: ix.context.clone(),
            }).collect();
            
            let scope = Scope {
                 program: p.clone(),
                 generated_at: chrono::Utc::now().to_rfc3339(),
                 instructions: scope_instructions,
            };
            
            output::write_outputs(&root, &scope, &facts, &summary)?;
            
            // Extraction diagnostics
            println!("\n=== rust-recon Extraction Summary ===");
            if let Some(ref pid) = facts.program_id {
                println!("  ✓ Program ID: {}", pid);
            } else {
                println!("  ⚠ Program ID: not found (no declare_id! macro)");
            }
            println!("  ✓ {} instructions extracted", facts.instructions.len());
            
            let ctx_count = facts.instructions.iter().filter(|ix| !ix.context.is_empty() && ix.context != "Unknown").count();
            println!("  {} {} context structs linked", if ctx_count > 0 { "✓" } else { "⚠" }, ctx_count);
            
            let acct_count: usize = facts.instructions.iter().map(|ix| ix.accounts.len()).sum();
            println!("  {} {} accounts total across instructions", if acct_count > 0 { "✓" } else { "⚠" }, acct_count);
            
            println!("  ✓ {} data structs catalogued", facts.data_structs.len());
            println!("  ✓ {} error codes indexed", facts.errors.len());
            
            let cpi_count: usize = facts.instructions.iter().map(|ix| ix.cpi_calls.len()).sum();
            println!("  ✓ {} CPI calls traced", cpi_count);
            
            let step_count: usize = facts.instructions.iter().map(|ix| ix.execution_steps.len()).sum();
            println!("  ✓ {} execution steps recorded", step_count);
            
            let mut_count: usize = facts.instructions.iter().map(|ix| ix.state_mutations.len()).sum();
            println!("  ✓ {} state mutations recorded", mut_count);
            
            println!("  ✓ {} parser flags generated", facts.flags.len());
            
            // Warn about instructions with 0 accounts (context struct likely not linked)
            let no_acct_ixs: Vec<&str> = facts.instructions.iter()
                .filter(|ix| ix.accounts.is_empty())
                .map(|ix| ix.name.as_str())
                .collect();
            if !no_acct_ixs.is_empty() {
                println!("\n  ⚠ Instructions with 0 accounts (context struct not linked):");
                for name in &no_acct_ixs {
                    println!("    - {}", name);
                }
            }
            
            if facts.instructions.is_empty() {
                println!("\n  ❌ ZERO instructions extracted! Check that:");
                println!("     - Anchor.toml workspace members point to correct program directories");
                println!("     - Program functions use Context<T> parameter signature");
                println!("     - Source path: {}", path.display());
            }
            
            println!("===================================\n");
            println!("Facts and Summary generated for program: {}", p);
            
            // Auto-deploy skill configs unless --no-setup is passed
            if !no_setup {
                println!("\n--- Auto-deploying skill configs ---");
                match skill_deploy::deploy_skill_configs(&root) {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Warning: Could not deploy skill configs: {}", e);
                        eprintln!("Run 'rust-recon setup' manually after installing the skill repo.");
                    }
                }
            }
        }
        Commands::Setup => {
            println!("Deploying rust-recon skill configs...");
            skill_deploy::deploy_skill_configs(&root)?;
        }
        Commands::Clean => {
            println!("Cleaning up rust-recon skill configs...");
            skill_deploy::cleanup_skill_configs(&root)?;
        }
    }
    
    Ok(())
}
