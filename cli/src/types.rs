use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScopeInstruction {
    pub ix: String,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scope {
    pub program: String,
    pub generated_at: String,
    pub instructions: Vec<ScopeInstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Facts {
    pub program: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub program_id: Option<String>,
    pub instructions: Vec<FactInstruction>,
    pub accounts: Vec<FactAccount>,
    pub data_structs: Vec<FactDataStruct>,
    pub errors: Vec<FactError>,
    pub cpi_calls: Vec<CpiCall>,
    pub risk_signals: Vec<RiskSignal>,
    pub flags: Vec<RiskFlag>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FactInstruction {
    pub name: String,
    pub context: String,
    pub args: Vec<InstructionArg>,
    pub params: Vec<InstructionParam>,
    pub accounts: Vec<InstructionAccount>,
    pub checks: Vec<String>,
    pub body_checks: Vec<BodyCheck>,
    pub arithmetic: Vec<Arithmetic>,
    pub cpi_calls: Vec<CpiCall>,
    pub events_emitted: Vec<String>,
    pub uses_remaining_accounts: bool,
    pub error_codes_referenced: Vec<String>,
    pub pda: Vec<String>,
    pub source: Option<String>,
    
    // NEW FIELDS for Behavioral Facts
    #[serde(default)]
    pub execution_steps: Vec<ExecutionStep>,
    #[serde(default)]
    pub sol_flows: Vec<SolFlow>,
    #[serde(default)]
    pub token_flows: Vec<TokenFlow>,
    #[serde(default)]
    pub state_mutations: Vec<StateMutation>,
    #[serde(default)]
    pub set_authority_calls: Vec<SetAuthorityCall>,
    
    #[serde(skip)]
    #[serde(default)]
    pub partial_lamport_flows: Vec<PartialLamportFlow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstructionArg {
    pub name: String,
    pub type_info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstructionParam {
    pub name: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub overflow_risk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstructionAccount {
    pub name: String,
    pub type_info: String,
    pub is_mut: bool,
    pub is_signer: bool,
    pub constraints: Vec<String>,
    pub attributes: String,
    pub wrapper_type: Option<String>,
    pub inner_type: Option<String>,
    pub unchecked: bool,
    pub has_one: Vec<String>,
    pub close_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BodyCheck {
    pub macro_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lhs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rhs: Option<String>,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Arithmetic {
    pub operation: String,
    pub style: String,
    pub expression: String,
    pub overflow_risk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskFlag {
    pub severity: String,
    pub r#type: String,
    pub instruction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drain_target: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FactAccount {
    pub name: String,
    pub fields: Vec<InstructionArg>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataStructField {
    pub name: String,
    pub r#type: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FactDataStruct {
    pub name: String,
    pub fields: Vec<DataStructField>,
    pub attributes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FactError {
    pub code: String,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CpiCall {
    pub target: String,
    pub instruction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer_seeds: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nesting_depth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskSignal {
    pub rule: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Summary {
    pub call_surface: Vec<String>,
    pub authority_map: Vec<String>,
    pub pda_map: Vec<String>,
    pub token_flows: Vec<String>, // Keep for back-compat
    pub top_risks: Vec<String>,
    
    // NEW SUMMARY FIELDS
    #[serde(default)]
    pub sol_flow_summary: Vec<SummarySolFlow>,
    #[serde(default)]
    pub token_flow_summary: Vec<SummaryTokenFlow>,
    #[serde(default)]
    pub authority_revocations: Vec<SummaryAuthorityRevocation>,
    #[serde(default)]
    pub state_mutation_summary: Vec<SummaryStateMutation>,
}

// NEW STRUCTS FOR EXECUTION FLOW EXTRACTION

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionStep {
    pub order: usize,
    pub kind: StepKind,
    pub expression: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    FieldRead,
    FieldAssignment,
    Arithmetic,
    CpiCall,
    LamportTransfer,
    SetAuthority,
    RequireCheck,
    Emit,
    ConditionalBranch,
    LetBinding,
    MethodCall,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartialLamportFlow {
    pub account: String,
    pub direction: String,
    pub amount_expression: String,
    pub instruction_order: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SolFlow {
    pub from: String,
    pub to: String,
    pub amount_expression: String,
    pub method: String,
    pub instruction_order: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenFlow {
    pub from: String,
    pub to: String,
    pub amount_expression: String,
    pub cpi_method: String,
    pub instruction_order: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateMutation {
    pub account: String,
    pub field: String,
    pub operation: String,
    pub value_expression: String,
    pub instruction_order: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetAuthorityCall {
    pub account: String,
    pub authority_type: String,
    pub new_authority: String,
    pub instruction_order: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SummarySolFlow {
    pub from: String,
    pub to: String,
    pub via: String,
    pub instructions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SummaryTokenFlow {
    pub from: String,
    pub to: String,
    pub via: String,
    pub instructions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SummaryAuthorityRevocation {
    pub account: String,
    pub authority_type: String,
    pub revoked_in: String, // Instruction name
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SummaryStateMutation {
    pub account: String,
    pub fields_mutated: Vec<String>,
    pub instructions: Vec<String>,
}
