use crate::types::{Facts, Summary, PartialLamportFlow, SolFlow, SummarySolFlow, SummaryTokenFlow, SummaryAuthorityRevocation, SummaryStateMutation};
use serde_json::Value;
use std::collections::HashMap;

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
                            execution_steps: vec![],
                            sol_flows: vec![],
                            token_flows: vec![],
                            state_mutations: vec![],
                            set_authority_calls: vec![],
                            partial_lamport_flows: vec![],
                        });
                    }
                }
            }
        }
    }
}

pub fn generate_summary(facts: &Facts) -> Summary {
    let mut sum = Summary::default();
    
    // Helpers for summary aggregation
    let mut sol_flows_map: HashMap<String, SummarySolFlow> = HashMap::new();
    let mut token_flows_map: HashMap<String, SummaryTokenFlow> = HashMap::new();
    let mut auth_revoke_map: HashMap<String, SummaryAuthorityRevocation> = HashMap::new();
    let mut state_mut_map: HashMap<String, SummaryStateMutation> = HashMap::new();

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
        
        // Populate legacy token_flows
        for cpi in &ix.cpi_calls {
            let flow = format!("{} calls {}::{}", ix.name, cpi.target, cpi.instruction);
            if !sum.token_flows.contains(&flow) {
                sum.token_flows.push(flow);
            }
        }

        // Populate new declarative logic
        for sf in &ix.sol_flows {
            let key = format!("{}->{} via {}", sf.from, sf.to, sf.method);
            let entry = sol_flows_map.entry(key).or_insert_with(|| SummarySolFlow {
                from: sf.from.clone(),
                to: sf.to.clone(),
                via: sf.method.clone(),
                instructions: vec![],
            });
            if !entry.instructions.contains(&ix.name) {
                entry.instructions.push(ix.name.clone());
            }
        }

        for tf in &ix.token_flows {
            let key = format!("{}->{} via {}", tf.from, tf.to, tf.cpi_method);
            let entry = token_flows_map.entry(key).or_insert_with(|| SummaryTokenFlow {
                from: tf.from.clone(),
                to: tf.to.clone(),
                via: tf.cpi_method.clone(),
                instructions: vec![],
            });
            if !entry.instructions.contains(&ix.name) {
                entry.instructions.push(ix.name.clone());
            }
        }

        for auth in &ix.set_authority_calls {
            if auth.new_authority == "None" || auth.new_authority == "none" {
                let key = format!("{}-{}", auth.account, auth.authority_type);
                auth_revoke_map.entry(key).or_insert_with(|| SummaryAuthorityRevocation {
                    account: auth.account.clone(),
                    authority_type: auth.authority_type.clone(),
                    revoked_in: ix.name.clone(),
                });
            }
        }

        for sm in &ix.state_mutations {
            let entry = state_mut_map.entry(sm.account.clone()).or_insert_with(|| SummaryStateMutation {
                account: sm.account.clone(),
                fields_mutated: vec![],
                instructions: vec![],
            });
            if !entry.fields_mutated.contains(&sm.field) {
                entry.fields_mutated.push(sm.field.clone());
            }
            if !entry.instructions.contains(&ix.name) {
                entry.instructions.push(ix.name.clone());
            }
        }
    }

    sum.sol_flow_summary = sol_flows_map.into_values().collect();
    sum.token_flow_summary = token_flows_map.into_values().collect();
    sum.authority_revocations = auth_revoke_map.into_values().collect();
    sum.state_mutation_summary = state_mut_map.into_values().collect();
    
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

fn pair_lamport_flows(partial: Vec<PartialLamportFlow>) -> Vec<SolFlow> {
    let mut flows = vec![];
    let mut i = 0;
    let partial_len = partial.len();
    while i < partial_len {
        if i + 1 < partial_len {
            let a = &partial[i];
            let b = &partial[i + 1];
            
            // If adjacent and one is "out" and one is "in"
            if a.direction == "out" && b.direction == "in" {
                flows.push(SolFlow {
                    from: a.account.clone(),
                    to: b.account.clone(),
                    amount_expression: a.amount_expression.clone(), // using a's amount since they should match
                    method: "try_borrow_mut_lamports()".to_string(),
                    instruction_order: a.instruction_order,
                });
                i += 2;
                continue;
            }
        }
        i += 1;
    }
    flows
}

pub fn deduplicate_instructions(facts: &mut Facts) {
    for ix in &mut facts.instructions {
        if !ix.partial_lamport_flows.is_empty() {
            let mut paired = pair_lamport_flows(ix.partial_lamport_flows.clone());
            ix.sol_flows.append(&mut paired);
            ix.partial_lamport_flows.clear();
        }
    }

    let mut unique_ixs: Vec<crate::types::FactInstruction> = Vec::new();

    for mut ix in facts.instructions.drain(..) {
        // Alias detection: different name, but identical context struct
        if let Some(existing) = unique_ixs.iter_mut().find(|e| e.context == ix.context && !ix.context.is_empty() && ix.context != "Unknown") {
            
            // Handle naming for aliases
            if existing.name != ix.name && !existing.name.contains(&format!("aliases: {}", ix.name)) {
                if existing.name.contains("(aliases: ") {
                    let mut parts: Vec<&str> = existing.name.split("(aliases: ").collect();
                    if parts.len() == 2 {
                        let inner = parts[1].replace(")", "");
                        existing.name = format!("{} (aliases: {}, {})", parts[0].trim(), inner, ix.name);
                    }
                } else {
                    existing.name = format!("{} (aliases: {})", existing.name, ix.name);
                }
            }

            // Merge logic below
            let existing_has_enrichment = !existing.body_checks.is_empty() 
                || !existing.arithmetic.is_empty() 
                || !existing.cpi_calls.is_empty();
            let new_has_enrichment = !ix.body_checks.is_empty() 
                || !ix.arithmetic.is_empty() 
                || !ix.cpi_calls.is_empty();

            if new_has_enrichment && !existing_has_enrichment {
                let mut merged = ix;
                // keep the accounts from existing if they were parsed better
                if merged.accounts.is_empty() && !existing.accounts.is_empty() {
                    merged.accounts = existing.accounts.clone();
                }
                merged.name = existing.name.clone(); // preserve aliased name
                *existing = merged;
            } else if !new_has_enrichment && !existing_has_enrichment {
                // prefer the one with accounts
                if existing.accounts.is_empty() && !ix.accounts.is_empty() {
                    let name = existing.name.clone();
                    *existing = ix;
                    existing.name = name;
                }
            } else if existing_has_enrichment && new_has_enrichment {
                 // append missing elements
                 for bc in ix.body_checks {
                     if !existing.body_checks.contains(&bc) { existing.body_checks.push(bc); }
                 }
                 for ar in ix.arithmetic {
                     if !existing.arithmetic.contains(&ar) { existing.arithmetic.push(ar); }
                 }
                 for cpi in ix.cpi_calls {
                     if !existing.cpi_calls.contains(&cpi) { existing.cpi_calls.push(cpi); }
                 }
                 for ev in ix.events_emitted {
                     if !existing.events_emitted.contains(&ev) { existing.events_emitted.push(ev); }
                 }
                 for err in ix.error_codes_referenced {
                     if !existing.error_codes_referenced.contains(&err) { existing.error_codes_referenced.push(err); }
                 }
                 for step in ix.execution_steps {
                     if !existing.execution_steps.contains(&step) { existing.execution_steps.push(step); }
                 }
                 for flow in ix.sol_flows {
                     if !existing.sol_flows.contains(&flow) { existing.sol_flows.push(flow); }
                 }
                 for flow in ix.token_flows {
                     if !existing.token_flows.contains(&flow) { existing.token_flows.push(flow); }
                 }
                 for muta in ix.state_mutations {
                     if !existing.state_mutations.contains(&muta) { existing.state_mutations.push(muta); }
                 }
                 for auth in ix.set_authority_calls {
                     if !existing.set_authority_calls.contains(&auth) { existing.set_authority_calls.push(auth); }
                 }
                 for arg in ix.args {
                     if !existing.args.contains(&arg) { existing.args.push(arg); }
                 }
                 // Handle accounts mapping
                 if existing.accounts.is_empty() && !ix.accounts.is_empty() {
                     existing.accounts = ix.accounts;
                 }
            } else {
                if existing.accounts.is_empty() && !ix.accounts.is_empty() {
                    existing.accounts = ix.accounts;
                }
            }
        } else {
            unique_ixs.push(ix);
        }
    }
    facts.instructions = unique_ixs;
}
