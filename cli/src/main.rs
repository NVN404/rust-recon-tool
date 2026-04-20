mod types;
mod project;
mod idl;
mod rust_parser;
mod merger;
mod output;
mod aggregate;

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
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let root = project::find_anchor_project()?;
    let programs_with_paths = project::detect_programs(&root)?;
    
    if programs_with_paths.is_empty() {
        println!("No programs found in the project.");
        return Ok(());
    }
    
    let default_prog = &programs_with_paths[0];
    
    match &cli.command {
        Commands::Scope { program } => {
            let (p, _path) = if let Some(name) = program {
                programs_with_paths.iter().find(|(n, _)| n == name).unwrap_or(default_prog)
            } else {
                default_prog
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
        Commands::Facts { program } => {
            let (p, path) = if let Some(name) = program {
                programs_with_paths.iter().find(|(n, _)| n == name).unwrap_or(default_prog)
            } else {
                default_prog
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
            println!("Facts and Summary generated for program: {}", p);
        }
    }
    
    Ok(())
}
