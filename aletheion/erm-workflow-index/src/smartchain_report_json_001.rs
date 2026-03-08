use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::{
    SmartChain, SmartChainConfig, SmartChainRegistry, ValidationError, PQMode,
};

// -----------------------------------------------------------------------------
// Core report structs
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartChainReportMeta {
    pub timestamp: String,
    pub repo: String,
    pub config_path: String,
    pub git_commit: Option<String>,
    pub git_branch: Option<String>,
    pub git_pr_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartChainErrorLocation {
    pub file: String,
    pub json_pointer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmartChainErrorContext {
    pub domain: Option<String>,
    pub found_pq_mode: Option<String>,
    pub required_pq_mode: Option<String>,
    pub multisig_threshold: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartChainErrorEntry {
    pub chain_id: Option<String>,
    pub error_code: String,
    pub severity: String,
    pub message: String,
    pub location: Option<SmartChainErrorLocation>,
    pub context: Option<SmartChainErrorContext>,
    pub suggested_remediation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartChainChainStatus {
    pub id: String,
    pub name: String,
    pub status: String,
    pub domains: Vec<String>,
    pub city_region: Vec<String>,
    pub pq_mode: String,
    pub rollbackforbidden: bool,
    pub multisig_threshold: i32,
    pub signers: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartChainReportSummary {
    pub status: String,
    pub chains_total: usize,
    pub chains_valid: usize,
    pub chains_invalid: usize,
    pub errors_total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSection {
    pub commit: Option<String>,
    pub branch: Option<String>,
    pub pr_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartChainValidationReport {
    pub timestamp: String,
    pub repo: String,
    pub smart_chain_config: String,
    pub git: GitSection,
    pub summary: SmartChainReportSummary,
    pub errors: Vec<SmartChainErrorEntry>,
    pub chains: Vec<SmartChainChainStatus>,
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn utc_now() -> String {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    let secs = since_epoch.as_secs();
    let nanos = since_epoch.subsec_nanos();
    format!("{}.{:09}Z", secs, nanos)
}

fn pq_mode_to_string(mode: &PQMode) -> String {
    match mode {
        PQMode::ClassicalOnly => "CLASSICAL_ONLY".to_string(),
        PQMode::Hybrid => "HYBRID".to_string(),
        PQMode::PQStrict => "PQSTRICT".to_string(),
    }
}

/// Best-effort JSON Pointer hints for config locations.
fn json_pointer_for_error(err: &ValidationError) -> Option<String> {
    use ValidationError::*;

    match err {
        MissingChainInIndex { chain_id } => Some(format!(
            "/smartchains/*[id='{}']",
            chain_id
        )),
        UnknownChainInIndex { chain_id } => Some(format!(
            "/index/chains/*[id='{}']",
            chain_id
        )),
        RollbackNotForbidden { chain_id } => Some(format!(
            "/smartchains/*[id='{}']/execution/rollbackforbidden",
            chain_id
        )),
        PQModeTooWeak { chain_id, .. } => Some(format!(
            "/smartchains/*[id='{}']/trust/pq_mode",
            chain_id
        )),
        MissingTreatyForDomain { chain_id, .. } => Some(format!(
            "/smartchains/*[id='{}']/accepts_restraints/treaties",
            chain_id
        )),
        MissingRightsForDomain { chain_id, .. } => Some(format!(
            "/smartchains/*[id='{}']/accepts_restraints/rightsgrammars",
            chain_id
        )),
        MissingSomaticEnvelope { chain_id, .. } => Some(format!(
            "/smartchains/*[id='{}']/accepts_restraints/somaticenvelopes",
            chain_id
        )),
        MultisigTooLow { chain_id, .. } => Some(format!(
            "/smartchains/*[id='{}']/trust/multisigthreshold",
            chain_id
        )),
        Io { .. } | Parse { .. } => None,
    }
}

// -----------------------------------------------------------------------------
// Error mapping
// -----------------------------------------------------------------------------

fn map_error(err: &ValidationError, config_path: &str) -> SmartChainErrorEntry {
    use ValidationError::*;

    let severity = "error".to_string();
    let mut context = SmartChainErrorContext::default();
    let mut suggested_remediation: Vec<String> = Vec::new();

    let (chain_id, error_code, message): (Option<String>, String, String) = match err {
        Io(e) => {
            let msg = format!("IO error reading SMART chain config: {}", e);
            suggested_remediation.push(
                "Verify the config path exists and is readable, then re-run the validator."
                    .to_string(),
            );
            (None, "Io".to_string(), msg)
        }
        Parse(e) => {
            let msg = format!("Failed to parse SMART chain JSON: {}", e);
            suggested_remediation.push(
                "Ensure SMART-HIERARCHY-CHAINS-001.json is valid JSON and matches the schema."
                    .to_string(),
            );
            (None, "Parse".to_string(), msg)
        }
        MissingChainInIndex { chain_id } => {
            suggested_remediation.push(
                "Add this chain ID to the index.chains array so it is discoverable by other modules."
                    .to_string(),
            );
            (
                Some(chain_id.clone()),
                "MissingChainInIndex".to_string(),
                format!(
                    "Chain '{}' is defined but not registered in index.chains.",
                    chain_id
                ),
            )
        }
        UnknownChainInIndex { chain_id } => {
            suggested_remediation.push(
                "Remove this ID from index.chains or define a matching chain with that ID."
                    .to_string(),
            );
            (
                Some(chain_id.clone()),
                "UnknownChainInIndex".to_string(),
                format!(
                    "Chain '{}' is listed in index.chains but no matching SmartChain is defined.",
                    chain_id
                ),
            )
        }
        RollbackNotForbidden { chain_id } => {
            suggested_remediation.push(
                "Set execution.rollbackforbidden = true for this chain; GOD-city rules require non-reversible chains."
                    .to_string(),
            );
            (
                Some(chain_id.clone()),
                "RollbackNotForbidden".to_string(),
                format!(
                    "Chain '{}' has rollbackforbidden = false, which violates GOD-city invariants.",
                    chain_id
                ),
            )
        }
        PQModeTooWeak {
            chain_id,
            domain,
            required,
            found,
        } => {
            context.domain = Some(domain.clone());
            context.found_pq_mode = Some(pq_mode_to_string(found));
            context.required_pq_mode = Some(pq_mode_to_string(required));
            suggested_remediation.push(
                "Upgrade trust.pq_mode to the required level for this domain (e.g., PQSTRICT for biotic/microbiome)."
                    .to_string(),
            );
            suggested_remediation.push(
                "Coordinate with the PQ crypto layer to ensure keys and signatures are available before changing pq_mode."
                    .to_string(),
            );
            (
                Some(chain_id.clone()),
                "PQModeTooWeak".to_string(),
                format!(
                    "Chain '{}' uses PQ mode '{}' which is too weak for domain '{}'; required: '{}'.",
                    chain_id,
                    pq_mode_to_string(found),
                    domain,
                    pq_mode_to_string(required),
                ),
            )
        }
        MissingTreatyForDomain { chain_id, domain } => {
            context.domain = Some(domain.clone());
            suggested_remediation.push(
                "Attach at least one Indigenous/biotic treaty ID to accepts_restraints.treaties for this chain."
                    .to_string(),
            );
            suggested_remediation.push(
                "Verify that water and biotic domains reference the correct treaty set for the affected region."
                    .to_string(),
            );
            (
                Some(chain_id.clone()),
                "MissingTreatyForDomain".to_string(),
                format!(
                    "Chain '{}' covers domain '{}' but has no required treaty bindings.",
                    chain_id, domain
                ),
            )
        }
        MissingRightsForDomain { chain_id, domain } => {
            context.domain = Some(domain.clone());
            suggested_remediation.push(
                "Add at least one LexEthos rights grammar ID under accepts_restraints.rightsgrammars for this chain."
                    .to_string(),
            );
            suggested_remediation.push(
                "Ensure somatic/thermal/movement chains include the appropriate rights grammars (e.g., RightToShade, RightToSafeMovement)."
                    .to_string(),
            );
            (
                Some(chain_id.clone()),
                "MissingRightsForDomain".to_string(),
                format!(
                    "Chain '{}' covers domain '{}' but is missing required LexEthos rights grammars.",
                    chain_id, domain
                ),
            )
        }
        MissingSomaticEnvelope { chain_id, domain } => {
            context.domain = Some(domain.clone());
            suggested_remediation.push(
                "Populate accepts_restraints.somaticenvelopes with one or more envelope IDs for this chain."
                    .to_string(),
            );
            suggested_remediation.push(
                "Wire Somaplex routing and corridor functions to use these somatic envelopes before actuation."
                    .to_string(),
            );
            (
                Some(chain_id.clone()),
                "MissingSomaticEnvelope".to_string(),
                format!(
                    "Chain '{}' covers domain '{}' but has no somatic envelopes defined.",
                    chain_id, domain
                ),
            )
        }
        MultisigTooLow { chain_id, threshold } => {
            context.multisig_threshold = Some(*threshold);
            suggested_remediation.push(
                "Increase trust.multisigthreshold to at least 2 for city-level actions, and ensure signers includes all required roles."
                    .to_string(),
            );
            (
                Some(chain_id.clone()),
                "MultisigTooLow".to_string(),
                format!(
                    "Chain '{}' has multisigthreshold = {}, which is below the required minimum.",
                    chain_id, threshold
                ),
            )
        }
    };

    let location = json_pointer_for_error(err).map(|ptr| SmartChainErrorLocation {
        file: config_path.to_string(),
        json_pointer: ptr,
    });

    let ctx_opt = if context.domain.is_some()
        || context.found_pq_mode.is_some()
        || context.required_pq_mode.is_some()
        || context.multisig_threshold.is_some()
    {
        Some(context)
    } else {
        None
    };

    SmartChainErrorEntry {
        chain_id,
        error_code,
        severity,
        message,
        location,
        context: ctx_opt,
        suggested_remediation,
    }
}

// -----------------------------------------------------------------------------
// Chain status construction
// -----------------------------------------------------------------------------

fn build_chain_statuses(
    cfg: &SmartChainConfig,
    errors: &[SmartChainErrorEntry],
) -> Vec<SmartChainChainStatus> {
    let mut chain_errors: HashMap<String, Vec<String>> = HashMap::new();
    for e in errors {
        if let Some(ref id) = e.chain_id {
            chain_errors
                .entry(id.clone())
                .or_default()
                .push(e.error_code.clone());
        }
    }

    cfg.smartchains
        .iter()
        .map(|chain: &SmartChain| {
            let id = chain.id.clone();
            let errs = chain_errors
                .get(&id)
                .cloned()
                .unwrap_or_else(Vec::new);
            let status = if errs.is_empty() {
                "valid".to_string()
            } else {
                "invalid".to_string()
            };

            SmartChainChainStatus {
                id: id.clone(),
                name: chain.name.clone(),
                status,
                domains: chain.scope.domains.clone(),
                city_region: chain.scope.cityregion.clone(),
                pq_mode: pq_mode_to_string(&chain.trust.pqmode),
                rollbackforbidden: chain.execution.rollbackforbidden,
                multisig_threshold: chain.trust.multisigthreshold,
                signers: chain.trust.signers.clone(),
                errors: errs,
            }
        })
        .collect()
}

// -----------------------------------------------------------------------------
// Report builders
// -----------------------------------------------------------------------------

pub fn build_report(
    cfg: &SmartChainConfig,
    _registry: &SmartChainRegistry,
    raw_errors: &[ValidationError],
    meta: SmartChainReportMeta,
) -> SmartChainValidationReport {
    let mut errors: Vec<SmartChainErrorEntry> = raw_errors
        .iter()
        .map(|e| map_error(e, &meta.config_path))
        .collect();

    // Deduplicate identical entries to keep CI logs cleaner.
    let mut seen: HashSet<(Option<String>, String, String)> = HashSet::new();
    errors.retain(|e| {
        let key = (e.chain_id.clone(), e.error_code.clone(), e.message.clone());
        if seen.contains(&key) {
            false
        } else {
            seen.insert(key);
            true
        }
    });

    let chains = build_chain_statuses(cfg, &errors);

    let chains_total = chains.len();
    let chains_invalid = chains
        .iter()
        .filter(|c| c.status == "invalid")
        .count();
    let chains_valid = chains_total.saturating_sub(chains_invalid);
    let errors_total = errors.len();
    let status = if errors_total > 0 {
        "fail".to_string()
    } else {
        "pass".to_string()
    };

    let summary = SmartChainReportSummary {
        status,
        chains_total,
        chains_valid,
        chains_invalid,
        errors_total,
    };

    let git = GitSection {
        commit: meta.git_commit.clone(),
        branch: meta.git_branch.clone(),
        pr_number: meta.git_pr_number,
    };

    SmartChainValidationReport {
        timestamp: meta.timestamp.clone(),
        repo: meta.repo.clone(),
        smart_chain_config: meta.config_path.clone(),
        git,
        summary,
        errors,
        chains,
    }
}

// Public entry for validator binary / CI integration
pub fn write_report_to_file(
    cfg: &SmartChainConfig,
    registry: &SmartChainRegistry,
    errors: &[ValidationError],
    output_path: &str,
    repo: &str,
    git_commit: Option<String>,
    git_branch: Option<String>,
    git_pr_number: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let meta = SmartChainReportMeta {
        timestamp: utc_now(),
        repo: repo.to_string(),
        config_path: "aletheion/erm-workflow-index/SMART-HIERARCHY-CHAINS-001.json"
            .to_string(),
        git_commit,
        git_branch,
        git_pr_number,
    };

    let report = build_report(cfg, registry, errors, meta);
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write(output_path, json)?;
    Ok(())
}
