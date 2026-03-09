// Domain: infra_orchestration
// Role: Edge orchestration model for Aletheion Phoenix.
//        - Describes edge nodes (routers, appliances, MCUs, vehicles).
//        - Captures energy, thermal, connectivity, and security profiles.
//        - Encodes suitability scoring with BirthSign-governance awareness.[file:2][file:5]
//
// ERM layers: L1 Edge Sensing, L2 State Modeling, L4 Optimization.
// Language: Rust only (no blacklisted terms).

#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Spatial governance tile identifier (forward-compatible with Birth-Signs). [file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BirthSignId(pub String);

/// Logical identifier for an edge node profile (firmware + hardware bundle).[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeProfileId(pub String);

/// Unique identifier for a concrete edge node instance in the city.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgeNodeId(pub String);

/// High-level hardware class for scheduling and placement decisions.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HardwareClass {
    Router,
    Appliance,
    Mcu,
    Vehicle,
    EdgeServer,
}

/// Qualitative energy availability class for a node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EnergyTier {
    GridStable,
    GridConstrained,
    BatteryOnly,
    HarvestingSolar,
    HarvestingMixed,
}

/// Simplified thermal risk band for node placement.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ThermalBand {
    Cool,
    Normal,
    Warm,
    Hot,
}

/// Connectivity tier, considering bandwidth, latency, and stability.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConnectivityTier {
    HighThroughputLowLatency,
    Moderate,
    Intermittent,
    LowBandwidth,
}

/// Security posture of a node profile, reused from governance research.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeSecurityTier {
    /// Trusted Execution Environment available; suitable for sensitive governance / consent.
    TeeBacked,
    /// Hardened firmware channel with signed manifests; no TEE.
    HardenedFirmwareOnly,
    /// Basic node; suitable only for low-risk sensing or noncritical tasks.
    Basic,
}

/// Summary of security features on a given node instance.[file:2]
#[derive(Debug, Clone)]
pub struct NodeSecurityProfile {
    pub node_profile_id: NodeProfileId,
    pub tier: NodeSecurityTier,
    /// Secure boot active and enforced.
    pub secure_boot: bool,
    /// Firmware updates are signed and verified.
    pub signed_updates: bool,
    /// TLS/DTLS enforced for node control traffic.
    pub secure_transport: bool,
    /// Timestamp of last successful security audit.
    pub last_audit_passed_at: Option<SystemTime>,
}

impl NodeSecurityProfile {
    /// True if this node can host highly sensitive governance workloads
    /// (e.g., consent processing, treaty evaluation, DID handling).[file:2]
    pub fn can_host_sensitive_governance(&self) -> bool {
        matches!(self.tier, NodeSecurityTier::TeeBacked)
            && self.secure_boot
            && self.signed_updates
            && self.secure_transport
    }

    /// True if this node is suitable for generic ERM logic that is not
    /// biosignal- or identity-sensitive.[file:2]
    pub fn can_host_general_erm(&self) -> bool {
        match self.tier {
            NodeSecurityTier::TeeBacked | NodeSecurityTier::HardenedFirmwareOnly => true,
            NodeSecurityTier::Basic => false,
        }
    }
}

/// Static capabilities and resource shape of an edge node.
#[derive(Debug, Clone)]
pub struct EdgeNodeResources {
    pub cpu_cores: u8,
    pub memory_mb: u32,
    pub storage_mb: u32,
    /// Approximate sustained power budget for Aletheion tasks (watts).
    pub power_budget_w: f32,
}

/// Dynamic energy profile snapshot.
#[derive(Debug, Clone)]
pub struct EnergyProfile {
    pub tier: EnergyTier,
    /// Fraction [0.0, 1.0] of budget currently usable for Aletheion workloads.
    pub available_fraction: f32,
    /// True if node is currently in an explicitly energy-scarce window.
    pub scarcity_flag: bool,
}

/// Dynamic thermal profile snapshot.[file:2]
#[derive(Debug, Clone)]
pub struct ThermalProfile {
    pub band: ThermalBand,
    /// Estimated margin before local thermal constraints are violated (degrees C).
    pub margin_c: f32,
}

/// Dynamic connectivity profile snapshot.
#[derive(Debug, Clone)]
pub struct ConnectivityProfile {
    pub tier: ConnectivityTier,
    /// Nominal upstream bandwidth (Mbps).
    pub upstream_mbps: f32,
    /// Nominal downstream bandwidth (Mbps).
    pub downstream_mbps: f32,
    /// Approximate round-trip latency (milliseconds).
    pub latency_ms: f32,
    /// Probability [0.0, 1.0] of being online in the next scheduling horizon.
    pub availability_prob: f32,
}

/// Governance domains that may constrain task placement; aligned with Birth-Signs.[file:2]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GovernanceDomain {
    Land,
    Water,
    Air,
    Materials,
    Mobility,
    Biosignals,
    Augmentation,
    Energy,
    Culture,
    Emergency,
}

/// High-level task class for orchestrator decisions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskClass {
    /// ERM ingestion / state update close to sensors.
    IngestionStateModel,
    /// Heavier optimization / planning workloads.
    Optimization,
    /// Governance and compliance checks.
    Governance,
    /// Actuation orchestration and control loops.
    Actuation,
    /// Citizen interface / notification tasks.
    CitizenInterface,
}

/// Sensitivity band for task data / impact.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaskSensitivity {
    Low,
    Medium,
    High,
}

/// Governance and territorial requirements for a task.[file:2]
#[derive(Debug, Clone)]
pub struct TaskGovernanceProfile {
    pub domains: Vec<GovernanceDomain>,
    pub required_birth_signs: Vec<BirthSignId>,
    pub sensitivity: TaskSensitivity,
    /// True if task requires that all execution remain within a set of tiles.
    pub jurisdiction_lock: bool,
}

/// Resource requirements and governance constraints for a single task instance.
#[derive(Debug, Clone)]
pub struct TaskInstance {
    pub id: String,
    pub class: TaskClass,
    pub cpu_cores_required: u8,
    pub memory_mb_required: u32,
    pub duration_hint: Duration,
    pub governance: TaskGovernanceProfile,
}

/// Full edge node descriptor used for placement.
#[derive(Debug, Clone)]
pub struct EdgeNode {
    pub id: EdgeNodeId,
    pub profile_id: NodeProfileId,
    pub hardware_class: HardwareClass,
    pub resources: EdgeNodeResources,
    pub energy: EnergyProfile,
    pub thermal: ThermalProfile,
    pub connectivity: ConnectivityProfile,
    pub security: NodeSecurityProfile,
    /// Tiles this node is physically associated with (for jurisdiction-aware placement).
    pub birth_signs: Vec<BirthSignId>,
    /// Free-form tags for domain-specific extensions.
    pub tags: HashMap<String, String>,
}

impl EdgeNode {
    /// True if this node is physically within at least one of the task's required tiles.
    pub fn satisfies_jurisdiction(&self, task: &TaskInstance) -> bool {
        if task.governance.required_birth_signs.is_empty() {
            return true;
        }
        self.birth_signs
            .iter()
            .any(|b| task.governance.required_birth_signs.contains(b))
    }

    /// Compute a suitability score in [0.0, 1.0] for placing the given task here.
    /// Returns 0.0 if hard constraints (resources, security, strict jurisdiction) fail.[file:2]
    pub fn suitability_score(&self, task: &TaskInstance) -> f32 {
        // Hard resource checks.
        if task.cpu_cores_required as u32 > self.resources.cpu_cores as u32 {
            return 0.0;
        }
        if task.memory_mb_required > self.resources.memory_mb {
            return 0.0;
        }

        // Hard governance constraints.
        if task.governance.jurisdiction_lock && !self.satisfies_jurisdiction(task) {
            return 0.0;
        }

        // Security gating by sensitivity.
        match task.governance.sensitivity {
            TaskSensitivity::High => {
                if !self.security.can_host_sensitive_governance() {
                    return 0.0;
                }
            }
            TaskSensitivity::Medium => {
                if !(self.security.can_host_general_erm()
                    || self.security.can_host_sensitive_governance())
                {
                    return 0.0;
                }
            }
            TaskSensitivity::Low => {}
        }

        // Soft factors combined into a multiplicative score.
        let energy_score = Self::score_energy(&self.energy, task);
        let thermal_score = Self::score_thermal(&self.thermal, task);
        let conn_score = Self::score_connectivity(&self.connectivity, task);
        let jurisdiction_score = if task.governance.required_birth_signs.is_empty() {
            1.0
        } else if self.satisfies_jurisdiction(task) {
            1.0
        } else {
            0.5 // soft penalty if not locked but still misaligned.
        };

        let raw = energy_score * thermal_score * conn_score * jurisdiction_score;
        // Clamp to [0.0, 1.0].
        raw.max(0.0).min(1.0)
    }

    fn score_energy(energy: &EnergyProfile, task: &TaskInstance) -> f32 {
        if energy.scarcity_flag {
            return 0.2;
        }
        let base = match energy.tier {
            EnergyTier::GridStable => 1.0,
            EnergyTier::GridConstrained => 0.7,
            EnergyTier::BatteryOnly => 0.5,
            EnergyTier::HarvestingSolar | EnergyTier::HarvestingMixed => 0.8,
        };
        let mut score = base * energy.available_fraction;
        if matches!(task.class, TaskClass::Optimization) && energy.tier == EnergyTier::BatteryOnly {
            score *= 0.6;
        }
        score.max(0.0).min(1.0)
    }

    fn score_thermal(thermal: &ThermalProfile, _task: &TaskInstance) -> f32 {
        let base = match thermal.band {
            ThermalBand::Cool => 1.0,
            ThermalBand::Normal => 0.9,
            ThermalBand::Warm => 0.7,
            ThermalBand::Hot => 0.3,
        };
        let margin_factor = if thermal.margin_c <= 0.0 {
            0.2
        } else if thermal.margin_c < 3.0 {
            0.5
        } else if thermal.margin_c < 7.0 {
            0.8
        } else {
            1.0
        };
        (base * margin_factor).max(0.0).min(1.0)
    }

    fn score_connectivity(conn: &ConnectivityProfile, task: &TaskInstance) -> f32 {
        let base = match conn.tier {
            ConnectivityTier::HighThroughputLowLatency => 1.0,
            ConnectivityTier::Moderate => 0.8,
            ConnectivityTier::Intermittent => 0.4,
            ConnectivityTier::LowBandwidth => 0.5,
        };
        let latency_penalty = match task.class {
            TaskClass::CitizenInterface | TaskClass::Actuation => {
                if conn.latency_ms > 200.0 {
                    0.5
                } else if conn.latency_ms > 80.0 {
                    0.8
                } else {
                    1.0
                }
            }
            _ => 1.0,
        };
        let availability_factor = conn.availability_prob;
        (base * latency_penalty * availability_factor)
            .max(0.0)
            .min(1.0)
    }
}

/// A simple scoring result used by higher-level orchestrators.
#[derive(Debug, Clone)]
pub struct PlacementScore {
    pub node_id: EdgeNodeId,
    pub task_id: String,
    pub score: f32,
}

/// Compute placement scores for a task across candidate nodes.
pub fn rank_nodes_for_task(nodes: &[EdgeNode], task: &TaskInstance) -> Vec<PlacementScore> {
    let mut scores: Vec<PlacementScore> = nodes
        .iter()
        .map(|n| PlacementScore {
            node_id: n.id.clone(),
            task_id: task.id.clone(),
            score: n.suitability_score(task),
        })
        .collect();

    scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scores
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_node(id: &str, tier: NodeSecurityTier) -> EdgeNode {
        EdgeNode {
            id: EdgeNodeId(id.to_string()),
            profile_id: NodeProfileId("PROFILE-1".to_string()),
            hardware_class: HardwareClass::Router,
            resources: EdgeNodeResources {
                cpu_cores: 4,
                memory_mb: 2048,
                storage_mb: 8192,
                power_budget_w: 15.0,
            },
            energy: EnergyProfile {
                tier: EnergyTier::GridStable,
                available_fraction: 0.9,
                scarcity_flag: false,
            },
            thermal: ThermalProfile {
                band: ThermalBand::Normal,
                margin_c: 5.0,
            },
            connectivity: ConnectivityProfile {
                tier: ConnectivityTier::HighThroughputLowLatency,
                upstream_mbps: 100.0,
                downstream_mbps: 100.0,
                latency_ms: 30.0,
                availability_prob: 0.99,
            },
            security: NodeSecurityProfile {
                node_profile_id: NodeProfileId("PROFILE-1".to_string()),
                tier,
                secure_boot: true,
                signed_updates: true,
                secure_transport: true,
                last_audit_passed_at: Some(SystemTime::now()),
            },
            birth_signs: vec![BirthSignId("TILE-1".to_string())],
            tags: HashMap::new(),
        }
    }

    fn high_sensitivity_task() -> TaskInstance {
        TaskInstance {
            id: "TASK-1".to_string(),
            class: TaskClass::Governance,
            cpu_cores_required: 1,
            memory_mb_required: 512,
            duration_hint: Duration::from_secs(600),
            governance: TaskGovernanceProfile {
                domains: vec![GovernanceDomain::Water],
                required_birth_signs: vec![BirthSignId("TILE-1".to_string())],
                sensitivity: TaskSensitivity::High,
                jurisdiction_lock: true,
            },
        }
    }

    #[test]
    fn tee_backed_node_can_host_sensitive() {
        let node = dummy_node("NODE-TEE", NodeSecurityTier::TeeBacked);
        let task = high_sensitivity_task();
        assert!(node.suitability_score(&task) > 0.0);
    }

    #[test]
    fn basic_node_rejected_for_sensitive() {
        let node = dummy_node("NODE-BASIC", NodeSecurityTier::Basic);
        let task = high_sensitivity_task();
        assert_eq!(node.suitability_score(&task), 0.0);
    }
}
