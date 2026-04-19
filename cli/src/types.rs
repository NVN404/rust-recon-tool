use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeInstruction {
    pub ix: String,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub program: String,
    pub generated_at: String,
    pub instructions: Vec<ScopeInstruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Facts {
    pub program: String,
    pub instructions: Vec<FactInstruction>,
    pub accounts: Vec<FactAccount>,
    pub data_structs: Vec<FactDataStruct>,  // NEW: All #[account] data structs with field-level analysis
    pub errors: Vec<FactError>,
    pub cpi_calls: Vec<CpiCall>,
    pub risk_signals: Vec<RiskSignal>,
    pub flags: Vec<RiskFlag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionArg {
    pub name: String,
    pub type_info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionParam {
    pub name: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub overflow_risk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyCheck {
    pub macro_name: String,
    // Using Option because require_keys_eq has lhs/rhs instead of condition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lhs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rhs: Option<String>,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arithmetic {
    pub operation: String,
    pub style: String,
    pub expression: String,
    pub overflow_risk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactAccount {
    pub name: String,
    pub fields: Vec<InstructionArg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStructField {
    pub name: String,
    pub r#type: String,
    pub tags: Vec<String>,  // [NUMERIC], [AUTHORITY], [ACCOUNTING], [TIMESTAMP], [PUBKEY], [PAUSE FLAG], [STORED BUMP]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactDataStruct {
    pub name: String,
    pub fields: Vec<DataStructField>,
    pub attributes: Vec<String>,  // #[account] and other macros
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactError {
    pub code: String,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpiCall {
    pub target: String,
    pub instruction: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer_seeds: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nesting_depth: Option<String>,  // "top-level", "conditional", "loop", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_name: Option<String>,  // which instruction contains this CPI
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSignal {
    pub rule: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Summary {
    pub call_surface: Vec<String>,
    pub authority_map: Vec<String>,
    pub pda_map: Vec<String>,
    pub token_flows: Vec<String>,
    pub top_risks: Vec<String>,
}
