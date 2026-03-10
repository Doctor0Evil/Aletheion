// Edge orchestration node capability and scheduling model for Aletheion.
// Used to place ERM and governance workloads on self-hosted consumer
// and urban electronics under energy and safety constraints.

#![forbid(unsafe_code)]

use std::time::SystemTime;

use crate::governance::birthsigns::core::{BirthSignId, NodeProfileId, NodeSecurityProfile, NodeSecurityTier};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HardwareClass {
    Router,
    BuildingController,
    Appliance,
    Vehicle,
    StreetscapeDevice,
    EdgeServer,
    Microcontroller,
}

#[derive(Debug, Clone)]
pub struct EnergyProfile {
    pub typical_power_watts: f32,
    pub max_power_watts: f32,
    pub duty_cycle_hint: f32, // 0.0–1.0 fraction of time available
}

#[derive(Debug, Clone)]
pub struct ThermalProfile {
    pub max_safe_temp_c: f32,
    pub current_temp_c: f32,
}

#[derive(Debug, Clone)]
pub struct ConnectivityProfile {
    pub upstream_mbps: f32,
    pub downstream_mbps: f32,
    pub average_latency_ms: f32,
    pub intermittent: bool,
}

#[derive(Debug, Clone)]
pub struct EdgeNode {
    pub id: NodeProfileId,
    pub hardware: HardwareClass,
    pub birth_sign_ids: Vec<BirthSignId>,
    pub energy: EnergyProfile,
    pub thermal: ThermalProfile,
    pub connectivity: ConnectivityProfile,
    pub security: NodeSecurityProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkloadSensitivity {
    GeneralErm,
    GovernanceSensitive,
    BiosignalSensitive,
}

#[derive(Debug, Clone)]
pub struct WorkloadProfile {
    pub id: String,
    pub sensitivity: WorkloadSensitivity,
    pub required_birth_sign_ids: Vec<BirthSignId>,
    pub cpu_weight: f32,
    pub memory_mb: u32,
    pub max_latency_ms: f32,
}

/// Compute a suitability score for assigning a workload to an edge node.
/// Higher scores indicate better fit; negative scores should be treated as ineligible.
pub fn suitability_score(node: &EdgeNode, workload: &WorkloadProfile, now: SystemTime) -> f32 {
    let _ = now;

    // Governance and sensitivity checks first.
    match workload.sensitivity {
        WorkloadSensitivity::GovernanceSensitive => {
            if !node.security.can_host_sensitive_governance() {
                return -1.0;
            }
        }
        WorkloadSensitivity::BiosignalSensitive => {
            if node.security.tier != NodeSecurityTier::TeeBacked {
                return -1.0;
            }
        }
        WorkloadSensitivity::GeneralErm => {
            if !node.security.can_host_general_erm() {
                return -1.0;
            }
        }
    }

    // BirthSign jurisdiction compatibility: require overlap.
    if !workload.required_birth_sign_ids.is_empty() {
        let mut overlap = false;
        for required in &workload.required_birth_sign_ids {
            if node.birth_sign_ids.contains(required) {
                overlap = true;
                break;
            }
        }
        if !overlap {
            return -1.0;
        }
    }

    // Energy and thermal heuristics.
    let energy_score = node.energy.duty_cycle_hint;
    let temp_margin =
        (node.thermal.max_safe_temp_c - node.thermal.current_temp_c).max(0.0) / 20.0;
    let thermal_score = temp_margin.min(1.0);

    // Connectivity heuristics.
    let latency_penalty = (workload.max_latency_ms / (node.connectivity.average_latency_ms + 1.0))
        .min(2.0);
    let connectivity_score = latency_penalty;

    energy_score * 0.4 + thermal_score * 0.3 + connectivity_score * 0.3
}
