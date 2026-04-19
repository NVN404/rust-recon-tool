pub fn aggregate_flags(facts: &mut crate::types::Facts) {
    let mut flags = Vec::new();
    for ix in &facts.instructions {
        for acc in &ix.accounts {
            if acc.unchecked {
                flags.push(crate::types::RiskFlag {
                    severity: "high".into(),
                    r#type: "unchecked_account".into(),
                    instruction: ix.name.clone(),
                    account: Some(acc.name.clone()),
                    expression: None,
                    drain_target: None,
                    note: "UncheckedAccount — zero Anchor validation.".into(),
                });
            }
            if acc.attributes.contains("init_if_needed") {
                flags.push(crate::types::RiskFlag {
                    severity: "medium".into(),
                    r#type: "init_if_needed".into(),
                    instruction: ix.name.clone(),
                    account: Some(acc.name.clone()),
                    expression: None,
                    drain_target: None,
                    note: "Re-initialization risk.".into(),
                });
            }
            if let Some(target) = &acc.close_target {
                flags.push(crate::types::RiskFlag {
                    severity: "info".into(),
                    r#type: "close_drain".into(),
                    instruction: format!("close_{}", acc.name), // per spec
                    account: Some(acc.name.clone()),
                    expression: None,
                    drain_target: Some(target.clone()),
                    note: "Verify drain target is not caller-controlled.".into(),
                });
            }
        }
        
        for arith in &ix.arithmetic {
            if arith.overflow_risk {
                flags.push(crate::types::RiskFlag {
                    severity: "medium".into(),
                    r#type: "unchecked_arithmetic".into(),
                    instruction: ix.name.clone(),
                    account: None,
                    expression: Some(arith.expression.clone()),
                    drain_target: None,
                    note: format!("Raw {} operation on numeric field.", arith.operation),
                });
            }
        }
        
        if ix.uses_remaining_accounts {
            flags.push(crate::types::RiskFlag {
                severity: "high".into(),
                r#type: "remaining_accounts".into(),
                instruction: ix.name.clone(),
                account: None,
                expression: None,
                drain_target: None,
                note: "Unvalidated account injection surface.".into(),
            });
        }
        
        let has_cpi_transfer = ix.cpi_calls.iter().any(|c| c.instruction == "transfer");
        if has_cpi_transfer && ix.body_checks.is_empty() {
             flags.push(crate::types::RiskFlag {
                severity: "medium".into(),
                r#type: "missing_access_control".into(),
                instruction: ix.name.clone(),
                account: None,
                expression: None,
                drain_target: None,
                note: "Instruction has CPI transfer but lacks require! checks.".into(),
            });
        }
    }
    facts.flags = flags;
}
