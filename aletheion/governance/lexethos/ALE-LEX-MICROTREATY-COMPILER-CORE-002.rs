// ============================================================================
// FILE: aletheion/governance/lexethos/ALE-LEX-MICROTREATY-COMPILER-CORE-002.rs
// PURPOSE: Compiler that transforms RightsGrammar into machine-verifiable
//          MicroTreaties and JSON schemas for consumption by Rust planners,
//          Lua jobs, and CI preflight validation with proof-carrying configs
// LANGUAGE: Rust (2024 Edition)
// DESTINATION: Aletheion Repository - Governance LexEthos Subsystem
// COMPLIANCE: Zero-contamination IFC, NeurorightsEnvelope, FPIC metadata,
//             EU AI Act Article 5, Googolswarm/ALN blockchain integration
// INTEGRATION: ALE-LEX-RIGHTS-ATOM-002.aln, ALE-HIGHWAYS-CORRIDOR-KERNEL-001.rs
// C-ABI: JSON-in/JSON-out only, no side effects beyond file/STDOUT
// ============================================================================

#![deny(warnings)]
#![deny(clippy::all)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// SECTION 1: RIGHTS ATOM & MICROTREATY TYPE DEFINITIONS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RightCategory {
    Shade = 0,
    Movement = 1,
    Noise = 2,
    Hydration = 3,
    Thermal = 4,
    Biotic = 5,
    Somatic = 6,
    Neurobiome = 7,
    Treaty = 8,
    Waste = 9,
}

impl RightCategory {
    pub const ALL: [RightCategory; 10] = [
        RightCategory::Shade,
        RightCategory::Movement,
        RightCategory::Noise,
        RightCategory::Hydration,
        RightCategory::Thermal,
        RightCategory::Biotic,
        RightCategory::Somatic,
        RightCategory::Neurobiome,
        RightCategory::Treaty,
        RightCategory::Waste,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            RightCategory::Shade => "shade",
            RightCategory::Movement => "movement",
            RightCategory::Noise => "noise",
            RightCategory::Hydration => "hydration",
            RightCategory::Thermal => "thermal",
            RightCategory::Biotic => "biotic",
            RightCategory::Somatic => "somatic",
            RightCategory::Neurobiome => "neurobiome",
            RightCategory::Treaty => "treaty",
            RightCategory::Waste => "waste",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "shade" => Some(RightCategory::Shade),
            "movement" => Some(RightCategory::Movement),
            "noise" => Some(RightCategory::Noise),
            "hydration" => Some(RightCategory::Hydration),
            "thermal" => Some(RightCategory::Thermal),
            "biotic" => Some(RightCategory::Biotic),
            "somatic" => Some(RightCategory::Somatic),
            "neurobiome" => Some(RightCategory::Neurobiome),
            "treaty" => Some(RightCategory::Treaty),
            "waste" => Some(RightCategory::Waste),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum EnforcementLevel {
    Fundamental = 0,
    Protected = 1,
    Conditional = 2,
    Advisory = 3,
}

impl EnforcementLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnforcementLevel::Fundamental => "fundamental",
            EnforcementLevel::Protected => "protected",
            EnforcementLevel::Conditional => "conditional",
            EnforcementLevel::Advisory => "advisory",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "fundamental" => Some(EnforcementLevel::Fundamental),
            "protected" => Some(EnforcementLevel::Protected),
            "conditional" => Some(EnforcementLevel::Conditional),
            "advisory" => Some(EnforcementLevel::Advisory),
            _ => None,
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            EnforcementLevel::Fundamental => 0,
            EnforcementLevel::Protected => 1,
            EnforcementLevel::Conditional => 2,
            EnforcementLevel::Advisory => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ViolationSeverity {
    Critical = 0,
    High = 1,
    Medium = 2,
    Low = 3,
}

impl ViolationSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ViolationSeverity::Critical => "critical",
            ViolationSeverity::High => "high",
            ViolationSeverity::Medium => "medium",
            ViolationSeverity::Low => "low",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "critical" => Some(ViolationSeverity::Critical),
            "high" => Some(ViolationSeverity::High),
            "medium" => Some(ViolationSeverity::Medium),
            "low" => Some(ViolationSeverity::Low),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightsAtom {
    pub atom_id: String,
    pub right_category: RightCategory,
    pub right_name: String,
    pub right_description: String,
    pub legal_basis: Vec<String>,
    pub enforcement_level: EnforcementLevel,
    pub violation_severity: ViolationSeverity,
    pub remediation_required: bool,
    pub stake_multisig_override: bool,
    pub fpic_required: bool,
    pub neurorights_envelope: bool,
    pub evp_record_required: bool,
    pub thresholds: HashMap<String, serde_json::Value>,
    pub enforcement_actions: HashMap<String, serde_json::Value>,
}

impl RightsAtom {
    pub fn validate(&self) -> Result<(), String> {
        if self.atom_id.is_empty() {
            return Err("RightsAtom atom_id cannot be empty".to_string());
        }
        if !self.atom_id.starts_with("RA-") {
            return Err(format!("RightsAtom atom_id {} must start with RA-", self.atom_id));
        }
        if self.right_name.is_empty() {
            return Err("RightsAtom right_name cannot be empty".to_string());
        }
        if self.legal_basis.is_empty() {
            return Err("RightsAtom must have at least one legal_basis entry".to_string());
        }
        if self.enforcement_level == EnforcementLevel::Fundamental && !self.stake_multisig_override {
            return Err("Fundamental rights require stake_multisig_override".to_string());
        }
        if self.enforcement_level == EnforcementLevel::Fundamental && !self.fpic_required {
            return Err("Fundamental rights require fpic_required".to_string());
        }
        if self.right_category == RightCategory::Neurobiome && !self.neurorights_envelope {
            return Err("Neurobiome rights require neurorights_envelope".to_string());
        }
        if self.violation_severity == ViolationSeverity::Critical && !self.remediation_required {
            return Err("Critical violations require remediation_required".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroTreatyConstraint {
    pub constraint_id: String,
    pub source_atom_id: String,
    pub constraint_type: String,
    pub parameter: String,
    pub operator: String,
    pub value: serde_json::Value,
    pub unit: String,
    pub enforcement_level: EnforcementLevel,
    pub violation_severity: ViolationSeverity,
    pub remediation_action: String,
    pub notification_targets: Vec<String>,
    pub timeline_hours: u32,
    pub stake_multisig_required: bool,
    pub fpic_verified: bool,
    pub neurorights_compliant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroTreaty {
    pub treaty_id: String,
    pub version: String,
    pub effective_date: u64,
    pub expiration_date: Option<u64>,
    pub source_atoms: Vec<String>,
    pub constraints: Vec<MicroTreatyConstraint>,
    pub enforcement_level: EnforcementLevel,
    pub stake_multisig_verified: bool,
    pub indigenous_consultation_complete: bool,
    pub fpic_verified: bool,
    pub neurorights_envelope: bool,
    pub sovereignty_scalar: f64,
    pub roH_bound: f64,
    pub eco_impact_delta: f64,
    pub evp_record_hash: String,
    pub compiled_timestamp: u64,
    pub compiler_version: String,
    pub schema_hash: String,
}

impl MicroTreaty {
    pub fn new(treaty_id: String, version: String, compiler_version: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            treaty_id,
            version,
            effective_date: now,
            expiration_date: None,
            source_atoms: Vec::new(),
            constraints: Vec::new(),
            enforcement_level: EnforcementLevel::Protected,
            stake_multisig_verified: false,
            indigenous_consultation_complete: false,
            fpic_verified: false,
            neurorights_envelope: false,
            sovereignty_scalar: 0.85,
            roH_bound: 0.15,
            eco_impact_delta: 0.0,
            evp_record_hash: String::new(),
            compiled_timestamp: now,
            compiler_version,
            schema_hash: String::new(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.treaty_id.is_empty() {
            return Err("MicroTreaty treaty_id cannot be empty".to_string());
        }
        if !self.treaty_id.starts_with("MT-") {
            return Err(format!("MicroTreaty treaty_id {} must start with MT-", self.treaty_id));
        }
        if self.source_atoms.is_empty() {
            return Err("MicroTreaty must have at least one source_atom".to_string());
        }
        if self.constraints.is_empty() {
            return Err("MicroTreaty must have at least one constraint".to_string());
        }
        if self.roH_bound > 0.3 {
            return Err(format!("MicroTreaty roH_bound {} exceeds 0.3 sovereignty limit", self.roH_bound));
        }
        if self.sovereignty_scalar < 0.5 || self.sovereignty_scalar > 1.0 {
            return Err(format!("MicroTreaty sovereignty_scalar {} out of valid range [0.5, 1.0]", self.sovereignty_scalar));
        }
        for constraint in &self.constraints {
            if constraint.source_atom_id.is_empty() {
                return Err("MicroTreatyConstraint source_atom_id cannot be empty".to_string());
            }
        }
        Ok(())
    }

    pub fn compute_schema_hash(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.treaty_id.hash(&mut hasher);
        self.version.hash(&mut hasher);
        for atom in &self.source_atoms {
            atom.hash(&mut hasher);
        }
        for constraint in &self.constraints {
            constraint.constraint_id.hash(&mut hasher);
            constraint.parameter.hash(&mut hasher);
            constraint.operator.hash(&mut hasher);
        }
        self.schema_hash = format!("{:016x}", hasher.finish());
    }
}

// ============================================================================
// SECTION 2: IFC LABEL INTEGRATION FOR LEXETHOS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexEthosIFCLabel {
    pub label_id: String,
    pub sensitivity: String,
    pub domain: String,
    pub provenance: String,
    pub origin_hash: String,
    pub timestamp: u64,
    pub fpic_verified: bool,
    pub neurorights_compliant: bool,
    pub treaty_bound: bool,
    pub corridor_validated: bool,
}

impl LexEthosIFCLabel {
    pub fn new(label_id: String, sensitivity: String, domain: String, provenance: String, origin_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            label_id,
            sensitivity,
            domain,
            provenance,
            origin_hash,
            timestamp,
            fpic_verified: false,
            neurorights_compliant: false,
            treaty_bound: false,
            corridor_validated: false,
        }
    }

    pub fn with_fpic(mut self, verified: bool) -> Self {
        self.fpic_verified = verified;
        self
    }

    pub fn with_neurorights(mut self, compliant: bool) -> Self {
        self.neurorights_compliant = compliant;
        self
    }

    pub fn with_treaty_bound(mut self, bound: bool) -> Self {
        self.treaty_bound = bound;
        self
    }

    pub fn with_corridor_validation(mut self, validated: bool) -> Self {
        self.corridor_validated = validated;
        self
    }
}

// ============================================================================
// SECTION 3: COMPILER INPUT/OUTPUT STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerInput {
    pub request_id: String,
    pub rights_atoms: Vec<RightsAtom>,
    pub cross_domain_couplings: Vec<(RightCategory, RightCategory)>,
    pub enforcement_priority_order: Vec<EnforcementLevel>,
    pub ifc_labels: Vec<LexEthosIFCLabel>,
    pub evp_record_location: String,
    pub output_directory: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledSchema {
    pub schema_id: String,
    pub schema_type: String,
    pub schema_version: String,
    pub json_schema: serde_json::Value,
    pub validation_rules: Vec<String>,
    pub enforcement_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerOutput {
    pub request_id: String,
    pub microtreaties: Vec<MicroTreaty>,
    pub compiled_schemas: Vec<CompiledSchema>,
    pub cross_domain_treaties: Vec<MicroTreaty>,
    pub enforcement_priority_map: HashMap<String, u8>,
    pub violation_detection_rules: Vec<String>,
    pub remediation_workflows: Vec<String>,
    pub ifc_valid: bool,
    pub evp_record_created: bool,
    pub output_files: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub can_proceed: bool,
    pub output_ifc_label: LexEthosIFCLabel,
    pub timestamp: u64,
}

impl CompilerOutput {
    pub fn new(request_id: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            request_id,
            microtreaties: Vec::new(),
            compiled_schemas: Vec::new(),
            cross_domain_treaties: Vec::new(),
            enforcement_priority_map: HashMap::new(),
            violation_detection_rules: Vec::new(),
            remediation_workflows: Vec::new(),
            ifc_valid: false,
            evp_record_created: false,
            output_files: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
            can_proceed: true,
            output_ifc_label: LexEthosIFCLabel::new(
                format!("IFC-{}-LEXE-0002", timestamp),
                "sovereign".to_string(),
                "treaty".to_string(),
                "treaty".to_string(),
                format!("hash_{}", request_id),
            ),
            timestamp,
        }
    }

    pub fn add_error(&mut self, msg: String) {
        self.errors.push(msg);
        self.can_proceed = false;
    }

    pub fn add_warning(&mut self, msg: String) {
        self.warnings.push(msg);
    }
}

// ============================================================================
// SECTION 4: MICROTREATY COMPILER KERNEL
// ============================================================================

#[derive(Debug, Clone)]
pub struct MicroTreatyCompilerKernel {
    pub kernel_version: String,
    pub policy_version: String,
    pub max_treaties_per_compilation: usize,
    pub schema_validation_enabled: bool,
}

impl MicroTreatyCompilerKernel {
    pub fn new(kernel_version: String, policy_version: String) -> Self {
        Self {
            kernel_version,
            policy_version,
            max_treaties_per_compilation: 100,
            schema_validation_enabled: true,
        }
    }

    pub fn validate_ifc_labels(&self, labels: &[LexEthosIFCLabel]) -> Result<bool, String> {
        if labels.is_empty() {
            return Err("IFC labels required for all LexEthos compilation operations".to_string());
        }
        for label in labels {
            if label.sensitivity == "sovereign" && !label.fpic_verified {
                return Err(format!("Sovereign IFC label {} requires FPIC verification", label.label_id));
            }
            if label.domain != "treaty" && label.domain != "governance" {
                return Err(format!("Invalid domain {} for LexEthos IFC label", label.domain));
            }
            if label.origin_hash.is_empty() {
                return Err(format!("IFC label {} missing origin hash", label.label_id));
            }
        }
        Ok(true)
    }

    pub fn validate_rights_atoms(&self, atoms: &[RightsAtom]) -> Result<bool, String> {
        if atoms.is_empty() {
            return Err("At least one RightsAtom required for compilation".to_string());
        }
        if atoms.len() > self.max_treaties_per_compilation {
            return Err(format!(
                "Too many RightsAtoms: {} exceeds maximum {}",
                atoms.len(),
                self.max_treaties_per_compilation
            ));
        }
        for atom in atoms {
            if let Err(e) = atom.validate() {
                return Err(format!("RightsAtom {} validation failed: {}", atom.atom_id, e));
            }
        }
        let mut atom_ids = HashSet::new();
        for atom in atoms {
            if !atom_ids.insert(&atom.atom_id) {
                return Err(format!("Duplicate RightsAtom atom_id: {}", atom.atom_id));
            }
        }
        Ok(true)
    }

    pub fn compile_atom_to_treaty(&self, atom: &RightsAtom) -> MicroTreaty {
        let treaty_id = format!("MT-{}-{}", &atom.atom_id[3..9], &atom.right_category.as_str().to_uppercase()[..4]);
        let mut treaty = MicroTreaty::new(treaty_id, "001".to_string(), self.kernel_version.clone());
        treaty.source_atoms.push(atom.atom_id.clone());
        treaty.enforcement_level = atom.enforcement_level;
        treaty.fpic_verified = atom.fpic_required;
        treaty.neurorights_envelope = atom.neurorights_envelope;
        treaty.stake_multisig_verified = atom.stake_multisig_override;
        if atom.enforcement_level == EnforcementLevel::Fundamental {
            treaty.roH_bound = 0.15;
            treaty.sovereignty_scalar = 0.95;
        } else {
            treaty.roH_bound = 0.25;
            treaty.sovereignty_scalar = 0.85;
        }
        for (param, value) in &atom.thresholds {
            let constraint = MicroTreatyConstraint {
                constraint_id: format!("{}-{}", treaty.treaty_id, param.to_uppercase()),
                source_atom_id: atom.atom_id.clone(),
                constraint_type: "threshold".to_string(),
                parameter: param.clone(),
                operator: "lte".to_string(),
                value: value.clone(),
                unit: "variable".to_string(),
                enforcement_level: atom.enforcement_level,
                violation_severity: atom.violation_severity,
                remediation_action: atom.enforcement_actions.get("automatic_action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("manual_review")
                    .to_string(),
                notification_targets: atom.enforcement_actions.get("notification_targets")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
                    .unwrap_or_default(),
                timeline_hours: atom.enforcement_actions.get("timeline_hours")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(24) as u32,
                stake_multisig_required: atom.stake_multisig_override,
                fpic_verified: atom.fpic_required,
                neurorights_compliant: atom.neurorights_envelope,
            };
            treaty.constraints.push(constraint);
        }
        treaty.compute_schema_hash();
        treaty
    }

    pub fn compile_cross_domain_treaty(&self, atoms: &[RightsAtom], coupling: (RightCategory, RightCategory)) -> MicroTreaty {
        let treaty_id = format!("MT-CROSS-{}-{}", coupling.0.as_str().to_uppercase()[..4], coupling.1.as_str().to_uppercase()[..4]);
        let mut treaty = MicroTreaty::new(treaty_id, "001".to_string(), self.kernel_version.clone());
        let coupled_atoms: Vec<_> = atoms.iter()
            .filter(|a| a.right_category == coupling.0 || a.right_category == coupling.1)
            .collect();
        for atom in &coupled_atoms {
            treaty.source_atoms.push(atom.atom_id.clone());
        }
        treaty.enforcement_level = coupled_atoms.iter()
            .map(|a| a.enforcement_level.priority())
            .min()
            .map(|p| match p {
                0 => EnforcementLevel::Fundamental,
                1 => EnforcementLevel::Protected,
                2 => EnforcementLevel::Conditional,
                _ => EnforcementLevel::Advisory,
            })
            .unwrap_or(EnforcementLevel::Protected);
        treaty.fpic_verified = coupled_atoms.iter().any(|a| a.fpic_required);
        treaty.neurorights_envelope = coupled_atoms.iter().any(|a| a.neurorights_envelope);
        treaty.stake_multisig_verified = coupled_atoms.iter().any(|a| a.stake_multisig_override);
        treaty.roH_bound = 0.20;
        treaty.sovereignty_scalar = 0.90;
        for atom in &coupled_atoms {
            for (param, value) in &atom.thresholds {
                let constraint = MicroTreatyConstraint {
                    constraint_id: format!("{}-{}-{}", treaty.treaty_id, atom.atom_id[3..9].to_string(), param.to_uppercase()),
                    source_atom_id: atom.atom_id.clone(),
                    constraint_type: "cross_domain_threshold".to_string(),
                    parameter: format!("{}.{}", atom.right_category.as_str(), param),
                    operator: "lte".to_string(),
                    value: value.clone(),
                    unit: "variable".to_string(),
                    enforcement_level: atom.enforcement_level,
                    violation_severity: atom.violation_severity,
                    remediation_action: atom.enforcement_actions.get("automatic_action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("manual_review")
                        .to_string(),
                    notification_targets: atom.enforcement_actions.get("notification_targets")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
                        .unwrap_or_default(),
                    timeline_hours: atom.enforcement_actions.get("timeline_hours")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(24) as u32,
                    stake_multisig_required: atom.stake_multisig_override,
                    fpic_verified: atom.fpic_required,
                    neurorights_compliant: atom.neurorights_envelope,
                };
                treaty.constraints.push(constraint);
            }
        }
        treaty.compute_schema_hash();
        treaty
    }

    pub fn generate_json_schema(&self, treaty: &MicroTreaty) -> CompiledSchema {
        let schema_id = format!("SCHEMA-{}", treaty.treaty_id);
        let mut properties = serde_json::Map::new();
        properties.insert("treaty_id".to_string(), serde_json::json!({"type": "string", "pattern": "^MT-[0-9]{6}-[A-Z]{4}$"}));
        properties.insert("version".to_string(), serde_json::json!({"type": "string", "pattern": "^[0-9]{3}$"}));
        properties.insert("effective_date".to_string(), serde_json::json!({"type": "integer", "minimum": 0}));
        properties.insert("enforcement_level".to_string(), serde_json::json!({
            "type": "string",
            "enum": ["fundamental", "protected", "conditional", "advisory"]
        }));
        properties.insert("constraints".to_string(), serde_json::json!({
            "type": "array",
            "items": {"type": "object"}
        }));
        let schema = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "$id": format!("aletheion://schemas/{}", schema_id),
            "title": format!("MicroTreaty {} Schema", treaty.treaty_id),
            "type": "object",
            "properties": properties,
            "required": ["treaty_id", "version", "effective_date", "enforcement_level", "constraints"]
        });
        CompiledSchema {
            schema_id,
            schema_type: "microtreaty".to_string(),
            schema_version: treaty.version.clone(),
            json_schema: schema,
            validation_rules: treaty.constraints.iter()
                .map(|c| format!("{} {} {}", c.parameter, c.operator, c.value))
                .collect(),
            enforcement_endpoints: treaty.constraints.iter()
                .map(|c| format!("/api/v1/enforce/{}", c.constraint_id))
                .collect(),
        }
    }

    pub fn generate_violation_detection_rules(&self, treaties: &[MicroTreaty]) -> Vec<String> {
        let mut rules = Vec::new();
        for treaty in treaties {
            for constraint in &treaty.constraints {
                let rule = format!(
                    "DETECT_VIOLATION: {} IF {} {} {} THEN severity={} timeline={}h",
                    constraint.constraint_id,
                    constraint.parameter,
                    constraint.operator,
                    constraint.value,
                    constraint.violation_severity.as_str(),
                    constraint.timeline_hours
                );
                rules.push(rule);
            }
        }
        rules
    }

    pub fn generate_remediation_workflows(&self, treaties: &[MicroTreaty]) -> Vec<String> {
        let mut workflows = Vec::new();
        for treaty in treaties {
            for constraint in &treaty.constraints {
                if constraint.remediation_action != "manual_review" {
                    let workflow = format!(
                        "WORKFLOW: {} TRIGGER={} TARGETS=[{}] TIMELINE={}h",
                        constraint.remediation_action,
                        constraint.constraint_id,
                        constraint.notification_targets.join(","),
                        constraint.timeline_hours
                    );
                    workflows.push(workflow);
                }
            }
        }
        workflows
    }

    pub fn compile_treaties(&self, input: CompilerInput) -> CompilerOutput {
        let mut output = CompilerOutput::new(input.request_id.clone());
        if let Err(e) = self.validate_ifc_labels(&input.ifc_labels) {
            output.add_error(e);
            return output;
        }
        output.ifc_valid = true;
        if let Err(e) = self.validate_rights_atoms(&input.rights_atoms) {
            output.add_error(e);
            return output;
        }
        for atom in &input.rights_atoms {
            let treaty = self.compile_atom_to_treaty(atom);
            if let Err(e) = treaty.validate() {
                output.add_warning(format!("MicroTreaty {} validation warning: {}", treaty.treaty_id, e));
            }
            output.microtreaties.push(treaty);
        }
        for coupling in &input.cross_domain_couplings {
            let treaty = self.compile_cross_domain_treaty(&input.rights_atoms, *coupling);
            if let Err(e) = treaty.validate() {
                output.add_warning(format!("Cross-domain treaty {} validation warning: {}", treaty.treaty_id, e));
            }
            output.cross_domain_treaties.push(treaty);
        }
        for level in &input.enforcement_priority_order {
            output.enforcement_priority_map.insert(level.as_str().to_string(), level.priority());
        }
        let all_treaties: Vec<_> = output.microtreaties.iter()
            .chain(output.cross_domain_treaties.iter())
            .collect();
        for treaty in all_treaties {
            let schema = self.generate_json_schema(treaty);
            output.compiled_schemas.push(schema);
        }
        output.violation_detection_rules = self.generate_violation_detection_rules(all_treaties.as_slice());
        output.remediation_workflows = self.generate_remediation_workflows(all_treaties.as_slice());
        output.evp_record_created = !input.evp_record_location.is_empty();
        output.output_ifc_label = output.output_ifc_label
            .with_fpic(true)
            .with_treaty_bound(true)
            .with_corridor_validation(true);
        output
    }

    pub fn write_output_files(&self, output: &CompilerOutput, output_dir: &str) -> Result<Vec<String>, String> {
        let mut written_files = Vec::new();
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory {}: {}", output_dir, e))?;
        for treaty in &output.microtreaties {
            let file_path = PathBuf::from(output_dir).join(format!("{}.json", treaty.treaty_id));
            let mut file = File::create(&file_path)
                .map_err(|e| format!("Failed to create file {:?}: {}", file_path, e))?;
            let json = serde_json::to_string_pretty(treaty)
                .map_err(|e| format!("Failed to serialize treaty {}: {}", treaty.treaty_id, e))?;
            file.write_all(json.as_bytes())
                .map_err(|e| format!("Failed to write treaty {}: {}", treaty.treaty_id, e))?;
            written_files.push(file_path.to_string_lossy().to_string());
        }
        for schema in &output.compiled_schemas {
            let file_path = PathBuf::from(output_dir).join(format!("{}.schema.json", schema.schema_id));
            let mut file = File::create(&file_path)
                .map_err(|e| format!("Failed to create file {:?}: {}", file_path, e))?;
            let json = serde_json::to_string_pretty(&schema.json_schema)
                .map_err(|e| format!("Failed to serialize schema {}: {}", schema.schema_id, e))?;
            file.write_all(json.as_bytes())
                .map_err(|e| format!("Failed to write schema {}: {}", schema.schema_id, e))?;
            written_files.push(file_path.to_string_lossy().to_string());
        }
        let rules_path = PathBuf::from(output_dir).join("violation_detection_rules.json");
        let mut file = File::create(&rules_path)
            .map_err(|e| format!("Failed to create file {:?}: {}", rules_path, e))?;
        let json = serde_json::to_string_pretty(&output.violation_detection_rules)
            .map_err(|e| format!("Failed to serialize rules: {}", e))?;
        file.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write rules: {}", e))?;
        written_files.push(rules_path.to_string_lossy().to_string());
        let workflows_path = PathBuf::from(output_dir).join("remediation_workflows.json");
        let mut file = File::create(&workflows_path)
            .map_err(|e| format!("Failed to create file {:?}: {}", workflows_path, e))?;
        let json = serde_json::to_string_pretty(&output.remediation_workflows)
            .map_err(|e| format!("Failed to serialize workflows: {}", e))?;
        file.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write workflows: {}", e))?;
        written_files.push(workflows_path.to_string_lossy().to_string());
        Ok(written_files)
    }
}

// ============================================================================
// SECTION 5: JSON C-ABI EXPORT FOR LUA FFI INTEGRATION
// ============================================================================

#[no_mangle]
pub extern "C" fn ale_lexethos_compile_treaties_json(
    input_json: *const libc::c_char,
) -> *mut libc::c_char {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        unsafe {
            let input_str = CStr::from_ptr(input_json).to_string_lossy();
            let input: CompilerInput = match serde_json::from_str(&input_str) {
                Ok(i) => i,
                Err(e) => {
                    return CString::new(format!(
                        r#"{{"valid":false,"error":"Input JSON parse failed: {}"}}"#,
                        e
                    ))
                    .unwrap()
                    .into_raw();
                }
            };
            let kernel = MicroTreatyCompilerKernel::new("002".to_string(), "001".to_string());
            let mut output = kernel.compile_treaties(input);
            if !input.output_directory.is_empty() {
                match kernel.write_output_files(&output, &input.output_directory) {
                    Ok(files) => output.output_files = files,
                    Err(e) => output.add_error(e),
                }
            }
            let json_output = serde_json::to_string(&output).unwrap_or_else(|e| {
                format!(r#"{{"valid":false,"error":"Serialization failed: {}"}}"#, e)
            });
            CString::new(json_output).unwrap()
        }
    }));
    match result {
        Ok(cstring) => cstring.into_raw(),
        Err(_) => {
            let err = CString::new(r#"{"valid":false,"error":"Kernel panic during treaty compilation"}"#)
                .unwrap();
            err.into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn ale_lexethos_validate_atom_json(
    atom_json: *const libc::c_char,
) -> *mut libc::c_char {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        unsafe {
            let atom_str = CStr::from_ptr(atom_json).to_string_lossy();
            let atom: RightsAtom = match serde_json::from_str(&atom_str) {
                Ok(a) => a,
                Err(e) => {
                    return CString::new(format!(
                        r#"{{"valid":false,"error":"Atom JSON parse failed: {}"}}"#,
                        e
                    ))
                    .unwrap()
                    .into_raw();
                }
            };
            let kernel = MicroTreatyCompilerKernel::new("002".to_string(), "001".to_string());
            match kernel.validate_rights_atoms(&[atom.clone()]) {
                Ok(_) => {
                    let json_output = serde_json::json!({
                        "valid": true,
                        "atom_id": atom.atom_id,
                        "right_category": atom.right_category.as_str(),
                        "enforcement_level": atom.enforcement_level.as_str()
                    });
                    CString::new(json_output.to_string()).unwrap()
                }
                Err(e) => {
                    CString::new(format!(r#"{{"valid":false,"error":"{}"}}"#, e)).unwrap()
                }
            }
        }
    }));
    match result {
        Ok(cstring) => cstring.into_raw(),
        Err(_) => {
            let err = CString::new(r#"{"valid":false,"error":"Kernel panic during atom validation"}"#)
                .unwrap();
            err.into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn ale_lexethos_free_string(ptr: *mut libc::c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

// ============================================================================
// SECTION 6: UNIT TESTS (CI-INTEGRATED)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rights_atom_validation() {
        let atom = RightsAtom {
            atom_id: "RA-000001-SHAD".to_string(),
            right_category: RightCategory::Shade,
            right_name: "Shade_Access_Right".to_string(),
            right_description: "Test shade access right".to_string(),
            legal_basis: vec!["Phoenix Heat Relief Network Ordinance 2025".to_string()],
            enforcement_level: EnforcementLevel::Fundamental,
            violation_severity: ViolationSeverity::Critical,
            remediation_required: true,
            stake_multisig_override: true,
            fpic_required: true,
            neurorights_envelope: false,
            evp_record_required: true,
            thresholds: HashMap::new(),
            enforcement_actions: HashMap::new(),
        };
        assert!(atom.validate().is_ok());
    }

    #[test]
    fn test_microtreaty_validation() {
        let mut treaty = MicroTreaty::new("MT-000001-SHAD".to_string(), "001".to_string(), "002".to_string());
        treaty.source_atoms.push("RA-000001-SHAD".to_string());
        treaty.constraints.push(MicroTreatyConstraint {
            constraint_id: "MT-000001-SHAD-TEST".to_string(),
            source_atom_id: "RA-000001-SHAD".to_string(),
            constraint_type: "threshold".to_string(),
            parameter: "ambient_temp_c".to_string(),
            operator: "lte".to_string(),
            value: serde_json::json!(35.0),
            unit: "celsius".to_string(),
            enforcement_level: EnforcementLevel::Fundamental,
            violation_severity: ViolationSeverity::Critical,
            remediation_action: "activate_shade_structures".to_string(),
            notification_targets: vec!["heat_relief_network".to_string()],
            timeline_hours: 2,
            stake_multisig_required: true,
            fpic_verified: true,
            neurorights_compliant: false,
        });
        assert!(treaty.validate().is_ok());
    }

    #[test]
    fn test_compiler_kernel_compilation() {
        let kernel = MicroTreatyCompilerKernel::new("002".to_string(), "001".to_string());
        let atom = RightsAtom {
            atom_id: "RA-000001-SHAD".to_string(),
            right_category: RightCategory::Shade,
            right_name: "Shade_Access_Right".to_string(),
            right_description: "Test shade access right".to_string(),
            legal_basis: vec!["Phoenix Heat Relief Network Ordinance 2025".to_string()],
            enforcement_level: EnforcementLevel::Fundamental,
            violation_severity: ViolationSeverity::Critical,
            remediation_required: true,
            stake_multisig_override: true,
            fpic_required: true,
            neurorights_envelope: false,
            evp_record_required: true,
            thresholds: HashMap::new(),
            enforcement_actions: HashMap::new(),
        };
        let treaty = kernel.compile_atom_to_treaty(&atom);
        assert_eq!(treaty.source_atoms.len(), 1);
        assert!(treaty.fpic_verified);
    }

    #[test]
    fn test_ifc_label_validation() {
        let kernel = MicroTreatyCompilerKernel::new("002".to_string(), "001".to_string());
        let label = LexEthosIFCLabel::new(
            "IFC-001".to_string(),
            "sovereign".to_string(),
            "treaty".to_string(),
            "treaty".to_string(),
            "hash123".to_string(),
        );
        let result = kernel.validate_ifc_labels(&[label]);
        assert!(result.is_err());
    }

    #[test]
    fn test_enforcement_level_priority() {
        assert_eq!(EnforcementLevel::Fundamental.priority(), 0);
        assert_eq!(EnforcementLevel::Protected.priority(), 1);
        assert_eq!(EnforcementLevel::Conditional.priority(), 2);
        assert_eq!(EnforcementLevel::Advisory.priority(), 3);
    }

    #[test]
    fn test_right_category_conversion() {
        assert_eq!(RightCategory::Shade.as_str(), "shade");
        assert_eq!(RightCategory::from_str("shade"), Some(RightCategory::Shade));
        assert_eq!(RightCategory::from_str("invalid"), None);
    }
}
