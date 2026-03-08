// ============================================================================
// FILE: aletheion/erm/ecosafety/rust/types/src/lib.rs
// VERSION: ALE-ERM-ECOSAFETY-TYPES-001.rs
// LICENSE: Apache-2.0 WITH Aletheion-Ecosafety-Exception-1.0
// STATUS: Production-Ready | Offline-Capable | Post-Quantum-Secure
// ============================================================================
// PURPOSE: Foundational type definitions for Aletheion's ecosafety grammar spine.
//          All risk coordinates, seven-capital states, corridor decisions, and
//          Lyapunov residual structures are defined here. This crate is the
//          canonical reference for all ecosafety semantics across Rust, ALN,
//          Lua, JavaScript, Kotlin, and C++ bindings.
// ============================================================================
// CONSTRAINTS: 
//   - No blacklisted cryptography (SHA-256, BLAKE, KECCAK, etc.)
//   - All values normalized to [0.0, 1.0] for risk coordinates
//   - Seven capitals mandatory: water, thermal, waste, biotic, somatic, neurobiome, treaty
//   - Lyapunov residual V_t must be non-increasing for safe state transitions
//   - DID-bound identity for all augmented-citizen interactions
// ============================================================================

#![no_std]
#![cfg_attr(not(test), no_main)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{self, Display, Formatter};

// ============================================================================
// SECTION 1: RISK COORDINATE DEFINITIONS (rx)
// ============================================================================
/// Risk coordinates are the atomic units of ecosafety measurement.
/// All values MUST be normalized to [0.0, 1.0] where:
///   - 0.0 = No risk detected
///   - 1.0 = Maximum allowable threshold (corridor boundary)
///   - >1.0 = Violation (triggers derate/stop)
/// 
/// Derived from: ISO 14851 (biodegradation), OECD toxicity guidelines,
/// LCMS (Liquid Chromatography-Mass Spectrometry) data.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(C)]
pub struct RiskCoord {
    /// Normalized risk value [0.0, 1.0]
    pub value: f32,
    /// Timestamp of measurement (Unix epoch, microseconds)
    pub timestamp_us: u64,
    /// Source sensor/node DID (Decentralized Identifier)
    pub source_did: [u8; 32],
    /// Confidence interval [0.0, 1.0]
    pub confidence: f32,
}

impl RiskCoord {
    /// Create a new RiskCoord with validation
    #[inline]
    pub const fn new(value: f32, timestamp_us: u64, source_did: [u8; 32], confidence: f32) -> Result<Self, EcosafetyError> {
        if value < 0.0 || value > 1.5 {
            return Err(EcosafetyError::InvalidRiskValue);
        }
        if confidence < 0.0 || confidence > 1.0 {
            return Err(EcosafetyError::InvalidConfidence);
        }
        Ok(Self { value, timestamp_us, source_did, confidence })
    }

    /// Check if this coordinate exceeds corridor threshold
    #[inline]
    pub const fn exceeds_corridor(&self, threshold: f32) -> bool {
        self.value > threshold
    }

    /// Check if this is a violation (>1.0)
    #[inline]
    pub const fn is_violation(&self) -> bool {
        self.value > 1.0
    }
}

/// Specific risk coordinate types for biodegradable nodes and cyboquatic modules
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum RiskCoordType {
    /// Degradation rate risk (r_degrade)
    Degrade = 0,
    /// Residual mass risk (r_residual_mass)
    ResidualMass = 1,
    /// Microplastic release risk (r_microplastics)
    Microplastics = 2,
    /// Acute toxicity risk (r_tox_acute)
    ToxAcute = 3,
    /// Chronic toxicity risk (r_tox_chronic)
    ToxChronic = 4,
    /// Out-of-band operation risk (r_out_of_band)
    OutOfBand = 5,
    /// Hydraulic stress risk (r_hydraulic)
    Hydraulic = 6,
    /// Thermal stress risk (r_thermal)
    Thermal = 7,
    /// Biotic disturbance risk (r_biotic)
    Biotic = 8,
    /// Custom/extended risk coordinate
    Custom = 255,
}

// ============================================================================
// SECTION 2: SEVEN CAPITAL STATE STRUCTURE
// ============================================================================
/// The Seven Capital State represents the holistic state of any Aletheion
/// subsystem across all required capitals. This is MANDATORY for all Service
/// functions before actuation.
/// 
/// Capitals: water, thermal, waste, biotic, somatic, neurobiome, treaty
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct SevenCapitalState {
    /// Water capital: availability, quality, allocation
    pub water: CapitalState,
    /// Thermal capital: heat budgets, cooling capacity, UHI metrics
    pub thermal: CapitalState,
    /// Waste capital: material flows, recovery rates, circularity
    pub waste: CapitalState,
    /// Biotic capital: species health, habitat integrity, treaty compliance
    pub biotic: CapitalState,
    /// Somatic capital: citizen physical safety, health envelopes
    pub somatic: CapitalState,
    /// Neurobiome capital: cognitive liberty, neurorights, mental privacy
    pub neurobiome: CapitalState,
    /// Treaty capital: Indigenous rights, FPIC, BioticTreaties
    pub treaty: CapitalState,
    /// Global Lyapunov residual V_t for this state
    pub lyapunov_residual: LyapunovResidual,
    /// Timestamp of state snapshot (Unix epoch, microseconds)
    pub timestamp_us: u64,
    /// DID of the entity this state represents
    pub entity_did: [u8; 32],
}

impl SevenCapitalState {
    /// Create a new SevenCapitalState with validation
    pub fn new(
        water: CapitalState,
        thermal: CapitalState,
        waste: CapitalState,
        biotic: CapitalState,
        somatic: CapitalState,
        neurobiome: CapitalState,
        treaty: CapitalState,
        entity_did: [u8; 32],
        timestamp_us: u64,
    ) -> Result<Self, EcosafetyError> {
        let lyapunov_residual = LyapunovResidual::compute_from_capitals(
            &water, &thermal, &waste, &biotic, &somatic, &neurobiome, &treaty
        )?;
        
        Ok(Self {
            water, thermal, waste, biotic, somatic, neurobiome, treaty,
            lyapunov_residual, timestamp_us, entity_did,
        })
    }

    /// Check if all capitals are within safe corridors
    #[inline]
    pub fn all_capitals_safe(&self) -> bool {
        self.water.is_safe() &&
        self.thermal.is_safe() &&
        self.waste.is_safe() &&
        self.biotic.is_safe() &&
        self.somatic.is_safe() &&
        self.neurobiome.is_safe() &&
        self.treaty.is_safe()
    }

    /// Check if V_t is non-increasing (stable state transition)
    #[inline]
    pub fn v_t_stable(&self, previous_v_t: f32) -> bool {
        self.lyapunov_residual.v_t <= previous_v_t
    }
}

/// Individual capital state with risk coordinates and corridor thresholds
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct CapitalState {
    /// Current normalized state value [0.0, 1.0]
    pub current: f32,
    /// Minimum allowable threshold (corridor floor)
    pub min_threshold: f32,
    /// Maximum allowable threshold (corridor ceiling)
    pub max_threshold: f32,
    /// Rate of change per time unit
    pub delta: f32,
    /// Associated risk coordinates for this capital
    pub risk_coords: Vec<RiskCoord>,
    /// FPIC (Free, Prior, Informed Consent) status for treaty-related capitals
    pub fpic_verified: bool,
}

impl CapitalState {
    /// Check if this capital is within safe corridors
    #[inline]
    pub const fn is_safe(&self) -> bool {
        self.current >= self.min_threshold &&
        self.current <= self.max_threshold &&
        !self.risk_coords.iter().any(|rc| rc.is_violation())
    }

    /// Create a new CapitalState with validation
    pub fn new(
        current: f32,
        min_threshold: f32,
        max_threshold: f32,
        fpic_verified: bool,
    ) -> Result<Self, EcosafetyError> {
        if min_threshold > max_threshold {
            return Err(EcosafetyError::InvalidCorridorThresholds);
        }
        Ok(Self {
            current,
            min_threshold,
            max_threshold,
            delta: 0.0,
            risk_coords: Vec::new(),
            fpic_verified,
        })
    }
}

// ============================================================================
// SECTION 3: LYAPUNOV RESIDUAL (V_t) STRUCTURE
// ============================================================================
/// Lyapunov residual function for system-wide stability measurement.
/// V_t must be non-increasing for safe state transitions.
/// 
/// Formula: V_t = Σ(w_i * norm(r_i)) where:
///   - w_i = weight for risk coordinate i (Σw_i = 1.0)
///   - r_i = normalized risk coordinate value
///   - norm() = normalization function [0.0, 1.0]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct LyapunovResidual {
    /// Current V_t value [0.0, 1.0+]
    pub v_t: f32,
    /// Previous V_t value (for delta calculation)
    pub v_t_previous: f32,
    /// Time derivative dV_t/dt
    pub v_t_derivative: f32,
    /// Weights for each capital in V_t calculation
    pub capital_weights: [f32; 7],
    /// Timestamp of calculation (Unix epoch, microseconds)
    pub timestamp_us: u64,
}

impl LyapunovResidual {
    /// Compute V_t from seven capital states
    pub fn compute_from_capitals(
        water: &CapitalState,
        thermal: &CapitalState,
        waste: &CapitalState,
        biotic: &CapitalState,
        somatic: &CapitalState,
        neurobiome: &CapitalState,
        treaty: &CapitalState,
    ) -> Result<Self, EcosafetyError> {
        // Default weights (can be tuned per deployment context)
        // Treaty and biotic weighted higher for Indigenous/biotic sovereignty
        let weights = [0.12, 0.12, 0.12, 0.18, 0.12, 0.16, 0.18]; // Sum = 1.0
        
        let v_t = water.current * weights[0] +
                  thermal.current * weights[1] +
                  waste.current * weights[2] +
                  biotic.current * weights[3] +
                  somatic.current * weights[4] +
                  neurobiome.current * weights[5] +
                  treaty.current * weights[6];
        
        Ok(Self {
            v_t,
            v_t_previous: v_t, // Initialized to current
            v_t_derivative: 0.0,
            capital_weights: weights,
            timestamp_us: 0, // Set by caller
        })
    }

    /// Check if V_t is increasing (unstable)
    #[inline]
    pub const fn is_increasing(&self) -> bool {
        self.v_t > self.v_t_previous
    }

    /// Check if V_t derivative exceeds safe rate
    #[inline]
    pub const fn derivative_safe(&self, max_derivative: f32) -> bool {
        self.v_t_derivative.abs() <= max_derivative
    }
}

// ============================================================================
// SECTION 4: CORRIDOR DECISION ENUMS
// ============================================================================
/// Decision output from corridor validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CorridorDecision {
    /// All corridors satisfied, action permitted
    Permit = 0,
    /// Corridors satisfied but approaching threshold, proceed with caution
    Derate = 1,
    /// Corridor violation detected, action blocked
    Stop = 2,
    /// Missing corridor data, build/actuation blocked
    NoBuild = 3,
    /// FPIC/treaty verification required before decision
    PendingFPIC = 4,
    /// System error during validation
    Error = 255,
}

impl Display for CorridorDecision {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CorridorDecision::Permit => write!(f, "PERMIT"),
            CorridorDecision::Derate => write!(f, "DERATE"),
            CorridorDecision::Stop => write!(f, "STOP"),
            CorridorDecision::NoBuild => write!(f, "NO_BUILD"),
            CorridorDecision::PendingFPIC => write!(f, "PENDING_FPIC"),
            CorridorDecision::Error => write!(f, "ERROR"),
        }
    }
}

// ============================================================================
// SECTION 5: NODE ACTION TYPES
// ============================================================================
/// Allowed actions for cyboquatic, biodegradable, and infrastructure nodes
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum NodeAction {
    /// Deploy node to specified location
    Deploy = 0,
    /// Park/hold node in current position
    Park = 1,
    /// Execute actuation (pump, valve, sensor read, etc.)
    Actuate = 2,
    /// Begin retirement/decomposition sequence
    Retire = 3,
    /// Emergency stop (immediate halt)
    EmergencyStop = 4,
    /// Derate operation (reduce power/throughput)
    Derate = 5,
    /// Request FPIC verification
    RequestFPIC = 6,
    /// Log violation shard
    LogViolation = 7,
}

impl Display for NodeAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NodeAction::Deploy => write!(f, "DEPLOY"),
            NodeAction::Park => write!(f, "PARK"),
            NodeAction::Actuate => write!(f, "ACTUATE"),
            NodeAction::Retire => write!(f, "RETIRE"),
            NodeAction::EmergencyStop => write!(f, "EMERGENCY_STOP"),
            NodeAction::Derate => write!(f, "DERATE"),
            NodeAction::RequestFPIC => write!(f, "REQUEST_FPIC"),
            NodeAction::LogViolation => write!(f, "LOG_VIOLATION"),
        }
    }
}

// ============================================================================
// SECTION 6: ECOSAFETY ERROR TYPES
// ============================================================================
/// Ecosafety-specific error types for Result handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EcosafetyError {
    /// Risk coordinate value out of valid range
    InvalidRiskValue = 0,
    /// Confidence value out of [0.0, 1.0]
    InvalidConfidence = 1,
    /// Corridor thresholds invalid (min > max)
    InvalidCorridorThresholds = 2,
    /// V_t increased beyond safe derivative
    VtDerivativeExceeded = 3,
    /// Corridor violation detected
    CorridorViolation = 4,
    /// Missing required corridor data
    MissingCorridor = 5,
    /// FPIC verification failed or pending
    FPICFailed = 6,
    /// Treaty constraint violation
    TreatyViolation = 7,
    /// Neurorights constraint violation
    NeurorightsViolation = 8,
    /// DID verification failed
    DIDVerificationFailed = 9,
    /// SMART-chain validation failed
    SmartChainValidationFailed = 10,
    /// Birth-Sign check failed
    BirthSignCheckFailed = 11,
    /// System error (unspecified)
    SystemError = 255,
}

impl Display for EcosafetyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EcosafetyError::InvalidRiskValue => write!(f, "Invalid risk coordinate value"),
            EcosafetyError::InvalidConfidence => write!(f, "Invalid confidence value"),
            EcosafetyError::InvalidCorridorThresholds => write!(f, "Invalid corridor thresholds"),
            EcosafetyError::VtDerivativeExceeded => write!(f, "V_t derivative exceeded safe limit"),
            EcosafetyError::CorridorViolation => write!(f, "Corridor violation detected"),
            EcosafetyError::MissingCorridor => write!(f, "Missing required corridor data"),
            EcosafetyError::FPICFailed => write!(f, "FPIC verification failed"),
            EcosafetyError::TreatyViolation => write!(f, "Treaty constraint violation"),
            EcosafetyError::NeurorightsViolation => write!(f, "Neurorights constraint violation"),
            EcosafetyError::DIDVerificationFailed => write!(f, "DID verification failed"),
            EcosafetyError::SmartChainValidationFailed => write!(f, "SMART-chain validation failed"),
            EcosafetyError::BirthSignCheckFailed => write!(f, "Birth-Sign check failed"),
            EcosafetyError::SystemError => write!(f, "System error"),
        }
    }
}

// ============================================================================
// SECTION 7: QPUDATASHARD STRUCTURE (Audit Trail)
// ============================================================================
/// DID-signed data shard for audit, compliance, and K/E/R measurement
/// Every deployment, actuation, and state transition MUST emit a shard
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct QpuDataShard {
    /// Unique shard identifier (DID-bound)
    pub shard_id: [u8; 32],
    /// Entity DID this shard belongs to
    pub entity_did: [u8; 32],
    /// Timestamp of shard creation (Unix epoch, microseconds)
    pub timestamp_us: u64,
    /// Seven capital state at time of shard creation
    pub state: SevenCapitalState,
    /// Action taken (or blocked)
    pub action: NodeAction,
    /// Corridor decision result
    pub corridor_decision: CorridorDecision,
    /// Lyapunov residual V_t at time of action
    pub v_t: f32,
    /// K/E/R metrics for this shard
    pub ker_metrics: KERMetrics,
    /// Violation details (if any)
    pub violation: Option<ViolationRecord>,
    /// FPIC verification status
    pub fpic_status: FPICStatus,
    /// SMART-chain validation hash (post-quantum secure)
    pub smart_chain_hash: [u8; 64],
    /// Birth-Sign verification status
    pub birth_sign_verified: bool,
}

/// K/E/R (Knowledge/Eco-impact/Risk) metrics per shard
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct KERMetrics {
    /// Knowledge factor [0.0, 1.0]
    pub k: f32,
    /// Eco-impact value [0.0, 1.0]
    pub e: f32,
    /// Risk of harm [0.0, 1.0+]
    pub r: f32,
}

/// Violation record for logged incidents
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct ViolationRecord {
    /// Violation type
    pub violation_type: EcosafetyError,
    /// Risk coordinate that triggered violation
    pub triggering_coord: RiskCoord,
    /// Severity level [0.0, 1.0]
    pub severity: f32,
    /// Remediation action taken
    pub remediation: NodeAction,
    /// Timestamp of violation (Unix epoch, microseconds)
    pub timestamp_us: u64,
}

/// FPIC (Free, Prior, Informed Consent) status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FPICStatus {
    /// FPIC verified and valid
    Verified = 0,
    /// FPIC pending verification
    Pending = 1,
    /// FPIC not required for this action
    NotRequired = 2,
    /// FPIC verification failed
    Failed = 3,
}

// ============================================================================
// SECTION 8: TRAIT DEFINITIONS FOR ECOSAFETY CONTRACTS
// ============================================================================
/// Trait for types that can be validated against ecosafety corridors
pub trait CorridorValidatable {
    /// Validate against corridors, return decision
    fn validate_corridors(&self) -> Result<CorridorDecision, EcosafetyError>;
    
    /// Check if all required corridors are present
    fn has_all_corridors(&self) -> bool;
}

/// Trait for types that can compute Lyapunov residual
pub trait LyapunovComputable {
    /// Compute V_t from current state
    fn compute_v_t(&self) -> Result<LyapunovResidual, EcosafetyError>;
    
    /// Check if V_t is non-increasing vs previous state
    fn is_v_t_stable(&self, previous_v_t: f32) -> bool;
}

/// Trait for types that emit QpuDataShards
pub trait ShardEmitting {
    /// Emit a DID-signed shard for this action/state
    fn emit_shard(&self) -> Result<QpuDataShard, EcosafetyError>;
}

// ============================================================================
// SECTION 9: COMPILATION FLAGS AND FEATURE GATES
// ============================================================================
/// Feature flags for conditional compilation
/// 
/// Features:
///   - "offline-mode": Disable network-dependent operations
///   - "pq-crypto": Enable post-quantum cryptographic primitives
///   - "neurorights-enforcement": Enable strict neurorights checks
///   - "biotic-treaty-mode": Enable BioticTreaty constraints
///   - "indigenous-sovereignty": Enable Indigenous land rights protocols
#[cfg(feature = "offline-mode")]
pub const OFFLINE_MODE: bool = true;
#[cfg(not(feature = "offline-mode"))]
pub const OFFLINE_MODE: bool = false;

#[cfg(feature = "pq-crypto")]
pub const PQ_CRYPTO_ENABLED: bool = true;
#[cfg(not(feature = "pq-crypto"))]
pub const PQ_CRYPTO_ENABLED: bool = false;

#[cfg(feature = "neurorights-enforcement")]
pub const NEURORIGHTS_ENFORCEMENT: bool = true;
#[cfg(not(feature = "neurorights-enforcement"))]
pub const NEURORIGHTS_ENFORCEMENT: bool = false;

#[cfg(feature = "biotic-treaty-mode")]
pub const BIOTIC_TREATY_MODE: bool = true;
#[cfg(not(feature = "biotic-treaty-mode"))]
pub const BIOTIC_TREATY_MODE: bool = false;

#[cfg(feature = "indigenous-sovereignty")]
pub const INDIGENOUS_SOVEREIGNTY: bool = true;
#[cfg(not(feature = "indigenous-sovereignty"))]
pub const INDIGENOUS_SOVEREIGNTY: bool = false;

// ============================================================================
// SECTION 10: CONSTANTS AND CONFIGURATION
// ============================================================================
/// Maximum allowable V_t derivative per time unit
pub const MAX_VT_DERIVATIVE: f32 = 0.05;

/// Default corridor threshold for all risk coordinates
pub const DEFAULT_CORRIDOR_THRESHOLD: f32 = 1.0;

/// Derate threshold (approaching violation)
pub const DERATE_THRESHOLD: f32 = 0.85;

/// Minimum confidence for risk coordinate acceptance
pub const MIN_CONFIDENCE_THRESHOLD: f32 = 0.70;

/// K/E/R target bands for biodegradable nodes
pub const TARGET_K_BIODEGRADABLE: f32 = 0.91;
pub const TARGET_E_BIODEGRADABLE: f32 = 0.90;
pub const TARGET_R_BIODEGRADABLE: f32 = 0.15;

/// K/E/R target bands for MAR cyboquatic modules
pub const TARGET_K_MAR: f32 = 0.93;
pub const TARGET_E_MAR: f32 = 0.92;
pub const TARGET_R_MAR: f32 = 0.14;

/// K/E/R target bands for ecosafety grammar spine
pub const TARGET_K_GRAMMAR: f32 = 0.94;
pub const TARGET_E_GRAMMAR: f32 = 0.90;
pub const TARGET_R_GRAMMAR: f32 = 0.12;

// ============================================================================
// END OF FILE: ALE-ERM-ECOSAFETY-TYPES-001.rs
// ============================================================================
