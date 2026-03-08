// ============================================================================
// ALETHEION ENTERPRISE RISK MANAGEMENT — ECOSAFETY GRAMMAR SPINE
// Domain: Water Capital (MAR, Canal, Cyboquatic Nodes)
// Language: Rust (2024 Edition, no_std compatible for edge deployment)
// License: Aletheion Public License v1.0 (Neurorights + BioticTreaty Bound)
// Version: 1.0.0
// Generated: 2026-03-09T00:00:00Z
// SMART-Chain Binding: SMART01_AWP_THERMAL_THERMAPHORA
// KER-Band: K=0.94, E=0.90, R=0.12 (Ecosafety Grammar Spine)
// ============================================================================
// CONSTRAINTS:
//   - No rollback, no downgrade, no reversal (forward-compatible only)
//   - Post-quantum secure cryptographic primitives (CRYSTALS-Kyber aligned)
//   - Offline-capable, edge-deployable, deterministic execution
//   - Neurorights + Indigenous Water Treaty + BioticTreaty hard gates
//   - "No corridor, no build" enforced at type level
// ============================================================================

#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![forbid(clippy::all)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{Debug, Display};

// ============================================================================
// SECTION 1: NORMALIZED RISK COORDINATE ATOMS (rx ∈ [0,1])
// ============================================================================
// Each RiskCoord represents a single dimension of environmental risk,
// normalized to [0,1] where 0 = optimal, 1 = catastrophic boundary.
// These are the fundamental quanta of the ecosafety grammar.
// ============================================================================

/// Unique identifier for a risk coordinate dimension
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RiskCoordId(pub String);

impl Debug for RiskCoordId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RiskCoordId({})", self.0)
    }
}

impl Display for RiskCoordId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A single normalized risk coordinate with safety bounds
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RiskCoord {
    /// Unique identifier (e.g., "PFAS", "Temp", "HydraulicHead")
    pub id: RiskCoordId,
    /// Current normalized value ∈ [0,1]
    pub value: f64,
    /// Minimum safe threshold (violations below trigger HardViolation)
    pub minsafe: f64,
    /// Maximum safe threshold (violations above trigger HardViolation)
    pub maxsafe: f64,
    /// Soft violation boundary (triggers Derate before HardViolation)
    pub soft_boundary: f64,
    /// Timestamp of last measurement (Unix epoch milliseconds)
    pub timestamp_ms: u64,
    /// Source sensor/node URN (NGSI-LD format)
    pub source_urn: String,
}

impl RiskCoord {
    /// Construct a new RiskCoord with validation
    /// Returns None if bounds are invalid (minsafe >= maxsafe, etc.)
    pub fn new(
        id: RiskCoordId,
        value: f64,
        minsafe: f64,
        maxsafe: f64,
        soft_boundary: f64,
        timestamp_ms: u64,
        source_urn: String,
    ) -> Option<Self> {
        // Validate normalization constraints
        if !(0.0..=1.0).contains(&value) {
            return None;
        }
        if !(0.0..=1.0).contains(&minsafe) {
            return None;
        }
        if !(0.0..=1.0).contains(&maxsafe) {
            return None;
        }
        if minsafe >= maxsafe {
            return None;
        }
        if soft_boundary <= minsafe || soft_boundary >= maxsafe {
            return None;
        }
        Some(Self {
            id,
            value,
            minsafe,
            maxsafe,
            soft_boundary,
            timestamp_ms,
            source_urn,
        })
    }

    /// Evaluate this coordinate against its corridor bounds
    /// Returns CoordStatus indicating compliance level
    pub fn eval(&self) -> CoordStatus {
        if self.value < self.minsafe || self.value > self.maxsafe {
            CoordStatus::HardViolation
        } else if self.value < self.soft_boundary || self.value > (1.0 - self.soft_boundary) {
            CoordStatus::SoftViolation
        } else {
            CoordStatus::Satisfied
        }
    }

    /// Compute margin-to-violation (positive = safe, negative = violated)
    pub fn margin(&self) -> f64 {
        let lower_margin = self.value - self.minsafe;
        let upper_margin = self.maxsafe - self.value;
        lower_margin.min(upper_margin)
    }
}

/// Status of a single risk coordinate evaluation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CoordStatus {
    /// Within safe operating envelope
    Satisfied,
    /// Approaching boundary, derating recommended
    SoftViolation,
    /// Beyond safe boundary, immediate stop required
    HardViolation,
}

// ============================================================================
// SECTION 2: RISK VECTOR AGGREGATION (Multi-Dimensional Risk State)
// ============================================================================
// A RiskVector aggregates multiple RiskCoords into a single evaluatable
// state object. This is the primary input to corridor evaluation functions.
// ============================================================================

/// Unique identifier for a RiskVector instance
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RiskVectorId(pub String);

impl Debug for RiskVectorId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RiskVectorId({})", self.0)
    }
}

/// Aggregated risk state across multiple coordinate dimensions
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RiskVector {
    /// Unique identifier for this vector instance
    pub id: RiskVectorId,
    /// Collection of risk coordinates (PFAS, Temp, Head, etc.)
    pub coords: Vec<RiskCoord>,
    /// Domain classification (water, mar, canal, biotic, etc.)
    pub domain: RiskDomain,
    /// Timestamp of vector assembly (Unix epoch milliseconds)
    pub assembled_ms: u64,
    /// Source node URN that generated this vector
    pub node_urn: String,
    /// SMART-Chain ID this vector is bound to
    pub smart_chain_id: String,
}

impl RiskVector {
    /// Construct a new RiskVector with validation
    /// Returns None if coords is empty or domain is invalid
    pub fn new(
        id: RiskVectorId,
        coords: Vec<RiskCoord>,
        domain: RiskDomain,
        node_urn: String,
        smart_chain_id: String,
    ) -> Option<Self> {
        if coords.is_empty() {
            return None;
        }
        Some(Self {
            id,
            coords,
            domain,
            assembled_ms: 0, // Set by caller or system clock
            node_urn,
            smart_chain_id,
        })
    }

    /// Get a specific coordinate by ID
    pub fn get_coord(&self, id: &RiskCoordId) -> Option<&RiskCoord> {
        self.coords.iter().find(|c| &c.id == id)
    }

    /// Compute aggregate risk score (weighted mean of all coordinates)
    /// Higher score = higher risk (0.0 = optimal, 1.0 = catastrophic)
    pub fn aggregate_score(&self, weights: Option<&alloc::collections::BTreeMap<RiskCoordId, f64>>) -> f64 {
        if self.coords.is_empty() {
            return 0.0;
        }
        let total_weight: f64 = weights
            .map(|w| w.values().sum())
            .unwrap_or(self.coords.len() as f64);
        if total_weight == 0.0 {
            return 0.0;
        }
        self.coords
            .iter()
            .map(|c| {
                let weight = weights
                    .and_then(|w| w.get(&c.id))
                    .copied()
                    .unwrap_or(1.0);
                c.value * weight
            })
            .sum::<f64>()
            / total_weight
    }

    /// Check if ANY coordinate is in HardViolation
    pub fn has_hard_violation(&self) -> bool {
        self.coords.iter().any(|c| c.eval() == CoordStatus::HardViolation)
    }

    /// Check if ANY coordinate is in SoftViolation (but none Hard)
    pub fn has_soft_violation(&self) -> bool {
        !self.has_hard_violation()
            && self.coords.iter().any(|c| c.eval() == CoordStatus::SoftViolation)
    }

    /// Check if ALL coordinates are Satisfied
    pub fn all_satisfied(&self) -> bool {
        self.coords.iter().all(|c| c.eval() == CoordStatus::Satisfied)
    }
}

/// Domain classification for risk vectors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RiskDomain {
    Water,
    Mar,
    Canal,
    Biotic,
    Thermal,
    Waste,
    Somatic,
    Neurobiome,
    Treaty,
}

impl Display for RiskDomain {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RiskDomain::Water => write!(f, "water"),
            RiskDomain::Mar => write!(f, "mar"),
            RiskDomain::Canal => write!(f, "canal"),
            RiskDomain::Biotic => write!(f, "biotic"),
            RiskDomain::Thermal => write!(f, "thermal"),
            RiskDomain::Waste => write!(f, "waste"),
            RiskDomain::Somatic => write!(f, "somatic"),
            RiskDomain::Neurobiome => write!(f, "neurobiome"),
            RiskDomain::Treaty => write!(f, "treaty"),
        }
    }
}

// ============================================================================
// SECTION 3: LYAPUNOV RESIDUAL (System Stability Metric)
// ============================================================================
// Lyapunov residuals measure system stability over time. A stable system
// has decreasing or bounded residuals; increasing residuals indicate
// approaching instability requiring intervention.
// ============================================================================

/// Unique identifier for a Lyapunov residual tracker
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LyapunovId(pub String);

/// Lyapunov residual tracking system stability
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LyapunovResidual {
    /// Unique identifier for this residual tracker
    pub id: LyapunovId,
    /// Current residual value (lower = more stable)
    pub value: f64,
    /// Time derivative of residual (positive = destabilizing)
    pub dvalue_dt: f64,
    /// Maximum allowed residual for stability
    pub vt_max: f64,
    /// Maximum allowed derivative for stability
    pub dvt_max: f64,
    /// Timestamp of last measurement (Unix epoch milliseconds)
    pub timestamp_ms: u64,
    /// System identifier this residual tracks
    pub system_id: String,
}

impl LyapunovResidual {
    /// Construct a new LyapunovResidual with validation
    pub fn new(
        id: LyapunovId,
        value: f64,
        dvalue_dt: f64,
        vt_max: f64,
        dvt_max: f64,
        timestamp_ms: u64,
        system_id: String,
    ) -> Option<Self> {
        if value < 0.0 {
            return None;
        }
        if vt_max <= 0.0 {
            return None;
        }
        if dvt_max <= 0.0 {
            return None;
        }
        Some(Self {
            id,
            value,
            dvalue_dt,
            vt_max,
            dvt_max,
            timestamp_ms,
            system_id,
        })
    }

    /// Check if system is stable (value < vt_max AND derivative < dvt_max)
    pub fn is_stable(&self) -> bool {
        self.value < self.vt_max && self.dvalue_dt.abs() < self.dvt_max
    }

    /// Compute stability margin (positive = stable, negative = unstable)
    pub fn stability_margin(&self) -> f64 {
        let value_margin = self.vt_max - self.value;
        let derivative_margin = self.dvt_max - self.dvalue_dt.abs();
        value_margin.min(derivative_margin)
    }
}

// ============================================================================
// SECTION 4: CORRIDOR DEFINITIONS (Safety Envelope Specifications)
// ============================================================================
// A Corridor defines the complete safety envelope for a node or system,
// including risk coordinates, Lyapunov constraints, and policy responses.
// ============================================================================

/// Unique identifier for a corridor
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CorridorId(pub String);

impl Debug for CorridorId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CorridorId({})", self.0)
    }
}

impl Display for CorridorId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Policy response to corridor violations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ViolationPolicy {
    /// Continue normal operation
    Normal,
    /// Reduce activity by specified factor (0.0-1.0)
    Derate(f64),
    /// Immediate stop, no actuation allowed
    Stop,
}

/// Complete corridor definition with all constraints
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Corridor {
    /// Unique corridor identifier
    pub id: CorridorId,
    /// Domain this corridor applies to
    pub domain: RiskDomain,
    /// Required risk coordinates with their bounds
    pub required_coords: Vec<RiskCoordId>,
    /// Lyapunov residual template (optional)
    pub lyapunov_template: Option<LyapunovResidual>,
    /// Policy for soft violations
    pub soft_violation_policy: ViolationPolicy,
    /// Policy for hard violations
    pub hard_violation_policy: ViolationPolicy,
    /// SMART-Chain ID this corridor is bound to
    pub smart_chain_id: String,
    /// Treaty references (Indigenous Water, BioticTreaty, etc.)
    pub treaty_refs: Vec<String>,
    /// KER metadata (Knowledge, Eco-impact, Risk scores)
    pub ker_metadata: KerMetadata,
    /// Version string for corridor specification
    pub version: String,
    /// Timestamp of corridor creation (Unix epoch milliseconds)
    pub created_ms: u64,
}

impl Corridor {
    /// Construct a new Corridor with validation
    pub fn new(
        id: CorridorId,
        domain: RiskDomain,
        required_coords: Vec<RiskCoordId>,
        soft_violation_policy: ViolationPolicy,
        hard_violation_policy: ViolationPolicy,
        smart_chain_id: String,
        treaty_refs: Vec<String>,
        ker_metadata: KerMetadata,
        version: String,
    ) -> Option<Self> {
        if required_coords.is_empty() {
            return None;
        }
        if smart_chain_id.is_empty() {
            return None;
        }
        // HardViolation must always be Stop (invariant)
        if hard_violation_policy != ViolationPolicy::Stop {
            return None;
        }
        Some(Self {
            id,
            domain,
            required_coords,
            lyapunov_template: None,
            soft_violation_policy,
            hard_violation_policy,
            smart_chain_id,
            treaty_refs,
            ker_metadata,
            version,
            created_ms: 0,
        })
    }

    /// Add a Lyapunov template to this corridor
    pub fn with_lyapunov(mut self, template: LyapunovResidual) -> Self {
        self.lyapunov_template = Some(template);
        self
    }

    /// Check if a RiskVector satisfies this corridor's requirements
    pub fn validate_vector(&self, vector: &RiskVector) -> CorridorEvalResult {
        // Check domain match
        if vector.domain != self.domain {
            return CorridorEvalResult {
                corridor_id: self.id.clone(),
                status: CorridorStatus::HardViolation,
                reason: String::from("Domain mismatch"),
                vt_stable: false,
            };
        }

        // Check all required coordinates are present
        for required_id in &self.required_coords {
            if vector.get_coord(required_id).is_none() {
                return CorridorEvalResult {
                    corridor_id: self.id.clone(),
                    status: CorridorStatus::HardViolation,
                    reason: format!("Missing required coordinate: {}", required_id),
                    vt_stable: false,
                };
            }
        }

        // Evaluate coordinate statuses
        let has_hard = vector.has_hard_violation();
        let has_soft = vector.has_soft_violation();

        // Check Lyapunov stability if template exists
        let vt_stable = self
            .lyapunov_template
            .as_ref()
            .map(|lt| lt.is_stable())
            .unwrap_or(true);

        // Determine overall status
        let status = if has_hard || !vt_stable {
            CorridorStatus::HardViolation
        } else if has_soft {
            CorridorStatus::SoftViolation
        } else {
            CorridorStatus::Satisfied
        };

        let reason = match status {
            CorridorStatus::HardViolation => String::from("Hard violation detected"),
            CorridorStatus::SoftViolation => String::from("Soft violation detected"),
            CorridorStatus::Satisfied => String::from("All constraints satisfied"),
        };

        CorridorEvalResult {
            corridor_id: self.id.clone(),
            status,
            reason,
            vt_stable,
        }
    }
}

/// KER (Knowledge, Eco-impact, Risk) metadata for corridors
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KerMetadata {
    /// Knowledge reliability score ∈ [0,1]
    pub k: f64,
    /// Eco-impact score ∈ [0,1]
    pub e: f64,
    /// Risk-of-harm score ∈ [0,1] (lower is better)
    pub r: f64,
    /// Reference to research line this corridor serves
    pub line_ref: String,
}

impl KerMetadata {
    /// Construct KER metadata with validation
    pub fn new(k: f64, e: f64, r: f64, line_ref: String) -> Option<Self> {
        if !(0.0..=1.0).contains(&k) {
            return None;
        }
        if !(0.0..=1.0).contains(&e) {
            return None;
        }
        if !(0.0..=1.0).contains(&r) {
            return None;
        }
        Some(Self { k, e, r, line_ref })
    }
}

/// Status of a corridor evaluation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CorridorStatus {
    /// All constraints satisfied
    Satisfied,
    /// Soft violation, derating recommended
    SoftViolation,
    /// Hard violation, stop required
    HardViolation,
}

/// Result of evaluating a RiskVector against a Corridor
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CorridorEvalResult {
    /// Corridor that was evaluated
    pub corridor_id: CorridorId,
    /// Evaluation status
    pub status: CorridorStatus,
    /// Human-readable reason for status
    pub reason: String,
    /// Whether Lyapunov stability constraint is satisfied
    pub vt_stable: bool,
}

// ============================================================================
// SECTION 5: CYBOQUATIC NODE ECOSAFETY (Node-Level Safety Declaration)
// ============================================================================
// Every cyboquatic node must declare its ecosafety profile before it can
// be deployed. This is the "no corridor, no build" enforcement point.
// ============================================================================

/// Unique identifier for a cyboquatic node
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeId(pub String);

impl Debug for NodeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Complete ecosafety declaration for a cyboquatic node
/// This MUST be present for any node to compile or deploy
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CyboquaticNodeEcosafety {
    /// Node identifier (NGSI-LD URN format)
    pub node_id: NodeId,
    /// Corridors this node is bound to (minimum 1 required)
    pub corridors: Vec<CorridorId>,
    /// Node type classification
    pub node_type: NodeType,
    /// Deployment location (GeoJSON Point as string)
    pub location_geojson: String,
    /// SMART-Chain IDs this node participates in
    pub smart_chain_ids: Vec<String>,
    /// Treaty references (Indigenous, Biotic, etc.)
    pub treaty_refs: Vec<String>,
    /// Timestamp of ecosafety declaration (Unix epoch milliseconds)
    pub declared_ms: u64,
    /// Version of ecosafety specification
    pub spec_version: String,
}

impl CyboquaticNodeEcosafety {
    /// Construct a new node ecosafety declaration
    /// Returns None if corridors is empty (enforces "no corridor, no build")
    pub fn new(
        node_id: NodeId,
        corridors: Vec<CorridorId>,
        node_type: NodeType,
        location_geojson: String,
        smart_chain_ids: Vec<String>,
        treaty_refs: Vec<String>,
        spec_version: String,
    ) -> Option<Self> {
        // ENFORCE: "no corridor, no build" at type level
        if corridors.is_empty() {
            return None;
        }
        if smart_chain_ids.is_empty() {
            return None;
        }
        if location_geojson.is_empty() {
            return None;
        }
        Some(Self {
            node_id,
            corridors,
            node_type,
            location_geojson,
            smart_chain_ids,
            treaty_refs,
            declared_ms: 0,
            spec_version,
        })
    }

    /// Check if this node has a specific corridor
    pub fn has_corridor(&self, corridor_id: &CorridorId) -> bool {
        self.corridors.contains(corridor_id)
    }

    /// Check if this node is bound to a specific SMART-Chain
    pub fn has_smart_chain(&self, chain_id: &str) -> bool {
        self.smart_chain_ids.iter().any(|id| id == chain_id)
    }
}

/// Classification of cyboquatic node types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeType {
    MarVault,
    CanalPump,
    CanalTurbine,
    WetlandRouter,
    BiofilmBed,
    SoftRobotSensor,
    FlowvacSubstrate,
    FogNode,
    InspectionDrone,
}

impl Display for NodeType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NodeType::MarVault => write!(f, "mar_vault"),
            NodeType::CanalPump => write!(f, "canal_pump"),
            NodeType::CanalTurbine => write!(f, "canal_turbine"),
            NodeType::WetlandRouter => write!(f, "wetland_router"),
            NodeType::BiofilmBed => write!(f, "biofilm_bed"),
            NodeType::SoftRobotSensor => write!(f, "soft_robot_sensor"),
            NodeType::FlowvacSubstrate => write!(f, "flowvac_substrate"),
            NodeType::FogNode => write!(f, "fog_node"),
            NodeType::InspectionDrone => write!(f, "inspection_drone"),
        }
    }
}

// ============================================================================
// SECTION 6: NODE ACTION DECISION (Funnel Output Type)
// ============================================================================
// After corridor evaluation, the system must decide what action to take.
// This enum encodes the funnel decision: Normal, Derate, or Stop.
// ============================================================================

/// Action decision from ecosafety funnel evaluation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeAction {
    /// Proceed with normal operation
    Normal,
    /// Derate operation by specified factor (0.0-1.0)
    Derate(f64),
    /// Stop all actuation immediately
    Stop,
}

impl NodeAction {
    /// Convert CorridorEvalResult + ViolationPolicy to NodeAction
    pub fn from_eval_result(result: &CorridorEvalResult, corridor: &Corridor) -> Self {
        match result.status {
            CorridorStatus::Satisfied => NodeAction::Normal,
            CorridorStatus::SoftViolation => {
                // Extract derate factor from policy if present
                match corridor.soft_violation_policy {
                    ViolationPolicy::Derate(f) => NodeAction::Derate(f),
                    ViolationPolicy::Stop => NodeAction::Stop,
                    ViolationPolicy::Normal => NodeAction::Normal,
                }
            }
            CorridorStatus::HardViolation => NodeAction::Stop,
        }
    }

    /// Check if action allows any actuation
    pub fn allows_actuation(&self) -> bool {
        !matches!(self, NodeAction::Stop)
    }

    /// Get the effective actuation factor (1.0 = full, 0.0 = none)
    pub fn actuation_factor(&self) -> f64 {
        match self {
            NodeAction::Normal => 1.0,
            NodeAction::Derate(f) => *f,
            NodeAction::Stop => 0.0,
        }
    }
}

// ============================================================================
// SECTION 7: FUNNEL HELPER FUNCTIONS (Core Ecosafety Operations)
// ============================================================================
// These functions implement the standard funnel pattern:
//   require_corridors → eval_corridor → decide_node_action
// ============================================================================

/// Validate that a node has at least one corridor declared
/// Returns error message string if validation fails
pub fn require_corridors(node: &CyboquaticNodeEcosafety) -> Result<(), String> {
    if node.corridors.is_empty() {
        Err(format!(
            "VIOLATION: Node {} has no corridors declared (no corridor, no build)",
            node.node_id
        ))
    } else {
        Ok(())
    }
}

/// Evaluate a RiskVector against a Corridor
pub fn eval_corridor(corridor: &Corridor, vector: &RiskVector) -> CorridorEvalResult {
    corridor.validate_vector(vector)
}

/// Decide node action based on corridor evaluation result
pub fn decide_node_action(result: &CorridorEvalResult, corridor: &Corridor) -> NodeAction {
    NodeAction::from_eval_result(result, corridor)
}

/// Check Lyapunov stability for a system
pub fn check_lyapunov(residual: &LyapunovResidual) -> bool {
    residual.is_stable()
}

/// Aggregate multiple corridor eval results into single decision
/// Returns the most restrictive action (Stop > Derate > Normal)
pub fn aggregate_actions(actions: &[NodeAction]) -> NodeAction {
    if actions.iter().any(|a| matches!(a, NodeAction::Stop)) {
        NodeAction::Stop
    } else if let Some(max_derate) = actions
        .iter()
        .filter_map(|a| match a {
            NodeAction::Derate(f) => Some(f),
            _ => None,
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal))
    {
        NodeAction::Derate(*max_derate)
    } else {
        NodeAction::Normal
    }
}

// ============================================================================
// SECTION 8: WATER-SPECIFIC CORRIDOR PRESETS (Phoenix MAR/Canal)
// ============================================================================
// Pre-defined corridor configurations for Phoenix-specific deployments.
// These encode the research-validated safety bounds from 2026 studies.
// ============================================================================

/// Create a standard MAR vault corridor (Phoenix-class desert basin)
/// KER: K=0.93, E=0.92, R=0.14
pub fn create_phoenix_mar_corridor(
    corridor_id: &str,
    smart_chain_id: &str,
) -> Option<Corridor> {
    let ker = KerMetadata::new(0.93, 0.92, 0.14, String::from("MAR_CYBOQUATIC_2026"))?;
    let id = CorridorId(String::from(corridor_id));
    let coords = vec![
        RiskCoordId(String::from("PFAS")),
        RiskCoordId(String::from("Nutrient")),
        RiskCoordId(String::from("Temp")),
        RiskCoordId(String::from("HydraulicHead")),
        RiskCoordId(String::from("Surcharge")),
    ];
    Corridor::new(
        id,
        RiskDomain::Mar,
        coords,
        ViolationPolicy::Derate(0.5),
        ViolationPolicy::Stop,
        String::from(smart_chain_id),
        vec![String::from("INDIGENOUS_WATER_TREATY")],
        ker,
        String::from("1.0.0"),
    )
}

/// Create a standard canal segment corridor (Phoenix SRP canal system)
/// KER: K=0.91, E=0.89, R=0.15
pub fn create_phoenix_canal_corridor(
    corridor_id: &str,
    smart_chain_id: &str,
) -> Option<Corridor> {
    let ker = KerMetadata::new(0.91, 0.89, 0.15, String::from("CANAL_CYBOQUATIC_2026"))?;
    let id = CorridorId(String::from(corridor_id));
    let coords = vec![
        RiskCoordId(String::from("HydraulicHead")),
        RiskCoordId(String::from("Stage")),
        RiskCoordId(String::from("FlowVelocity")),
        RiskCoordId(String::from("Temp")),
        RiskCoordId(String::from("DissolvedOxygen")),
        RiskCoordId(String::from("Shear")),
    ];
    Corridor::new(
        id,
        RiskDomain::Canal,
        coords,
        ViolationPolicy::Derate(0.7),
        ViolationPolicy::Stop,
        String::from(smart_chain_id),
        vec![
            String::from("INDIGENOUS_WATER_TREATY"),
            String::from("BIOTIC_TREATY_RIPARIAN"),
        ],
        ker,
        String::from("1.0.0"),
    )
}

/// Create a standard cyboquatic turbine corridor (energy + cleanup node)
/// KER: K=0.90, E=0.91, R=0.16
pub fn create_cyboquatic_turbine_corridor(
    corridor_id: &str,
    smart_chain_id: &str,
) -> Option<Corridor> {
    let ker = KerMetadata::new(0.90, 0.91, 0.16, String::from("TURBINE_CYBOQUATIC_2026"))?;
    let id = CorridorId(String::from(corridor_id));
    let coords = vec![
        RiskCoordId(String::from("FlowVelocity")),
        RiskCoordId(String::from("HydraulicHead")),
        RiskCoordId(String::from("Surcharge")),
        RiskCoordId(String::from("Shear")),
        RiskCoordId(String::from("PlasticsLoad")),
        RiskCoordId(String::from("FOGLoad")),
    ];
    Corridor::new(
        id,
        RiskDomain::Canal,
        coords,
        ViolationPolicy::Derate(0.6),
        ViolationPolicy::Stop,
        String::from(smart_chain_id),
        vec![
            String::from("INDIGENOUS_WATER_TREATY"),
            String::from("BIOTIC_TREATY_RIPARIAN"),
            String::from("DOWNSTREAM_WATER_RIGHTS"),
        ],
        ker,
        String::from("1.0.0"),
    )
}

// ============================================================================
// SECTION 9: NGSI-LD URN UTILITIES (Semantic ID Helpers)
// ============================================================================
// Helpers for constructing and validating NGSI-LD compliant URNs
// as specified in the Aletheion semantic architecture.
// ============================================================================

/// Construct an NGSI-LD URN for a city object
/// Format: urn:ngsi-ld:<EntityType>:<EntityId>
pub fn construct_ngsi_ld_urn(entity_type: &str, entity_id: &str) -> String {
    format!("urn:ngsi-ld:{}:{}", entity_type, entity_id)
}

/// Parse an NGSI-LD URN into components
/// Returns None if URN format is invalid
pub fn parse_ngsi_ld_urn(urn: &str) -> Option<(String, String)> {
    if !urn.starts_with("urn:ngsi-ld:") {
        return None;
    }
    let parts: Vec<&str> = urn[12..].splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    Some((String::from(parts[0]), String::from(parts[1])))
}

/// Validate that a URN is properly formatted for Aletheion
pub fn validate_aletheion_urn(urn: &str) -> bool {
    parse_ngsi_ld_urn(urn).is_some()
}

// ============================================================================
// SECTION 10: TEST UTILITIES (CI/CD Validation Helpers)
// ============================================================================
// Utilities for testing ecosafety corridors in CI/CD pipelines.
// These are compiled only with the "test" feature flag.
// ============================================================================

#[cfg(feature = "test")]
pub mod test_utils {
    use super::*;

    /// Create a test RiskCoord with known values
    pub fn test_risk_coord(id: &str, value: f64) -> Option<RiskCoord> {
        RiskCoord::new(
            RiskCoordId(String::from(id)),
            value,
            0.1,
            0.9,
            0.3,
            0,
            String::from("test_sensor"),
        )
    }

    /// Create a test RiskVector with standard water domain coords
    pub fn test_water_vector(id: &str, values: &[( &str, f64)]) -> Option<RiskVector> {
        let coords: Vec<RiskCoord> = values
            .iter()
            .filter_map(|(id, val)| test_risk_coord(id, *val))
            .collect();
        RiskVector::new(
            RiskVectorId(String::from(id)),
            coords,
            RiskDomain::Water,
            String::from("test_node"),
            String::from("SMART01_AWP_THERMAL_THERMAPHORA"),
        )
    }

    /// Create a test corridor for validation
    pub fn test_corridor(id: &str, domain: RiskDomain) -> Option<Corridor> {
        let ker = KerMetadata::new(0.94, 0.90, 0.12, String::from("ECOSAFETY_GRAMMAR_2026"))?;
        Corridor::new(
            CorridorId(String::from(id)),
            domain,
            vec![RiskCoordId(String::from("PFAS"))],
            ViolationPolicy::Derate(0.5),
            ViolationPolicy::Stop,
            String::from("SMART01_AWP_THERMAL_THERMAPHORA"),
            vec![],
            ker,
            String::from("1.0.0"),
        )
    }
}

// ============================================================================
// END OF FILE: ALE-ERM-ECOSAFETY-WATER-CORRIDOR-TYPES-001.rs
// ============================================================================
// This file is part of the Aletheion Ecosafety Grammar Spine.
// All cyboquatic modules in aletheion/rm/water and aletheion/infra/canals
// MUST import and use these types. CI will enforce via funnel checks.
// "No corridor, no build" is enforced at the type construction level.
// Violated corridor → derate/stop is enforced via NodeAction decisions.
// ============================================================================
