use crate::types::{Facts, Summary};
use serde_json::Value;

pub fn merge_idl_and_rust(idl: Option<Value>, facts: &mut Facts) {
    if let Some(i) = idl {
        if let Some(idl_ixs) = i.get("instructions").and_then(|v| v.as_array()) {
            for idl_ix in idl_ixs {
                if let Some(name) = idl_ix.get("name").and_then(|n| n.as_str()) {
                    if !facts.instructions.iter().any(|fx| fx.name == name) {
                        facts.instructions.push(crate::types::FactInstruction {
                            name: name.to_string(),
                            context: "Unknown".to_string(),
                            args: vec![],
                            accounts: vec![],
                            checks: vec![],
                            cpi_calls: vec![],
                            pda: vec![],
                            source: None, params: vec![], body_checks: vec![], arithmetic: vec![], events_emitted: vec![], uses_remaining_accounts: false, error_codes_referenced: vec![],
                        });
                    }
                }
            }
        }
    }
}

pub fn generate_summary(facts: &Facts) -> Summary {
    let mut sum = Summary::default();
    
    for ix in &facts.instructions {
        // Detailed Call Surface Construction
        let mut account_desc = Vec::new();
        for acc in &ix.accounts {
            let mut labels = Vec::new();
            if acc.is_mut { labels.push("Mut"); }
            if acc.is_signer { labels.push("Signer"); }
            
            let label_str = if labels.is_empty() { String::new() } else { format!(" [{}]", labels.join(", ")) };
            account_desc.push(format!("{}{}", acc.name, label_str));
            
            // Populate Authority Map
            if acc.is_signer && !sum.authority_map.contains(&acc.name) {
                sum.authority_map.push(format!("{} (in {})", acc.name, ix.name));
            }
            
            // Populate PDA Map
            for constraint in &acc.constraints {
                if constraint.contains("seeds") {
                    // Extract a condensed hint of the seed string
                    let clean_constraint = constraint.replace('\n', " ").replace("  ", " ");
                    if !sum.pda_map.contains(&clean_constraint) {
                        sum.pda_map.push(format!("PDA on {} ({}): {}", acc.name, ix.name, clean_constraint));
                    }
                }
            }
        }
        
        let accounts_str = if account_desc.is_empty() {
            "No accounts parsed".to_string()
        } else {
            account_desc.join(" | ")
        };
        
        sum.call_surface.push(format!("* {}: {}", ix.name, accounts_str));
        
        // Token & CPI Flows Map
        for cpi in &ix.cpi_calls {
            let flow = format!("{} calls {}::{}", ix.name, cpi.target, cpi.instruction);
            if !sum.token_flows.contains(&flow) {
                sum.token_flows.push(flow);
            }
        }
    }
    
    // Check missing signers
    if sum.authority_map.is_empty() {
         sum.top_risks.push("No explicit Signer accounts found in instruction validations!".to_string());
    }
    
    if facts.errors.is_empty() {
        sum.top_risks.push("No custom errors defined (Possible logic bugs missing revert conditions)".to_string());
    } else {
        sum.top_risks.push(format!("{} Custom Errors declared", facts.errors.len()));
    }
    
    sum
}
