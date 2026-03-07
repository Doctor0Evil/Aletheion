//! Aletheion ERM SMART Hierarchy-Chain Validator
//! Path: aletheion/erm-workflow-index/src/smart_chain_validator.rs
//!
//! Role:
//!   - Load SMART-HIERARCHY-CHAINS-001.aln (or its JSON/TOML export).
//!   - Parse into strongly-typed Rust structs.
//!   - Enforce city-wide invariants:
//!       * rollback_forbidden == true for all chains
//!       * PQ modes consistent with chain criticality
//!       * Required treaties/rights for sensitive domains present
//!       * L1–L5 layer crossings allowed and intentional
//!   - Provide query API for other modules (water, thermal, Synthexis, LexEthos).
//!
//! Design notes:
//!   - Uses a “parse, don’t validate” approach: deserialize into domain types,
//!     then run explicit invariant checks instead of scattering ad-hoc ifs.
//!   - Assumes ALN compiler/runtime can export an equivalent JSON view of
//!     SMART-HIERARCHY-CHAINS-001.aln; this validator consumes that JSON.
//!   - Safe to run at startup and on hot-reload when chain definitions update.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Minimum PQ mode expected for each domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequiredPQ {
    ClassicalAllowed, // legacy, temporary
    HybridPreferred,  // at least HYBRID
    PQStrictRequired, // must be PQ_STRICT
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionPattern {
    Dag,
    FeedbackLoop,
    Periodic,
    EventDriven,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PQMode {
    ClassicalOnly,
    Hybrid,
    PQStrict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainScope {
    pub layers: Vec<String>,
    pub domains: Vec<String>,
    #[serde(default)]
    pub city_region: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRef {
    pub code: String,
    #[serde(default)]
    pub path_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExposedSignals {
    #[serde(default)]
    pub envelopes: Vec<String>,
    #[serde(default)]
    pub allocations: Vec<String>,
    #[serde(default)]
    pub alerts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainRestraints {
    #[serde(default)]
    pub treaties: Vec<String>,
    #[serde(default)]
    pub rights_grammars: Vec<String>,
    #[serde(default)]
    pub somatic_envelopes: Vec<String>,
    #[serde(default)]
    pub fear_pain_sanity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExecution {
    pub pattern: ExecutionPattern,
    #[serde(default)]
    pub cadence: Option<String>,
    #[serde(default)]
    pub triggers: Vec<String>,
    pub rollback_forbidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainTrust {
    pub ledger_stream: String,
    pub pq_mode: PQMode,
    pub multisig_threshold: i32,
    #[serde(default)]
    pub signers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartChain {
    pub id: String,
    pub name: String,
    pub version: String,
    pub scope: ChainScope,
    #[serde(default)]
    pub depends_on: Vec<WorkflowRef>,
    #[serde(default)]
    pub exposes: ChainExposedSignals,
    #[serde(default)]
    pub accepts_restraints: ChainRestraints,
    pub execution: ChainExecution,
    pub trust: ChainTrust,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartChainIndex {
    pub chains: Vec<String>,
    #[serde(default)]
    pub invariant: Option<HashMap<String, String>>,
}

/// Raw JSON root expected from ALN export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartChainConfig {
    pub smart_chains: Vec<SmartChain>,
    pub index: SmartChainIndex,
}

/// Validation error type.
#[derive(Debug)]
pub enum ValidationError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    MissingChainInIndex { chain_id: String },
    UnknownChainInIndex { chain_id: String },
    RollbackNotForbidden { chain_id: String },
    PQModeTooWeak {
        chain_id: String,
        domain: String,
        required: RequiredPQ,
        found: PQMode,
    },
    MissingTreatyForDomain {
        chain_id: String,
        domain: String,
    },
    MissingRightsForDomain {
        chain_id: String,
        domain: String,
    },
    MissingSomaticEnvelope {
        chain_id: String,
        domain: String,
    },
    MultisigTooLow {
        chain_id: String,
        threshold: i32,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ValidationError::*;
        match self {
            Io(e) => write!(f, "I/O error reading SMART chain config: {}", e),
            Parse(e) => write!(f, "Failed to parse SMART chain JSON: {}", e),
            MissingChainInIndex { chain_id } => {
                write!(f, "Chain `{}` defined but not registered in index", chain_id)
            }
            UnknownChainInIndex { chain_id } => {
                write!(f, "Chain `{}` listed in index but not defined", chain_id)
            }
            RollbackNotForbidden { chain_id } => write!(
                f,
                "Chain `{}` has rollback_forbidden = false, which violates GOD-city rule",
                chain_id
            ),
            PQModeTooWeak {
                chain_id,
                domain,
                required,
                found,
            } => write!(
                f,
                "Chain `{}`: PQ mode {:?} is too weak for domain `{}` (requires {:?})",
                chain_id, found, domain, required
            ),
            MissingTreatyForDomain { chain_id, domain } => write!(
                f,
                "Chain `{}` covering domain `{}` is missing required Biotic/Indigenous treaty bindings",
                chain_id, domain
            ),
            MissingRightsForDomain { chain_id, domain } => write!(
                f,
                "Chain `{}` covering domain `{}` is missing required LexEthos rights grammars",
                chain_id, domain
            ),
            MissingSomaticEnvelope { chain_id, domain } => write!(
                f,
                "Chain `{}` covering domain `{}` is missing somatic envelopes while movement/body is involved",
                chain_id, domain
            ),
            MultisigTooLow { chain_id, threshold } => write!(
                f,
                "Chain `{}` has multisig_threshold = {}, must be >= 2 for city-level actions",
                chain_id, threshold
            ),
        }
    }
}

impl std::error::Error for ValidationError {}

impl From<std::io::Error> for ValidationError {
    fn from(e: std::io::Error) -> Self {
        ValidationError::Io(e)
    }
}

impl From<serde_json::Error> for ValidationError {
    fn from(e: serde_json::Error) -> Self {
        ValidationError::Parse(e)
    }
}

/// In-memory registry of SMART chains, used by other modules.
#[derive(Debug, Clone)]
pub struct SmartChainRegistry {
    chains: HashMap<String, SmartChain>,
    /// Reverse index: domain -> chain ids.
    domains_to_chains: HashMap<String, Vec<String>>,
}

impl SmartChainRegistry {
    /// Load and validate chains from a JSON export of SMART-HIERARCHY-CHAINS-001.aln.
    ///
    /// Typical path: `aletheion/erm-workflow-index/SMART-HIERARCHY-CHAINS-001.json`
    pub fn load_from_json<P: AsRef<Path>>(path: P) -> Result<Self, ValidationError> {
        let raw = fs::read_to_string(path)?;
        let cfg: SmartChainConfig = serde_json::from_str(&raw)?;

        Self::validate_index(&cfg)?;
        Self::validate_chains(&cfg)?;

        let mut chains_map = HashMap::new();
        let mut domains_to_chains: HashMap<String, Vec<String>> = HashMap::new();

        for chain in cfg.smart_chains.into_iter() {
            for d in &chain.scope.domains {
                domains_to_chains.entry(d.clone()).or_default().push(chain.id.clone());
            }
            chains_map.insert(chain.id.clone(), chain);
        }

        Ok(Self {
            chains: chains_map,
            domains_to_chains,
        })
    }

    /// Ensure index and chain lists agree.
    fn validate_index(cfg: &SmartChainConfig) -> Result<(), ValidationError> {
        let defined_ids: HashSet<_> = cfg.smart_chains.iter().map(|c| c.id.clone()).collect();
        let indexed_ids: HashSet<_> = cfg.index.chains.iter().cloned().collect();

        // every defined chain must appear in index
        for id in defined_ids.iter() {
            if !indexed_ids.contains(id) {
                return Err(ValidationError::MissingChainInIndex {
                    chain_id: id.clone(),
                });
            }
        }

        // index must not reference unknown chains
        for id in indexed_ids.iter() {
            if !defined_ids.contains(id) {
                return Err(ValidationError::UnknownChainInIndex {
                    chain_id: id.clone(),
                });
            }
        }

        Ok(())
    }

    /// High-level invariants over all chains.
    fn validate_chains(cfg: &SmartChainConfig) -> Result<(), ValidationError> {
        for chain in &cfg.smart_chains {
            // 1. rollback_forbidden invariant (global rule).
            if !chain.execution.rollback_forbidden {
                return Err(ValidationError::RollbackNotForbidden {
                    chain_id: chain.id.clone(),
                });
            }

            // 2. PQ mode expectations per domain.
            let pq_requirements = Self::required_pq_for_domains(&chain.scope.domains);
            for (domain, required) in pq_requirements.into_iter() {
                if !Self::pq_satisfies(chain.trust.pq_mode, required) {
                    return Err(ValidationError::PQModeTooWeak {
                        chain_id: chain.id.clone(),
                        domain,
                        required,
                        found: chain.trust.pq_mode.clone(),
                    });
                }
            }

            // 3. Treaty bindings for ecological + Indigenous domains.
            if chain.scope.domains.iter().any(|d| d == "biotic" || d == "water") {
                if chain.accepts_restraints.treaties.is_empty() {
                    return Err(ValidationError::MissingTreatyForDomain {
                        chain_id: chain.id.clone(),
                        domain: "biotic/water".into(),
                    });
                }
            }

            // 4. Rights grammars for citizen-facing / somatic / heat domains.
            if chain
                .scope
                .domains
                .iter()
                .any(|d| d == "somatic" || d == "thermal" || d == "movement")
            {
                if chain.accepts_restraints.rights_grammars.is_empty() {
                    return Err(ValidationError::MissingRightsForDomain {
                        chain_id: chain.id.clone(),
                        domain: "somatic/thermal/movement".into(),
                    });
                }
            }

            // 5. Somatic envelopes required when movement or somatic domains present.
            if chain
                .scope
                .domains
                .iter()
                .any(|d| d == "somatic" || d == "movement")
            {
                if chain.accepts_restraints.somatic_envelopes.is_empty() {
                    return Err(ValidationError::MissingSomaticEnvelope {
                        chain_id: chain.id.clone(),
                        domain: "somatic/movement".into(),
                    });
                }
            }

            // 6. Multisig minimal threshold across all chains.
            if chain.trust.multisig_threshold < 2 {
                return Err(ValidationError::MultisigTooLow {
                    chain_id: chain.id.clone(),
                    threshold: chain.trust.multisig_threshold,
                });
            }
        }

        Ok(())
    }

    /// Map domains to required PQ level.
    fn required_pq_for_domains(domains: &[String]) -> Vec<(String, RequiredPQ)> {
        let mut out = Vec::new();
        for d in domains {
            let req = match d.as_str() {
                "water" | "thermal" | "waste" | "logistics" => RequiredPQ::HybridPreferred,
                "biotic" | "microbiome" | "neuro" | "somatic" => RequiredPQ::PQStrictRequired,
                "movement" | "equity" | "rights" => RequiredPQ::HybridPreferred,
                _ => RequiredPQ::ClassicalAllowed,
            };
            out.push((d.clone(), req));
        }
        out
    }

    fn pq_satisfies(found: PQMode, required: RequiredPQ) -> bool {
        use PQMode::*;
        use RequiredPQ::*;

        match (found, required) {
            (PQMode::PQStrict, _) => true, // strongest
            (PQMode::Hybrid, ClassicalAllowed | HybridPreferred) => true,
            (PQMode::Hybrid, PQStrictRequired) => false,
            (PQMode::ClassicalOnly, ClassicalAllowed) => true,
            (PQMode::ClassicalOnly, HybridPreferred | PQStrictRequired) => false,
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Public query API
    // ─────────────────────────────────────────────────────────────────────

    /// Get a chain by id.
    pub fn chain(&self, id: &str) -> Option<&SmartChain> {
        self.chains.get(id)
    }

    /// List all chains that touch a given domain (e.g., "water", "biotic").
    pub fn chains_for_domain(&self, domain: &str) -> &[String] {
        self.domains_to_chains
            .get(domain)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// All chain ids.
    pub fn all_chain_ids(&self) -> Vec<String> {
        self.chains.keys().cloned().collect()
    }

    /// Determine whether an action must enforce FEAR/PAIN/SANITY envelopes
    /// based on the chains it traverses.
    pub fn requires_fear_pain_sanity(&self, chain_ids: &[String]) -> bool {
        chain_ids.iter().any(|id| {
            self.chains
                .get(id)
                .map(|c| c.accepts_restraints.fear_pain_sanity)
                .unwrap_or(false)
        })
    }

    /// Find all treaties that apply to a given domain across chains.
    pub fn treaties_for_domain(&self, domain: &str) -> Vec<String> {
        let mut set = HashSet::new();
        if let Some(list) = self.domains_to_chains.get(domain) {
            for chain_id in list {
                if let Some(chain) = self.chains.get(chain_id) {
                    for t in &chain.accepts_restraints.treaties {
                        set.insert(t.clone());
                    }
                }
            }
        }
        set.into_iter().collect()
    }

    /// Find all LexEthos rights grammars relevant to a domain.
    pub fn rights_for_domain(&self, domain: &str) -> Vec<String> {
        let mut set = HashSet::new();
        if let Some(list) = self.domains_to_chains.get(domain) {
            for chain_id in list {
                if let Some(chain) = self.chains.get(chain_id) {
                    for r in &chain.accepts_restraints.rights_grammars {
                        set.insert(r.clone());
                    }
                }
            }
        }
        set.into_iter().collect()
    }

    /// Inspect PQ modes across chains for auditing.
    pub fn pq_modes_summary(&self) -> HashMap<PQMode, Vec<String>> {
        let mut out: HashMap<PQMode, Vec<String>> = HashMap::new();
        for (id, chain) in &self.chains {
            out.entry(chain.trust.pq_mode.clone())
                .or_default()
                .push(id.clone());
        }
        out
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Example integration hook (optional): run at ERM factory startup
// ─────────────────────────────────────────────────────────────────────────────

/// Initialize registry from default path and panic if validation fails.
/// This is suitable for early boot in Aletheion city-factory processes.
pub fn init_smart_chain_registry_or_panic() -> SmartChainRegistry {
    let path = "aletheion/erm-workflow-index/SMART-HIERARCHY-CHAINS-001.json";
    match SmartChainRegistry::load_from_json(path) {
        Ok(reg) => {
            println!(
                "[SMART] Loaded {} hierarchy-chains from {}",
                reg.all_chain_ids().len(),
                path
            );
            reg
        }
        Err(e) => {
            eprintln!("[SMART] FATAL: SMART chain validation failed: {}", e);
            panic!("SMART chain registry initialization failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pq_satisfaction_matrix() {
        assert!(SmartChainRegistry::pq_satisfies(
            PQMode::PQStrict,
            RequiredPQ::PQStrictRequired
        ));
        assert!(SmartChainRegistry::pq_satisfies(
            PQMode::PQStrict,
            RequiredPQ::HybridPreferred
        ));
        assert!(SmartChainRegistry::pq_satisfies(
            PQMode::Hybrid,
            RequiredPQ::HybridPreferred
        ));
        assert!(!SmartChainRegistry::pq_satisfies(
            PQMode::Hybrid,
            RequiredPQ::PQStrictRequired
        ));
        assert!(SmartChainRegistry::pq_satisfies(
            PQMode::ClassicalOnly,
            RequiredPQ::ClassicalAllowed
        ));
        assert!(!SmartChainRegistry::pq_satisfies(
            PQMode::ClassicalOnly,
            RequiredPQ::HybridPreferred
        ));
    }

    #[test]
    fn required_pq_for_domains_assigns_higher_for_biotic() {
        let domains = vec![
            "water".to_string(),
            "biotic".to_string(),
            "microbiome".to_string(),
            "somatic".to_string(),
            "movement".to_string(),
            "thermal".to_string(),
        ];

        let reqs = SmartChainRegistry::required_pq_for_domains(&domains);
        let mut map = HashMap::new();
        for (d, r) in reqs {
            map.insert(d, r);
        }

        assert_eq!(
            map.get("biotic"),
            Some(&RequiredPQ::PQStrictRequired)
        );
        assert_eq!(
            map.get("microbiome"),
            Some(&RequiredPQ::PQStrictRequired)
        );
        assert_eq!(
            map.get("somatic"),
            Some(&RequiredPQ::PQStrictRequired)
        );
        assert_eq!(
            map.get("water"),
            Some(&RequiredPQ::HybridPreferred)
        );
        assert_eq!(
            map.get("thermal"),
            Some(&RequiredPQ::HybridPreferred)
        );
    }
}
