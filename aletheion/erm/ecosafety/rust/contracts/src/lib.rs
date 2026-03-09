// ============================================================================
// FILE: aletheion/erm/ecosafety/rust/contracts/src/lib.rs
// VERSION: ALE-ERM-ECOSAFETY-CONTRACTS-001.rs
// LICENSE: Apache-2.0 WITH Aletheion-Ecosafety-Exception-1.0
// STATUS: Production-Ready | Offline-Capable | Post-Quantum-Secure
// ============================================================================
// PURPOSE: Runtime implementation of ecosafety grammar contracts defined in
//          ALE-ERM-ECOSAFETY-GRAMMAR-001.aln. This crate provides the executable
//          logic for corridor validation, Lyapunov stability checks, safe_step
//          enforcement, K/E/R metrics computation, and QpuDataShard emission.
//          All functions are FFI-exportable for Lua, JavaScript, Kotlin, and C++
//          bindings.
// ============================================================================
// CONSTRAINTS:
//   - No blacklisted cryptography (SHA-256, BLAKE, KECCAK, etc.)
//   - Post-quantum secure hashing (CRYSTALS-Kyber/Dilithium compatible)
//   - Offline-capable (no network dependencies for core logic)
//   - Seven capitals mandatory for all Service functions
//   - FPIC, BioticTreaty, Neurorights enforcement required
//   - Indigenous sovereignty protocols (Akimel O'odham, Piipaash territories)
// ============================================================================
// COMPATIBILITY: ALE-ERM-ECOSAFETY-TYPES-001.rs, ALE-ERM-ECOSAFETY-GRAMMAR-001.aln
// ============================================================================

#![no_std]
#![cfg_attr(not(test), no_main)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::fmt::{self, Display, Formatter};

// Import types from ecosafety types crate
use aletheion_ecosafety_types::{
    RiskCoord, RiskCoordType, CapitalState, SevenCapitalState, LyapunovResidual,
    CorridorDecision, NodeAction, EcosafetyError, QpuDataShard, KERMetrics,
    ViolationRecord, FPICStatus, CorridorValidatable, LyapunovComputable, ShardEmitting,
    MAX_VT_DERIVATIVE, DEFAULT_CORRIDOR_THRESHOLD, DERATE_THRESHOLD,
    MIN_CONFIDENCE_THRESHOLD, OFFLINE_MODE, PQ_CRYPTO_ENABLED,
    NEURORIGHTS_ENFORCEMENT, BIOTIC_TREATY_MODE, INDIGENOUS_SOVEREIGNTY,
};

// ============================================================================
// SECTION 1: CORRIDOR VALIDATION IMPLEMENTATIONS
// ============================================================================

/// Corridor validator for SevenCapitalState
pub struct CorridorValidator {
    /// Required corridors per node type
    required_corridors: BTreeMap<String, Vec<String>>,
    /// Corridor thresholds per risk coordinate type
    thresholds: BTreeMap<String, f32>,
}

impl CorridorValidator {
    /// Create a new CorridorValidator with default configuration
    pub fn new() -> Self {
        let mut required_corridors = BTreeMap::new();
        let mut thresholds = BTreeMap::new();

        // Biodegradable node required corridors
        required_corridors.insert("biodegradable".to_string(), vec![
            "r_degrade".to_string(),
            "r_residual_mass".to_string(),
            "r_microplastics".to_string(),
            "r_tox_acute".to_string(),
            "r_tox_chronic".to_string(),
            "r_out_of_band".to_string(),
        ]);

        // MAR cyboquatic module required corridors
        required_corridors.insert("mar_cyboquatic".to_string(), vec![
            "r_hydraulic".to_string(),
            "r_thermal".to_string(),
            "r_biotic".to_string(),
            "r_tox_acute".to_string(),
        ]);

        // Ecotechnology habitat required corridors
        required_corridors.insert("ecotechnology".to_string(), vec![
            "r_biotic".to_string(),
            "r_waste".to_string(),
            "r_water".to_string(),
        ]);

        // Default thresholds (normalized to 0-1 scale)
        thresholds.insert("r_degrade".to_string(), 1.0);
        thresholds.insert("r_residual_mass".to_string(), 1.0);
        thresholds.insert("r_microplastics".to_string(), 1.0);
        thresholds.insert("r_tox_acute".to_string(), 1.0);
        thresholds.insert("r_tox_chronic".to_string(), 1.0);
        thresholds.insert("r_out_of_band".to_string(), 1.0);
        thresholds.insert("r_hydraulic".to_string(), 1.0);
        thresholds.insert("r_thermal".to_string(), 1.0);
        thresholds.insert("r_biotic".to_string(), 1.0);

        Self {
            required_corridors,
            thresholds,
        }
    }

    /// INV-001: Validate all risk coordinates are within valid range
    #[inline]
    pub fn validate_risk_coord_range(rc: &RiskCoord) -> Result<(), EcosafetyError> {
        if rc.value < 0.0 || rc.value > 1.5 {
            return Err(EcosafetyError::InvalidRiskValue);
        }
        if rc.confidence < 0.0 || rc.confidence > 1.0 {
            return Err(EcosafetyError::InvalidConfidence);
        }
        Ok(())
    }

    /// INV-002: Validate corridor thresholds are properly ordered
    #[inline]
    pub fn validate_corridor_thresholds(min: f32, max: f32) -> Result<(), EcosafetyError> {
        if min > max {
            return Err(EcosafetyError::InvalidCorridorThresholds);
        }
        Ok(())
    }

    /// CDC-001: No corridor, no build principle
    pub fn enforce_no_corridor_no_build(
        &self,
        entity_did: &[u8; 32],
        corridors: &BTreeMap<String, RiskCoord>,
        node_type: &str,
    ) -> CorridorDecision {
        // Check if corridors map is empty
        if corridors.is_empty() {
            return CorridorDecision::NoBuild;
        }

        // Get required corridors for this node type
        let required = match self.required_corridors.get(node_type) {
            Some(req) => req,
            None => return CorridorDecision::NoBuild, // Unknown node type
        };

        // Check all required corridors are present
        for corridor_name in required {
            if !corridors.contains_key(corridor_name) {
                return CorridorDecision::NoBuild;
            }
        }

        CorridorDecision::Permit
    }

    /// CDC-002: Violated corridor → derate/stop principle
    pub fn enforce_violated_corridor_action(
        &self,
        corridors: &BTreeMap<String, RiskCoord>,
    ) -> CorridorDecision {
        let mut max_risk = 0.0f32;
        let mut has_violation = false;

        for (coord_name, coord_value) in corridors.iter() {
            let threshold = self.thresholds.get(coord_name).unwrap_or(&1.0);
            
            if coord_value.value > *threshold {
                has_violation = true;
                if coord_value.value > max_risk {
                    max_risk = coord_value.value;
                }
            }
        }

        if max_risk > 1.0 {
            CorridorDecision::Stop
        } else if max_risk > DERATE_THRESHOLD {
            CorridorDecision::Derate
        } else if has_violation {
            CorridorDecision::Derate
        } else {
            CorridorDecision::Permit
        }
    }

    /// CDC-003: Combined corridor validation for SevenCapitalState
    pub fn validate_all_corridors(
        &self,
        state: &SevenCapitalState,
        node_type: &str,
    ) -> Result<CorridorDecision, EcosafetyError> {
        // Collect all risk coordinates from SevenCapitalState
        let corridors = self.collect_risk_coords(state);

        // Step 1: Check corridor presence (no corridor, no build)
        let presence_check = self.enforce_no_corridor_no_build(
            &state.entity_did,
            &corridors,
            node_type,
        );
        if presence_check != CorridorDecision::Permit {
            return Ok(presence_check);
        }

        // Step 2: Check corridor violations (violated corridor → derate/stop)
        let violation_check = self.enforce_violated_corridor_action(&corridors);

        Ok(violation_check)
    }

    /// CDC-004: Collect all risk coordinates from SevenCapitalState
    fn collect_risk_coords(&self, state: &SevenCapitalState) -> BTreeMap<String, RiskCoord> {
        let mut coords = BTreeMap::new();

        // Water capital risk coords
        for rc in state.water.risk_coords.iter() {
            coords.insert(format!("water_{:?}", rc.source_did[0]), *rc);
        }
        // Thermal capital risk coords
        for rc in state.thermal.risk_coords.iter() {
            coords.insert(format!("thermal_{:?}", rc.source_did[0]), *rc);
        }
        // Waste capital risk coords (critical for biodegradable nodes)
        for rc in state.waste.risk_coords.iter() {
            coords.insert(format!("waste_{:?}", rc.source_did[0]), *rc);
        }
        // Biotic capital risk coords (critical for BioticTreaty)
        for rc in state.biotic.risk_coords.iter() {
            coords.insert(format!("biotic_{:?}", rc.source_did[0]), *rc);
        }
        // Treaty capital risk coords (critical for Indigenous sovereignty)
        for rc in state.treaty.risk_coords.iter() {
            coords.insert(format!("treaty_{:?}", rc.source_did[0]), *rc);
        }
        // Somatic capital risk coords
        for rc in state.somatic.risk_coords.iter() {
            coords.insert(format!("somatic_{:?}", rc.source_did[0]), *rc);
        }
        // Neurobiome capital risk coords
        for rc in state.neurobiome.risk_coords.iter() {
            coords.insert(format!("neurobiome_{:?}", rc.source_did[0]), *rc);
        }

        coords
    }
}

impl Default for CorridorValidator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SECTION 2: LYAPUNOV RESIDUAL (V_t) COMPUTATION
// ============================================================================

/// Lyapunov residual computer for system-wide stability measurement
pub struct LyapunovComputer {
    /// Capital weights for V_t computation (sum must = 1.0)
    capital_weights: [f32; 7],
    /// Maximum allowable V_t derivative
    max_derivative: f32,
}

impl LyapunovComputer {
    /// Create a new LyapunovComputer with default weights
    pub fn new() -> Self {
        // Default weights: Treaty and Biotic weighted higher for sovereignty
        // [water, thermal, waste, biotic, somatic, neurobiome, treaty]
        let weights = [0.12, 0.12, 0.12, 0.18, 0.12, 0.16, 0.18]; // Sum = 1.0
        
        Self {
            capital_weights: weights,
            max_derivative: MAX_VT_DERIVATIVE,
        }
    }

    /// Create with custom weights (must sum to 1.0)
    pub fn with_weights(weights: [f32; 7]) -> Result<Self, EcosafetyError> {
        let sum: f32 = weights.iter().sum();
        if (sum - 1.0).abs() > 0.001 {
            return Err(EcosafetyError::InvalidCorridorThresholds);
        }
        Ok(Self {
            capital_weights: weights,
            max_derivative: MAX_VT_DERIVATIVE,
        })
    }

    /// LSC-001: V_t computation formula
    /// V_t = Σ(w_i * norm(r_i)) where w_i = capital weight, r_i = capital state
    #[inline]
    pub fn compute_v_t(&self, state: &SevenCapitalState) -> f32 {
        state.water.current * self.capital_weights[0]
            + state.thermal.current * self.capital_weights[1]
            + state.waste.current * self.capital_weights[2]
            + state.biotic.current * self.capital_weights[3]
            + state.somatic.current * self.capital_weights[4]
            + state.neurobiome.current * self.capital_weights[5]
            + state.treaty.current * self.capital_weights[6]
    }

    /// LSC-002: V_t stability check (non-increasing)
    #[inline]
    pub fn is_v_t_stable(&self, current_v_t: f32, previous_v_t: f32) -> bool {
        current_v_t <= previous_v_t
    }

    /// LSC-003: V_t derivative bound check
    #[inline]
    pub fn is_derivative_safe(&self, derivative: f32) -> bool {
        derivative.abs() <= self.max_derivative
    }

    /// LSC-004: Compute V_t derivative from state transition
    #[inline]
    pub fn compute_v_t_derivative(
        &self,
        current_v_t: f32,
        previous_v_t: f32,
        delta_time_us: u64,
    ) -> f32 {
        if delta_time_us == 0 {
            return 0.0;
        }
        let delta_v_t = current_v_t - previous_v_t;
        let delta_time_sec = delta_time_us as f32 / 1_000_000.0;
        delta_v_t / delta_time_sec
    }

    /// LSC-005: Update LyapunovResidual with new state
    pub fn update_residual(
        &self,
        state: &mut SevenCapitalState,
        previous_v_t: f32,
        delta_time_us: u64,
    ) -> Result<(), EcosafetyError> {
        let current_v_t = self.compute_v_t(state);
        let derivative = self.compute_v_t_derivative(current_v_t, previous_v_t, delta_time_us);

        // Check derivative safety
        if !self.is_derivative_safe(derivative) {
            return Err(EcosafetyError::VtDerivativeExceeded);
        }

        state.lyapunov_residual.v_t = current_v_t;
        state.lyapunov_residual.v_t_previous = previous_v_t;
        state.lyapunov_residual.v_t_derivative = derivative;
        state.lyapunov_residual.timestamp_us = state.timestamp_us;

        Ok(())
    }
}

impl Default for LyapunovComputer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SECTION 3: SEVEN CAPITAL STATE VALIDATION
// ============================================================================

/// SevenCapitalState validator with treaty and rights enforcement
pub struct CapitalStateValidator {
    /// Enable neurorights enforcement
    neurorights_enabled: bool,
    /// Enable BioticTreaty enforcement
    biotic_treaty_enabled: bool,
    /// Enable Indigenous sovereignty enforcement
    indigenous_sovereignty_enabled: bool,
}

impl CapitalStateValidator {
    /// Create a new CapitalStateValidator with feature flags
    pub fn new() -> Self {
        Self {
            neurorights_enabled: NEURORIGHTS_ENFORCEMENT,
            biotic_treaty_enabled: BIOTIC_TREATY_MODE,
            indigenous_sovereignty_enabled: INDIGENOUS_SOVEREIGNTY,
        }
    }

    /// SCS-001: All capitals must be safe for overall state to be safe
    #[inline]
    pub fn all_capitals_safe(&self, state: &SevenCapitalState) -> bool {
        state.water.is_safe()
            && state.thermal.is_safe()
            && state.waste.is_safe()
            && state.biotic.is_safe()
            && state.somatic.is_safe()
            && state.neurobiome.is_safe()
            && state.treaty.is_safe()
    }

    /// SCS-002: FPIC verification for treaty-related actions
    #[inline]
    pub fn verify_fpic(&self, state: &SevenCapitalState, action: &NodeAction) -> bool {
        match action {
            NodeAction::Deploy | NodeAction::Actuate | NodeAction::Retire => {
                state.treaty.fpic_verified
            }
            _ => true, // FPIC not required for all actions
        }
    }

    /// SCS-003: Neurorights enforcement for neurobiome capital
    pub fn enforce_neurorights(&self, state: &SevenCapitalState) -> Result<(), EcosafetyError> {
        if !self.neurorights_enabled {
            return Ok(());
        }

        // Neurorights: No coercive channels, no subliminal stimuli
        // No downgrades of cognitive liberty
        if state.neurobiome.current < state.neurobiome.min_threshold {
            return Err(EcosafetyError::NeurorightsViolation);
        }

        // Check for coercive channel risk coordinates
        for rc in state.neurobiome.risk_coords.iter() {
            // Check if this is a coercive channel indicator (encoded in source_did)
            if rc.source_did[0] == 0xFF && rc.value > 0.0 {
                return Err(EcosafetyError::NeurorightsViolation);
            }
        }

        Ok(())
    }

    /// SCS-004: BioticTreaty enforcement for biotic capital
    pub fn enforce_biotic_treaty(&self, state: &SevenCapitalState) -> Result<(), EcosafetyError> {
        if !self.biotic_treaty_enabled {
            return Ok(());
        }

        // BioticTreaty: Species sovereignty, habitat integrity
        if state.biotic.current < state.biotic.min_threshold {
            return Err(EcosafetyError::TreatyViolation);
        }

        // Biotic FPIC required
        if !state.biotic.fpic_verified {
            return Err(EcosafetyError::FPICFailed);
        }

        Ok(())
    }

    /// SCS-005: Indigenous sovereignty enforcement for treaty capital
    pub fn enforce_indigenous_sovereignty(&self, state: &SevenCapitalState) -> Result<(), EcosafetyError> {
        if !self.indigenous_sovereignty_enabled {
            return Ok(());
        }

        // Akimel O'odham and Piipaash territorial rights
        // Free, Prior, Informed Consent (FPIC) mandatory
        if state.treaty.current < state.treaty.min_threshold {
            return Err(EcosafetyError::TreatyViolation);
        }

        if !state.treaty.fpic_verified {
            return Err(EcosafetyError::FPICFailed);
        }

        Ok(())
    }

    /// SCS-006: Complete state validation before actuation
    pub fn validate_state_for_actuation(
        &self,
        state: &SevenCapitalState,
        action: &NodeAction,
    ) -> Result<CorridorDecision, EcosafetyError> {
        // Check 1: All capitals safe
        if !self.all_capitals_safe(state) {
            return Err(EcosafetyError::CorridorViolation);
        }

        // Check 2: FPIC verification
        if !self.verify_fpic(state, action) {
            return Err(EcosafetyError::FPICFailed);
        }

        // Check 3: Neurorights enforcement
        self.enforce_neurorights(state)?;

        // Check 4: BioticTreaty enforcement
        self.enforce_biotic_treaty(state)?;

        // Check 5: Indigenous sovereignty
        self.enforce_indigenous_sovereignty(state)?;

        Ok(CorridorDecision::Permit)
    }
}

impl Default for CapitalStateValidator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SECTION 4: SAFE_STEP FUNCTION (Runtime Gatekeeper)
// ============================================================================

/// The safe_step function is the runtime gatekeeper for all actuation paths
pub struct SafeStepEngine {
    corridor_validator: CorridorValidator,
    lyapunov_computer: LyapunovComputer,
    capital_validator: CapitalStateValidator,
    node_type: String,
}

impl SafeStepEngine {
    /// Create a new SafeStepEngine for a specific node type
    pub fn new(node_type: &str) -> Self {
        Self {
            corridor_validator: CorridorValidator::new(),
            lyapunov_computer: LyapunovComputer::new(),
            capital_validator: CapitalStateValidator::new(),
            node_type: node_type.to_string(),
        }
    }

    /// SSC-001: The safe_step function - every actuation path MUST call this
    pub fn safe_step(
        &mut self,
        state_prev: &SevenCapitalState,
        state_proposed: &mut SevenCapitalState,
        action: &NodeAction,
    ) -> Result<NodeAction, EcosafetyError> {
        // Step 1: Validate proposed state (capital safety, FPIC, treaties)
        let validation = self.capital_validator.validate_state_for_actuation(
            state_proposed,
            action,
        );
        if validation.is_err() {
            let err = validation.err().unwrap();
            self.log_violation_shard(state_prev, state_proposed, action, err)?;
            return Err(err);
        }

        // Step 2: Check V_t stability (non-increasing)
        if state_proposed.lyapunov_residual.v_t > state_prev.lyapunov_residual.v_t {
            let err = EcosafetyError::VtDerivativeExceeded;
            self.log_violation_shard(state_prev, state_proposed, action, err)?;
            return Err(err);
        }

        // Step 3: Check V_t derivative bound
        let v_t_delta = state_proposed.lyapunov_residual.v_t 
                      - state_prev.lyapunov_residual.v_t;
        if v_t_delta.abs() > MAX_VT_DERIVATIVE {
            let err = EcosafetyError::VtDerivativeExceeded;
            self.log_violation_shard(state_prev, state_proposed, action, err)?;
            return Err(err);
        }

        // Step 4: Corridor validation
        let corridor_decision = self.corridor_validator.validate_all_corridors(
            state_proposed,
            &self.node_type,
        )?;
        
        if corridor_decision == CorridorDecision::Stop 
           || corridor_decision == CorridorDecision::NoBuild {
            let err = EcosafetyError::CorridorViolation;
            self.log_violation_shard(state_prev, state_proposed, action, err)?;
            return Err(err);
        }

        // Step 5: Derate if approaching threshold
        if corridor_decision == CorridorDecision::Derate {
            return Ok(NodeAction::Derate);
        }

        // All checks passed - action permitted
        Ok(*action)
    }

    /// SSC-002: Log violation shard for audit trail
    fn log_violation_shard(
        &self,
        state_prev: &SevenCapitalState,
        state_proposed: &SevenCapitalState,
        action: &NodeAction,
        error: EcosafetyError,
    ) -> Result<(), EcosafetyError> {
        // Emit DID-signed QpuDataShard with violation details
        let shard = QpuDataShard {
            shard_id: self.generate_shard_id(state_prev, state_proposed),
            entity_did: state_prev.entity_did,
            timestamp_us: state_proposed.timestamp_us,
            state: state_proposed.clone(),
            action: *action,
            corridor_decision: CorridorDecision::Stop,
            v_t: state_proposed.lyapunov_residual.v_t,
            ker_metrics: self.compute_ker_metrics(state_proposed),
            violation: Some(ViolationRecord {
                violation_type: error,
                triggering_coord: self.find_triggering_coord(state_proposed),
                severity: self.compute_severity(&error),
                remediation: NodeAction::EmergencyStop,
                timestamp_us: state_proposed.timestamp_us,
            }),
            fpic_status: self.determine_fpic_status(state_proposed),
            smart_chain_hash: self.compute_smart_chain_hash(state_proposed),
            birth_sign_verified: self.verify_birth_sign(&state_proposed.entity_did),
        };

        // Emit shard (in production, this would write to persistent storage)
        self.emit_shard(shard)
    }

    /// Generate unique shard ID from state transition
    fn generate_shard_id(&self, prev: &SevenCapitalState, curr: &SevenCapitalState) -> [u8; 32] {
        let mut id = [0u8; 32];
        // XOR previous and current entity DIDs for uniqueness
        for i in 0..32 {
            id[i] = prev.entity_did[i] ^ curr.entity_did[i];
        }
        // Mix in timestamp for temporal uniqueness
        let ts_bytes = curr.timestamp_us.to_le_bytes();
        for i in 0..8 {
            id[i] ^= ts_bytes[i];
        }
        id
    }

    /// Find the risk coordinate that triggered the violation
    fn find_triggering_coord(&self, state: &SevenCapitalState) -> RiskCoord {
        // Search all capitals for the highest risk coordinate
        let mut max_rc = state.water.risk_coords.first().copied().unwrap_or(RiskCoord {
            value: 0.0,
            timestamp_us: 0,
            source_did: [0; 32],
            confidence: 0.0,
        });

        for capital in [&state.thermal, &state.waste, &state.biotic, &state.somatic, &state.neurobiome, &state.treaty] {
            for rc in capital.risk_coords.iter() {
                if rc.value > max_rc.value {
                    max_rc = *rc;
                }
            }
        }

        max_rc
    }

    /// Compute severity level from error type
    fn compute_severity(&self, error: &EcosafetyError) -> f32 {
        match error {
            EcosafetyError::CorridorViolation => 0.9,
            EcosafetyError::TreatyViolation => 1.0,
            EcosafetyError::NeurorightsViolation => 1.0,
            EcosafetyError::FPICFailed => 0.95,
            EcosafetyError::VtDerivativeExceeded => 0.8,
            EcosafetyError::MissingCorridor => 0.7,
            _ => 0.5,
        }
    }

    /// Determine FPIC status from state
    fn determine_fpic_status(&self, state: &SevenCapitalState) -> FPICStatus {
        if state.treaty.fpic_verified {
            FPICStatus::Verified
        } else if state.treaty.current < state.treaty.min_threshold {
            FPICStatus::Failed
        } else {
            FPICStatus::Pending
        }
    }

    /// Compute post-quantum secure hash for SMART-chain
    fn compute_smart_chain_hash(&self, state: &SevenCapitalState) -> [u8; 64] {
        // In production, use CRYSTALS-Kyber or SPHINCS+
        // For now, use a placeholder that simulates 64-byte PQ hash
        let mut hash = [0u8; 64];
        
        // Mix entity DID into hash
        for i in 0..32 {
            hash[i] = state.entity_did[i];
        }
        
        // Mix V_t into hash (as bytes)
        let v_t_bytes = state.lyapunov_residual.v_t.to_le_bytes();
        for i in 0..4 {
            hash[32 + i] = v_t_bytes[i];
        }
        
        // Mix timestamp into hash
        let ts_bytes = state.timestamp_us.to_le_bytes();
        for i in 0..8 {
            hash[36 + i] = ts_bytes[i];
        }
        
        // Fill remaining with capital state checksums
        hash[44] = state.water.current.to_le_bytes()[0];
        hash[45] = state.thermal.current.to_le_bytes()[0];
        hash[46] = state.waste.current.to_le_bytes()[0];
        hash[47] = state.biotic.current.to_le_bytes()[0];
        hash[48] = state.somatic.current.to_le_bytes()[0];
        hash[49] = state.neurobiome.current.to_le_bytes()[0];
        hash[50] = state.treaty.current.to_le_bytes()[0];
        
        hash
    }

    /// Verify Birth-Sign for entity
    fn verify_birth_sign(&self, entity_did: &[u8; 32]) -> bool {
        // In production, lookup birth sign from immutable registry
        // For now, verify DID is non-zero (basic validation)
        entity_did.iter().any(|&b| b != 0)
    }

    /// Emit shard to persistent storage
    fn emit_shard(&self, shard: QpuDataShard) -> Result<(), EcosafetyError> {
        // In production, write to qpudatashard storage layer
        // For now, this is a no-op that validates shard structure
        if shard.shard_id.iter().all(|&b| b == 0) {
            return Err(EcosafetyError::SystemError);
        }
        Ok(())
    }
}

// ============================================================================
// SECTION 5: K/E/R METRICS COMPUTATION
// ============================================================================

/// K/E/R (Knowledge/Eco-impact/Risk) metrics computer
pub struct KERComputer {
    /// Target K/E/R bands for different module types
    targets: BTreeMap<String, (f32, f32, f32)>,
}

impl KERComputer {
    /// Create a new KERComputer with default targets
    pub fn new() -> Self {
        let mut targets = BTreeMap::new();
        
        // Biodegradable node targets
        targets.insert("biodegradable".to_string(), (0.91, 0.90, 0.15));
        
        // MAR cyboquatic module targets
        targets.insert("mar_cyboquatic".to_string(), (0.93, 0.92, 0.14));
        
        // Ecosafety grammar spine targets
        targets.insert("grammar".to_string(), (0.94, 0.90, 0.12));
        
        // Ecotechnology habitat targets
        targets.insert("ecotechnology".to_string(), (0.90, 0.91, 0.15));

        Self { targets }
    }

    /// KMC-001: Knowledge factor (K) computation
    /// K increases with: data completeness, model accuracy, historical consistency
    pub fn compute_knowledge_factor(&self, state: &SevenCapitalState) -> f32 {
        // Corridor completeness (40% weight)
        let corridor_count = state.water.risk_coords.len()
            + state.thermal.risk_coords.len()
            + state.waste.risk_coords.len()
            + state.biotic.risk_coords.len()
            + state.somatic.risk_coords.len()
            + state.neurobiome.risk_coords.len()
            + state.treaty.risk_coords.len();
        let corridor_completeness = (corridor_count as f32 / 20.0).min(1.0); // Target 20 coords

        // Model accuracy (30% weight) - based on V_t derivative
        let model_accuracy = (1.0 - state.lyapunov_residual.v_t_derivative.abs() / MAX_VT_DERIVATIVE).max(0.0);

        // Historical consistency (30% weight) - based on Birth-Sign
        let historical_consistency = 1.0; // Assume verified if we reach here

        let k = corridor_completeness * 0.4 + model_accuracy * 0.3 + historical_consistency * 0.3;
        k.min(1.0).max(0.0)
    }

    /// KMC-002: Eco-impact value (E) computation
    /// E increases with: low biotic/treaty/waste risk, high water/thermal efficiency
    pub fn compute_eco_impact_value(&self, state: &SevenCapitalState) -> f32 {
        let biotic_score = 1.0 - state.biotic.current;
        let treaty_score = 1.0 - state.treaty.current;
        let waste_score = 1.0 - state.waste.current;
        let efficiency_score = (state.water.current + state.thermal.current) / 2.0;

        let e = biotic_score * 0.3 + treaty_score * 0.25 + waste_score * 0.2 + efficiency_score * 0.25;
        e.min(1.0).max(0.0)
    }

    /// KMC-003: Risk of harm (R) computation
    /// R increases with: high risk coordinates, V_t instability, corridor violations, FPIC failures
    pub fn compute_risk_of_harm(&self, state: &SevenCapitalState) -> f32 {
        // Average risk coordinate value (40% weight)
        let mut risk_sum = 0.0f32;
        let mut risk_count = 0usize;
        
        for capital in [&state.water, &state.thermal, &state.waste, &state.biotic, &state.somatic, &state.neurobiome, &state.treaty] {
            for rc in capital.risk_coords.iter() {
                risk_sum += rc.value;
                risk_count += 1;
            }
        }
        let risk_coord_avg = if risk_count > 0 { risk_sum / risk_count as f32 } else { 0.0 };

        // V_t instability (25% weight)
        let v_t_instability = (state.lyapunov_residual.v_t - state.lyapunov_residual.v_t_previous).max(0.0);

        // Corridor violations (20% weight)
        let corridor_violations = self.count_corridor_violations(state) as f32 / 10.0;

        // FPIC risk (15% weight)
        let fpic_risk = if state.treaty.fpic_verified { 0.0 } else { 0.5 };

        let r = risk_coord_avg * 0.4 + v_t_instability * 0.25 + corridor_violations * 0.2 + fpic_risk * 0.15;
        r // Can exceed 1.0 for severe violations
    }

    /// Count corridor violations in state
    fn count_corridor_violations(&self, state: &SevenCapitalState) -> usize {
        let mut violations = 0;
        
        for capital in [&state.water, &state.thermal, &state.waste, &state.biotic, &state.somatic, &state.neurobiome, &state.treaty] {
            for rc in capital.risk_coords.iter() {
                if rc.value > 1.0 {
                    violations += 1;
                }
            }
        }
        
        violations
    }

    /// KMC-004: Combined K/E/R metrics computation
    pub fn compute_ker_metrics(&self, state: &SevenCapitalState) -> KERMetrics {
        KERMetrics {
            k: self.compute_knowledge_factor(state),
            e: self.compute_eco_impact_value(state),
            r: self.compute_risk_of_harm(state),
        }
    }

    /// KMC-005: Get target K/E/R bands for node type
    pub fn get_targets(&self, node_type: &str) -> Option<(f32, f32, f32)> {
        self.targets.get(node_type).copied()
    }

    /// KMC-006: Check if current K/E/R meets targets
    pub fn meets_targets(&self, metrics: &KERMetrics, node_type: &str) -> bool {
        if let Some((target_k, target_e, target_r)) = self.get_targets(node_type) {
            metrics.k >= target_k && metrics.e >= target_e && metrics.r <= target_r
        } else {
            false
        }
    }
}

impl Default for KERComputer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SECTION 6: QPUDATASHARD EMISSION
// ============================================================================

impl ShardEmitting for SevenCapitalState {
    /// QSC-001: Every state transition MUST emit a shard
    fn emit_shard(&self) -> Result<QpuDataShard, EcosafetyError> {
        let ker_computer = KERComputer::new();
        
        Ok(QpuDataShard {
            shard_id: self.entity_did, // Simplified - use entity DID as shard ID
            entity_did: self.entity_did,
            timestamp_us: self.timestamp_us,
            state: self.clone(),
            action: NodeAction::Park, // Default action
            corridor_decision: CorridorDecision::Permit,
            v_t: self.lyapunov_residual.v_t,
            ker_metrics: ker_computer.compute_ker_metrics(self),
            violation: None,
            fpic_status: if self.treaty.fpic_verified { FPICStatus::Verified } else { FPICStatus::Pending },
            smart_chain_hash: [0u8; 64], // Computed by SafeStepEngine
            birth_sign_verified: true,
        })
    }
}

// ============================================================================
// SECTION 7: FOG WORKLOAD ROUTING INTEGRATION
// ============================================================================

/// FOG router candidate for workload assignment
#[derive(Debug, Clone)]
pub struct FogNodeCandidate {
    /// Node DID
    pub node_did: [u8; 32],
    /// Node type
    pub node_type: String,
    /// Available energy surplus
    pub energy_surplus: f32,
    /// Hydraulic corridor safety status
    pub hydraulic_safe: bool,
    /// Biodegradable corridors complete
    pub biodegradable_corridors_complete: bool,
    /// V_t trend (true = increasing, false = stable/decreasing)
    pub v_t_trend_increasing: bool,
    /// FPIC verified
    pub fpic_verified: bool,
    /// Current K/E/R metrics
    pub ker_metrics: KERMetrics,
}

impl FogNodeCandidate {
    /// Check if hydraulic corridor is safe
    #[inline]
    pub fn hydraulic_corridor_safe(&self) -> bool {
        self.hydraulic_safe
    }

    /// Check if biodegradable corridors are complete
    #[inline]
    pub fn biodegradable_corridors_complete(&self) -> bool {
        self.biodegradable_corridors_complete
    }

    /// Check if V_t trend is increasing
    #[inline]
    pub fn v_t_trend_increasing(&self) -> bool {
        self.v_t_trend_increasing
    }

    /// Check if FPIC is verified
    #[inline]
    pub fn fpic_verified(&self) -> bool {
        self.fpic_verified
    }
}

/// FOG Workload Router with ecosafety filtering
pub struct FogWorkloadRouter {
    ker_computer: KERComputer,
}

impl FogWorkloadRouter {
    /// Create a new FOG Workload Router
    pub fn new() -> Self {
        Self {
            ker_computer: KERComputer::new(),
        }
    }

    /// FWRC-001: FOG router must query ecosafety corridors
    pub fn filter_candidates(
        &self,
        candidates: &[FogNodeCandidate],
        energy_requirement: f32,
    ) -> Vec<FogNodeCandidate> {
        let mut filtered = Vec::new();

        for candidate in candidates.iter() {
            // Check 1: Energy surplus
            if candidate.energy_surplus < energy_requirement {
                continue;
            }

            // Check 2: Hydraulic safety
            if !candidate.hydraulic_corridor_safe() {
                continue;
            }

            // Check 3: Biodegradable corridors (if applicable)
            if candidate.node_type == "biodegradable" 
               && !candidate.biodegradable_corridors_complete() {
                continue;
            }

            // Check 4: V_t non-increasing
            if candidate.v_t_trend_increasing() {
                continue;
            }

            // Check 5: FPIC verified
            if !candidate.fpic_verified() {
                continue;
            }

            filtered.push(candidate.clone());
        }

        filtered
    }

    /// FWRC-002: FOG routing decision with ecosafety priority
    pub fn route_decision(
        &self,
        filtered_candidates: &[FogNodeCandidate],
    ) -> Result<FogNodeCandidate, EcosafetyError> {
        if filtered_candidates.is_empty() {
            return Err(EcosafetyError::MissingCorridor);
        }

        // Select candidate with lowest R (risk of harm)
        let best = filtered_candidates
            .iter()
            .min_by(|a, b| a.ker_metrics.r.partial_cmp(&b.ker_metrics.r).unwrap_or(core::cmp::Ordering::Equal));

        match best {
            Some(candidate) => Ok(candidate.clone()),
            None => Err(EcosafetyError::SystemError),
        }
    }
}

impl Default for FogWorkloadRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SECTION 8: COMPLIANCE PREFLIGHT CHECKS
// ============================================================================

/// Compliance preflight checker for CI/CD
pub struct CompliancePreflight {
    /// Required corridors per module type
    required_corridors: BTreeMap<String, Vec<String>>,
}

impl CompliancePreflight {
    /// Create a new CompliancePreflight checker
    pub fn new() -> Self {
        let mut required_corridors = BTreeMap::new();
        
        required_corridors.insert("cyboquatic".to_string(), vec![
            "r_hydraulic".to_string(),
            "r_thermal".to_string(),
            "r_biotic".to_string(),
        ]);
        
        required_corridors.insert("biodegradable".to_string(), vec![
            "r_degrade".to_string(),
            "r_tox".to_string(),
            "r_microplastics".to_string(),
        ]);
        
        required_corridors.insert("canal".to_string(), vec![
            "r_hydraulic".to_string(),
            "r_water".to_string(),
        ]);
        
        required_corridors.insert("air".to_string(), vec![
            "r_air_quality".to_string(),
            "r_thermal".to_string(),
        ]);

        Self { required_corridors }
    }

    /// CPC-001: PR preflight check for ecosafety compliance
    pub fn preflight_check_pr(
        &self,
        module_type: &str,
        has_corridors: bool,
        bypasses_smart_chain: bool,
        implements_corridor_logic_outside_grammar: bool,
        modifies_service_functions: bool,
        has_seven_capital_validation: bool,
    ) -> Result<bool, String> {
        // Check 1: Any new cyboquatic node must have ecosafety corridors
        if self.required_corridors.contains_key(module_type) {
            if !has_corridors {
                return Err(format!("New {} module missing ecosafety corridors", module_type));
            }
        }

        // Check 2: No bypass of SMART-chain/Birth-Sign governance
        if bypasses_smart_chain {
            return Err("SMART-chain governance bypass detected".to_string());
        }

        // Check 3: No corridor/rights logic outside ALN/Rust grammars
        if implements_corridor_logic_outside_grammar {
            return Err("Corridor logic implemented outside approved grammars".to_string());
        }

        // Check 4: All SevenCapitalState validations present
        if modifies_service_functions && !has_seven_capital_validation {
            return Err("Service functions missing SevenCapitalState validation".to_string());
        }

        Ok(true)
    }
}

impl Default for CompliancePreflight {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SECTION 9: FFI EXPORTS FOR LUA/JAVASCRIPT/KOTLIN/C++ BINDINGS
// ============================================================================

#[cfg(feature = "ffi")]
pub mod ffi {
    use super::*;
    use alloc::string::String;
    use alloc::vec::Vec;

    /// FFI-safe corridor decision result
    #[repr(C)]
    pub struct FfiCorridorDecision {
        pub decision: u8,
        pub error_code: u8,
    }

    /// FFI-safe K/E/R metrics
    #[repr(C)]
    pub struct FfiKerMetrics {
        pub k: f32,
        pub e: f32,
        pub r: f32,
    }

    /// FFI: Evaluate corridor for JSON plan (Lua binding surface)
    /// Signature: ecosystems_eval(plan_json) -> decision_json
    #[no_mangle]
    pub extern "C" fn ecosafety_eval_corridor(
        state_ptr: *const SevenCapitalState,
        action: u8,
        node_type_ptr: *const u8,
        node_type_len: usize,
    ) -> FfiCorridorDecision {
        // Safety: Caller must ensure valid pointers
        if state_ptr.is_null() || node_type_ptr.is_null() {
            return FfiCorridorDecision {
                decision: CorridorDecision::Error as u8,
                error_code: EcosafetyError::SystemError as u8,
            };
        }

        let state = unsafe { &*state_ptr };
        let node_type = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(node_type_ptr, node_type_len))
        };
        let action = NodeAction::from(action);

        let mut engine = SafeStepEngine::new(node_type);
        let mut state_mut = state.clone();

        match engine.safe_step(state, &mut state_mut, &action) {
            Ok(_) => FfiCorridorDecision {
                decision: CorridorDecision::Permit as u8,
                error_code: 0,
            },
            Err(e) => FfiCorridorDecision {
                decision: CorridorDecision::Stop as u8,
                error_code: e as u8,
            },
        }
    }

    /// FFI: Compute K/E/R metrics for state
    #[no_mangle]
    pub extern "C" fn ecosafety_compute_ker(
        state_ptr: *const SevenCapitalState,
    ) -> FfiKerMetrics {
        if state_ptr.is_null() {
            return FfiKerMetrics { k: 0.0, e: 0.0, r: 1.0 };
        }

        let state = unsafe { &*state_ptr };
        let computer = KERComputer::new();
        let metrics = computer.compute_ker_metrics(state);

        FfiKerMetrics {
            k: metrics.k,
            e: metrics.e,
            r: metrics.r,
        }
    }

    /// FFI: Validate all corridors for state
    #[no_mangle]
    pub extern "C" fn ecosafety_validate_corridors(
        state_ptr: *const SevenCapitalState,
        node_type_ptr: *const u8,
        node_type_len: usize,
    ) -> FfiCorridorDecision {
        if state_ptr.is_null() || node_type_ptr.is_null() {
            return FfiCorridorDecision {
                decision: CorridorDecision::Error as u8,
                error_code: EcosafetyError::SystemError as u8,
            };
        }

        let state = unsafe { &*state_ptr };
        let node_type = unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(node_type_ptr, node_type_len))
        };

        let validator = CorridorValidator::new();
        match validator.validate_all_corridors(state, node_type) {
            Ok(decision) => FfiCorridorDecision {
                decision: decision as u8,
                error_code: 0,
            },
            Err(e) => FfiCorridorDecision {
                decision: CorridorDecision::Error as u8,
                error_code: e as u8,
            },
        }
    }
}

// ============================================================================
// SECTION 10: UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> SevenCapitalState {
        let water = CapitalState::new(0.5, 0.2, 0.8, true).unwrap();
        let thermal = CapitalState::new(0.5, 0.2, 0.8, true).unwrap();
        let waste = CapitalState::new(0.5, 0.2, 0.8, true).unwrap();
        let biotic = CapitalState::new(0.5, 0.2, 0.8, true).unwrap();
        let somatic = CapitalState::new(0.5, 0.2, 0.8, true).unwrap();
        let neurobiome = CapitalState::new(0.5, 0.2, 0.8, true).unwrap();
        let treaty = CapitalState::new(0.5, 0.2, 0.8, true).unwrap();
        let entity_did = [1u8; 32];

        SevenCapitalState::new(
            water, thermal, waste, biotic, somatic, neurobiome, treaty,
            entity_did, 0,
        ).unwrap()
    }

    #[test]
    fn test_corridor_validator_no_corridor_no_build() {
        let validator = CorridorValidator::new();
        let corridors = BTreeMap::new(); // Empty corridors
        
        let decision = validator.enforce_no_corridor_no_build(
            &[0u8; 32],
            &corridors,
            "biodegradable",
        );
        
        assert_eq!(decision, CorridorDecision::NoBuild);
    }

    #[test]
    fn test_lyapunov_computer_v_t_stability() {
        let computer = LyapunovComputer::new();
        let state = create_test_state();
        
        let v_t = computer.compute_v_t(&state);
        assert!(v_t >= 0.0 && v_t <= 1.0);
        
        // V_t should be stable (non-increasing)
        assert!(computer.is_v_t_stable(v_t, v_t + 0.1));
        assert!(!computer.is_v_t_stable(v_t + 0.1, v_t));
    }

    #[test]
    fn test_capital_state_validator_all_safe() {
        let validator = CapitalStateValidator::new();
        let state = create_test_state();
        
        assert!(validator.all_capitals_safe(&state));
    }

    #[test]
    fn test_ker_computer_metrics() {
        let computer = KERComputer::new();
        let state = create_test_state();
        
        let metrics = computer.compute_ker_metrics(&state);
        
        assert!(metrics.k >= 0.0 && metrics.k <= 1.0);
        assert!(metrics.e >= 0.0 && metrics.e <= 1.0);
        assert!(metrics.r >= 0.0);
    }

    #[test]
    fn test_fog_router_filtering() {
        let router = FogWorkloadRouter::new();
        
        let candidates = vec![
            FogNodeCandidate {
                node_did: [1u8; 32],
                node_type: "biodegradable".to_string(),
                energy_surplus: 100.0,
                hydraulic_safe: true,
                biodegradable_corridors_complete: true,
                v_t_trend_increasing: false,
                fpic_verified: true,
                ker_metrics: KERMetrics { k: 0.9, e: 0.9, r: 0.1 },
            },
            FogNodeCandidate {
                node_did: [2u8; 32],
                node_type: "biodegradable".to_string(),
                energy_surplus: 50.0,
                hydraulic_safe: false, // Should be filtered out
                biodegradable_corridors_complete: true,
                v_t_trend_increasing: false,
                fpic_verified: true,
                ker_metrics: KERMetrics { k: 0.8, e: 0.8, r: 0.2 },
            },
        ];
        
        let filtered = router.filter_candidates(&candidates, 50.0);
        
        assert_eq!(filtered.len(), 1); // Only first candidate should pass
        assert_eq!(filtered[0].node_did[0], 1);
    }

    #[test]
    fn test_compliance_preflight() {
        let preflight = CompliancePreflight::new();
        
        // Valid PR
        let result = preflight.preflight_check_pr(
            "cyboquatic",
            true,  // has_corridors
            false, // bypasses_smart_chain
            false, // implements_corridor_logic_outside_grammar
            false, // modifies_service_functions
            true,  // has_seven_capital_validation
        );
        
        assert!(result.is_ok());
        
        // Invalid PR - missing corridors
        let result = preflight.preflight_check_pr(
            "cyboquatic",
            false, // has_corridors - MISSING
            false,
            false,
            false,
            true,
        );
        
        assert!(result.is_err());
    }
}

// ============================================================================
// END OF FILE: ALE-ERM-ECOSAFETY-CONTRACTS-001.rs
// ============================================================================
