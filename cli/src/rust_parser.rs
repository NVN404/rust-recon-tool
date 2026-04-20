use anyhow::Result;
use std::path::Path;
use std::fs;
use crate::types::*;
use quote::ToTokens;

pub fn process_rust_code(project_root: &Path, program: &str) -> Result<Facts> {
    let mut facts = Facts::default();
    facts.program = program.to_string();

    let src_dir = project_root.join("src"); // Fixed path for workspace mode
    if !src_dir.exists() {
        return Ok(facts);
    }

    let mut files_to_parse = vec![src_dir.clone()];
    
    while let Some(path) = files_to_parse.pop() {
        if path.is_dir() {
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.filter_map(Result::ok) {
                    files_to_parse.push(entry.path());
                }
            }
        } else if path.extension().unwrap_or_default() == "rs" {
            let content = fs::read_to_string(&path)?;
            parse_file_content(&content, &path.to_string_lossy(), &mut facts)?;
        }
    }

    Ok(facts)
}

fn parse_file_content(content: &str, file_path: &str, facts: &mut Facts) -> Result<()> {
    if let Ok(syntax_tree) = syn::parse_file(content) {
        let mut visitor = AnchorVisitor { facts, current_file: file_path.to_string() };
        syn::visit::Visit::visit_file(&mut visitor, &syntax_tree);
    }
    Ok(())
}

/// Auto-tag struct fields based on name + type patterns
fn auto_tag_field(field_name: &str, field_type: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let name_lower = field_name.to_lowercase();
    let type_lower = field_type.to_lowercase();

    // [STORED BUMP]
    if name_lower == "bump" {
        tags.push("[STORED BUMP]".into());
    }

    // [AUTHORITY]
    if name_lower.contains("admin") || name_lower.contains("owner") || 
       name_lower.contains("authority") || name_lower.contains("signer") || 
       name_lower.contains("key") {
        if type_lower.contains("pubkey") {
            tags.push("[AUTHORITY]".into());
        }
    }

    // [NUMERIC ⚠ overflow]
    if (name_lower.contains("amount") || name_lower.contains("balance") || 
        name_lower.contains("total") || name_lower.contains("reserve") || 
        name_lower.contains("reward") || name_lower.contains("fee") || 
        name_lower.contains("shares") || name_lower.contains("stake") || 
        name_lower.contains("deposit")) &&
       (type_lower.contains("u64") || type_lower.contains("u128") || type_lower.contains("i64")) {
        tags.push("[NUMERIC ⚠ overflow]".into());
    }

    // [ACCOUNTING ⚠ reset]
    if (name_lower.contains("debt") || name_lower.contains("accrued") || 
        name_lower.contains("pending") || name_lower.contains("claimable")) &&
       (type_lower.contains("u64") || type_lower.contains("u128") || type_lower.contains("i64")) {
        tags.push("[ACCOUNTING ⚠ reset]".into());
    }

    // [TIMESTAMP ⚠ manipulation]
    if (name_lower.contains("timestamp") || name_lower.contains("slot") || 

    name_lower.contains("epoch") || name_lower.contains("time") || 
        name_lower.contains("duration") || name_lower.contains("created_at")) &&
       (type_lower.contains("i64") || type_lower.contains("u64")) {
        tags.push("[TIMESTAMP ⚠ manipulation]".into());
    }

    // [PAUSE FLAG]
    if (name_lower.contains("paused") || name_lower.contains("frozen") || 
        name_lower.contains("active") || name_lower.contains("enabled")) &&
       type_lower.contains("bool") {
        tags.push("[PAUSE FLAG]".into());
    }

    // [PUBKEY ⚠ validation]
    if name_lower.contains("authority") || name_lower.contains("owner") || 
       name_lower.contains("creator") || name_lower.contains("recipient") ||
       name_lower.contains("signer") {
        if type_lower.contains("pubkey") {
            tags.push("[PUBKEY ⚠ validation]".into());
        }
    }

    tags
}

// Support structs for visiting function bodies specifically
struct BodyVisitor {
    body_checks: Vec<BodyCheck>,
    arithmetic: Vec<Arithmetic>,
    events_emitted: Vec<String>,
    uses_remaining_accounts: bool,
    error_codes_referenced: Vec<String>,
}

impl<'ast> syn::visit::Visit<'ast> for BodyVisitor {
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let mac_path = m.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
        let tokens_str = m.tokens.to_token_stream().to_string();
        
        if mac_path == "require" || mac_path == "require_gt" || mac_path == "require_gte" || mac_path == "require_neq" {
            let parts: Vec<&str> = tokens_str.splitn(2, ',').collect();
            if parts.len() == 2 {
                let err = parts[1].trim().to_string();
                self.body_checks.push(BodyCheck {
                    macro_name: mac_path,
                    condition: Some(parts[0].trim().to_string()),
                    lhs: None,
                    rhs: None,
                    error: err.clone(),
                });
                if !self.error_codes_referenced.contains(&err) {
                    self.error_codes_referenced.push(err);
                }
            }
        } else if mac_path == "require_keys_eq" {
            let parts: Vec<&str> = tokens_str.splitn(3, ',').collect();
            if parts.len() == 3 {
                let err = parts[2].trim().to_string();
                self.body_checks.push(BodyCheck {
                    macro_name: mac_path,
                    condition: None,
                    lhs: Some(parts[0].trim().to_string()),
                    rhs: Some(parts[1].trim().to_string()),
                    error: err.clone(),
                });
                if !self.error_codes_referenced.contains(&err) {
                    self.error_codes_referenced.push(err);
                }
            }
        } else if mac_path == "emit" {
            let emitted = m.tokens.to_token_stream().into_iter().next().map(|tt| tt.to_string());
            if let Some(event) = emitted {
                if !self.events_emitted.contains(&event) {
                    self.events_emitted.push(event);
                }
            }
        }
        
        syn::visit::visit_macro(self, m);
    }
    
    fn visit_expr_field(&mut self, i: &'ast syn::ExprField) {
        let member_str = match &i.member {
            syn::Member::Named(ident) => ident.to_string(),
            _ => String::new(),
        };
        if member_str == "remaining_accounts" {
            self.uses_remaining_accounts = true;
        }
        syn::visit::visit_expr_field(self, i);
    }
    
    fn visit_expr_method_call(&mut self, i: &'ast syn::ExprMethodCall) {
        let method = i.method.to_string();
        if method == "remaining_accounts" {
            self.uses_remaining_accounts = true;
        }
        
        let safe_math = ["checked_add", "checked_sub", "checked_mul", "checked_div", "saturating_add", "saturating_sub", "saturating_mul"];
        let unsafe_math = ["wrapping_add", "wrapping_sub"];
        
        if safe_math.contains(&method.as_str()) || unsafe_math.contains(&method.as_str()) {
            let expr_str = i.to_token_stream().to_string();
            let is_overflow_risk = unsafe_math.contains(&method.as_str());
            self.arithmetic.push(Arithmetic {
                operation: method.replace("checked_", "").replace("saturating_", "").replace("wrapping_", ""),
                style: if is_overflow_risk { "unchecked".to_string() } else { "checked".to_string() },
                expression: expr_str,
                overflow_risk: is_overflow_risk,
            });
        }
        
        syn::visit::visit_expr_method_call(self, i);
    }
    
    fn visit_expr_binary(&mut self, i: &'ast syn::ExprBinary) {
        let op_str = match i.op {
            syn::BinOp::Add(_) => Some("add"),
            syn::BinOp::Sub(_) => Some("sub"),
            syn::BinOp::Mul(_) => Some("mul"),
            syn::BinOp::Div(_) => Some("div"),
            _ => None,
        };
        
        if let Some(op) = op_str {
            let expr_str = i.to_token_stream().to_string();
            // Flag raw math if variables imply amounts
            if expr_str.contains("amount") || expr_str.contains("balance") || expr_str.contains("total") ||
               expr_str.contains("shares") || expr_str.contains("reward") || expr_str.contains("fee") ||
               expr_str.contains("price") || expr_str.contains("reserve") {
                self.arithmetic.push(Arithmetic {
                    operation: op.to_string(),
                    style: "unchecked".to_string(),
                    expression: expr_str,
                    overflow_risk: true,
                });
            }
        }
        
        syn::visit::visit_expr_binary(self, i);
    }
}

struct AnchorVisitor<'a> {
    facts: &'a mut Facts,
    current_file: String,
}

impl<'ast, 'a> syn::visit::Visit<'ast> for AnchorVisitor<'a> {
    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        let name = i.sig.ident.to_string();
        
        let mut has_context = false;
        let mut context_name = String::new();
        let mut params = Vec::new();
        
        for input in &i.sig.inputs {
            if let syn::FnArg::Typed(pat_type) = input {
                let ty_str = pat_type.ty.to_token_stream().to_string();
                let param_name = pat_type.pat.to_token_stream().to_string();
                
                if let syn::Type::Path(type_path) = &*pat_type.ty {
                    if let Some(segment) = type_path.path.segments.last() {
                        if segment.ident == "Context" {
                            has_context = true;
                            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                                if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_path))) = args.args.first() {
                                    if let Some(inner_seg) = inner_path.path.segments.last() {
                                        context_name = inner_seg.ident.to_string();
                                    }
                                }
                            }
                            continue;
                        }
                    }
                }
                
                // Handle remaining non-Context params
                let overflow_risk = (ty_str.contains("u64") || ty_str.contains("u128")) &&
                    (param_name.contains("amount") || param_name.contains("shares") || 
                     param_name.contains("tokens") || param_name.contains("reward") || param_name.contains("fee"));
                
                params.push(InstructionParam {
                    name: param_name,
                    r#type: ty_str,
                    overflow_risk,
                });
            }
        }
        
        let mut checks = Vec::new();
        let mut cpi_calls = Vec::new();
        
        for stmt in &i.block.stmts {
            let stmt_str = stmt.to_token_stream().to_string();
            
            // Extract raw checks
            if stmt_str.contains("require ! ") || stmt_str.contains("require_eq ! ") || stmt_str.contains("require_keys_eq ! ") {
                checks.push(stmt_str.clone());
            }
            
            // Extract CPI flows (including token_2022)
            if stmt_str.contains("token :: transfer") { 
                cpi_calls.push(CpiCall { target: "token".into(), instruction: "transfer".into(), signer_seeds: None, nesting_depth: Some("top-level".into()), instruction_name: None }); 
            }
            if stmt_str.contains("token :: mint_to") { 
                cpi_calls.push(CpiCall { target: "token".into(), instruction: "mint_to".into(), signer_seeds: None, nesting_depth: Some("top-level".into()), instruction_name: None }); 
            }
            if stmt_str.contains("token :: burn") { 
                cpi_calls.push(CpiCall { target: "token".into(), instruction: "burn".into(), signer_seeds: None, nesting_depth: Some("top-level".into()), instruction_name: None }); 
            }
            if stmt_str.contains("token_2022 :: transfer_checked") { 
                cpi_calls.push(CpiCall { target: "token_2022".into(), instruction: "transfer_checked".into(), signer_seeds: None, nesting_depth: Some("top-level".into()), instruction_name: None }); 
            }
            if stmt_str.contains("token_2022 :: transfer") { 
                cpi_calls.push(CpiCall { target: "token_2022".into(), instruction: "transfer".into(), signer_seeds: None, nesting_depth: Some("top-level".into()), instruction_name: None }); 
            }
            if stmt_str.contains("token_2022 :: mint_to_checked") { 
                cpi_calls.push(CpiCall { target: "token_2022".into(), instruction: "mint_to_checked".into(), signer_seeds: None, nesting_depth: Some("top-level".into()), instruction_name: None }); 
            }
            if stmt_str.contains("system_program :: transfer") || stmt_str.contains("system_instruction :: transfer") { 
                cpi_calls.push(CpiCall { target: "system".into(), instruction: "transfer".into(), signer_seeds: None, nesting_depth: Some("top-level".into()), instruction_name: None }); 
            }
            if stmt_str.contains("invoke_signed") { 
                cpi_calls.push(CpiCall { target: "unknown".into(), instruction: "invoke_signed".into(), signer_seeds: None, nesting_depth: Some("top-level".into()), instruction_name: None }); 
            }
            else if stmt_str.contains("invoke") { 
                cpi_calls.push(CpiCall { target: "unknown".into(), instruction: "invoke".into(), signer_seeds: None, nesting_depth: Some("top-level".into()), instruction_name: None }); 
            }
        }
        
        let mut body_visitor = BodyVisitor {
            body_checks: vec![],
            arithmetic: vec![],
            events_emitted: vec![],
            uses_remaining_accounts: false,
            error_codes_referenced: vec![],
        };
        syn::visit::Visit::visit_block(&mut body_visitor, &i.block);
        
        if has_context {
            // Set instruction_name on all CPI calls
            for cpi in &mut cpi_calls {
                cpi.instruction_name = Some(name.clone());
            }
            
            self.facts.instructions.push(FactInstruction {
                name,
                context: context_name,
                args: vec![],
                params,
                accounts: vec![], // Linked later
                checks,
                body_checks: body_visitor.body_checks,
                arithmetic: body_visitor.arithmetic,
                cpi_calls,
                events_emitted: body_visitor.events_emitted,
                uses_remaining_accounts: body_visitor.uses_remaining_accounts,
                error_codes_referenced: body_visitor.error_codes_referenced,
                pda: vec![],
                source: Some(self.current_file.clone()),
            });
        }
        
        syn::visit::visit_item_fn(self, i);
    }
    
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        let name = i.ident.to_string();
        
        let is_accounts = i.attrs.iter().any(|attr| {
            let path = attr.path().segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
            path == "derive" && attr.to_token_stream().to_string().contains("Accounts")
        });
        
        if is_accounts {
            let mut fields = Vec::new();
            
            if let syn::Fields::Named(named_fields) = &i.fields {
                for field in &named_fields.named {
                    let field_name = field.ident.as_ref().map(|id| id.to_string()).unwrap_or_default();
                    let field_type = field.ty.to_token_stream().to_string();
                    
                    let mut wrapper_type = None;
                    let mut inner_type = None;
                    let mut is_mut = false;
                    let mut is_signer = false;
                    let mut close_target = None;
                    let mut has_one: Vec<String> = Vec::new();
                    
                    if let syn::Type::Path(tp) = &field.ty {
                        if let Some(segment) = tp.path.segments.last() {
                            let w_type = segment.ident.to_string();
                            wrapper_type = Some(w_type.clone());
                            if w_type == "Signer" { is_signer = true; }
                            
                            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                                if w_type == "Account" || w_type == "Program" || w_type == "Box" || w_type == "UncheckedAccount" {
                                    // inner constraint
                                    let mut type_args = args.args.iter();
                                    // Usually Account<'info, Inner> so we skip lifetime
                                    for arg in type_args {
                                        if let syn::GenericArgument::Type(syn::Type::Path(inner_path)) = arg {
                                            if let Some(inner_seg) = inner_path.path.segments.last() {
                                                inner_type = Some(inner_seg.ident.to_string());
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    let is_unchecked = match wrapper_type.as_deref() {
                        Some("UncheckedAccount") | Some("AccountInfo") => true,
                        _ => false,
                    };
                    
                    let mut constraints = Vec::new();
                    let mut attributes = String::new();
                    
                    for attr in &field.attrs {
                        if attr.path().segments.last().map(|s| s.ident.to_string()).unwrap_or_default() == "account" {
                            let tokens_str = attr.meta.to_token_stream().to_string();
                            attributes = tokens_str.clone();
                            constraints.push(tokens_str.clone());
                            if tokens_str.contains("mut") { is_mut = true; }
                            
                            // Extract close target
                            if tokens_str.contains("close = ") {
                                let parts: Vec<&str> = tokens_str.split("close = ").collect();
                                if parts.len() > 1 {
                                    let rest: Vec<&str> = parts[1].split(|c: char| !c.is_alphanumeric() && c != '_').collect();
                                    if !rest.is_empty() {
                                        close_target = Some(rest[0].to_string());
                                    }
                                }
                            }
                            
                            // Extract has_one = ... 
                            // Since a single #[account(...)] may have multiple has_one tags, we find them all
                            let mut search_str = tokens_str.as_str();
                            while let Some(idx) = search_str.find("has_one = ") {
                                let after_has_one = &search_str[idx + "has_one = ".len()..];
                                let parts: Vec<&str> = after_has_one.split(|c: char| !c.is_alphanumeric() && c != '_').collect();
                                if !parts.is_empty() {
                                    has_one.push(parts[0].to_string());
                                }
                                search_str = after_has_one;
                            }
                        }
                    }
                    
                    fields.push(InstructionAccount {
                        name: field_name.clone(),
                        type_info: field_type,
                        is_mut,
                        is_signer,
                        constraints,
                        attributes,
                        wrapper_type,
                        inner_type,
                        unchecked: is_unchecked,
                        has_one,
                        close_target,
                    });
                }
            }
            
            for ix in &mut self.facts.instructions {
                if ix.context == name {
                    ix.accounts = fields.clone();
                }
            }
            
            self.facts.accounts.push(FactAccount {
                name: name.clone(),
                fields: fields.into_iter().map(|f| InstructionArg { name: f.name, type_info: f.type_info }).collect(),
            });
        }
        
        // NEW: Extract data structs with #[account] macro (not just Context structs)
        let is_data_account = i.attrs.iter().any(|attr| {
            let path = attr.path().segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
            path == "account"
        });
        
        if is_data_account && !is_accounts {
            let mut data_fields = Vec::new();
            let mut attributes = Vec::new();
            
            // Collect all #[account(...)] attributes
            for attr in &i.attrs {
                if attr.path().segments.last().map(|s| s.ident.to_string()).unwrap_or_default() == "account" {
                    attributes.push(attr.to_token_stream().to_string());
                }
            }
            
            // Extract fields from the struct
            if let syn::Fields::Named(named_fields) = &i.fields {
                for field in &named_fields.named {
                    let field_name = field.ident.as_ref().map(|id| id.to_string()).unwrap_or_default();
                    let field_type = field.ty.to_token_stream().to_string();
                    
                    // Auto-tag based on field name and type
                    let tags = auto_tag_field(&field_name, &field_type);
                    
                    data_fields.push(DataStructField {
                        name: field_name,
                        r#type: field_type,
                        tags,
                    });
                }
            }
            
            // Store the data struct
            self.facts.data_structs.push(FactDataStruct {
                name: name.clone(),
                fields: data_fields,
                attributes,
            });
        }
        
        syn::visit::visit_item_struct(self, i);
    }
    
    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        let is_error = i.attrs.iter().any(|attr| {
            attr.path().segments.last().map(|s| s.ident.to_string()).unwrap_or_default() == "error_code"
        });
        
        if is_error {
            for variant in &i.variants {
                let msg = variant.attrs.iter()
                    .find(|a| a.path().segments.last().map(|s| s.ident.to_string()).unwrap_or_default() == "msg")
                    .map(|a| a.to_token_stream().to_string())
                    .unwrap_or_else(|| variant.ident.to_string());
                    
                self.facts.errors.push(FactError {
                    code: variant.ident.to_string(),
                    msg,
                });
            }
        }
        
        syn::visit::visit_item_enum(self, i);
    }
}
