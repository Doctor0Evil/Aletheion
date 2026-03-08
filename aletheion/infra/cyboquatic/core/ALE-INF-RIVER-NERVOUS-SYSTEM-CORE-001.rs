#![forbid(unsafe_code)]
#![deny(warnings)]

use core::fmt;
use core::time::Duration;

/// Core corridor identifier across water, sewer, stormwater, and habitat flows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CorridorId(pub u64);

/// Identifier for a physical reach (canal segment, river reach, sewer trunk slice).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReachId(pub u64);

/// Identifier for a side cell (SAT basin, retention basin, diversion vault).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SideCellId(pub u64);

/// Identifier for a BioticTreaty or species corridor binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BioticTreatyId(pub u64);

/// Identifier for an Indigenous territory or FPIC envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IndigenousTerritoryId(pub u64);

/// Identifier for a governance corridor / ROW policy shard.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GovernanceCorridorId(pub u64);

/// Identifier for a SMART chain (Sense→Model→Allocate→Rule-check→Actuate→Record→Talk-back).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SmartChainId(pub u64);

/// Identifier for a capital type, to allow reuse of this nervous-system core across domains.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CapitalKind {
    Water,
    Sewer,
    Stormwater,
    Thermal,
    AirQuality,
    SomaticRoute,
    Neurobiome,
}

/// Governance hooks for any corridor slice or actuation event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GovernanceHooks {
    pub indigenous_territory: Option<IndigenousTerritoryId>,
    pub biotic_treaties: Vec<BioticTreatyId>,
    pub governance_corridor: Option<GovernanceCorridorId>,
    pub smart_chain: Option<SmartChainId>,
    /// True if the decision is legally required to be irreversible (WORM-only).
    pub irreversible: bool,
}

/// Risk dimensions that contribute to the normalized risk vector r_j.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RiskDimension {
    /// Hydrologic / hydraulic safety (flooding, overtopping, cavitation).
    HydroSafety,
    /// Water quality and treatment risk (AWP stability, pollutant load).
    WaterQuality,
    /// Infrastructure health (pump wear, vibration, energy stress).
    AssetIntegrity,
    /// Treaty and Indigenous rights compliance.
    TreatyCompliance,
    /// Biotic / habitat continuity risk.
    BioticContinuity,
    /// Thermal / microclimate risk for humans.
    ThermalStress,
    /// Somatic / movement and ergonomics risk.
    SomaticRisk,
    /// Neurorights / biosignal privacy exposure.
    NeuroRights,
}

/// Normalized risk coordinate r_j for one dimension, in [-1, 1].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiskCoordinate {
    pub dimension: RiskDimension,
    /// Normalized risk, with 0 meaning nominal, positive above target, negative below target.
    pub value: f64,
}

impl RiskCoordinate {
    pub fn clamped(self) -> Self {
        let v = self.value.max(-1.0).min(1.0);
        Self { value: v, ..self }
    }
}

/// Static weighting for each risk dimension in the Lyapunov residual V_t.
#[derive(Clone, Debug, PartialEq)]
pub struct RiskWeights {
    pub hydro_safety: f64,
    pub water_quality: f64,
    pub asset_integrity: f64,
    pub treaty_compliance: f64,
    pub biotic_continuity: f64,
    pub thermal_stress: f64,
    pub somatic_risk: f64,
    pub neuro_rights: f64,
}

impl RiskWeights {
    pub fn unit() -> Self {
        Self {
            hydro_safety: 1.0,
            water_quality: 1.0,
            asset_integrity: 1.0,
            treaty_compliance: 1.0,
            biotic_continuity: 1.0,
            thermal_stress: 1.0,
            somatic_risk: 1.0,
            neuro_rights: 1.0,
        }
    }

    pub fn weight(&self, dim: RiskDimension) -> f64 {
        match dim {
            RiskDimension::HydroSafety => self.hydro_safety,
            RiskDimension::WaterQuality => self.water_quality,
            RiskDimension::AssetIntegrity => self.asset_integrity,
            RiskDimension::TreatyCompliance => self.treaty_compliance,
            RiskDimension::BioticContinuity => self.biotic_continuity,
            RiskDimension::ThermalStress => self.thermal_stress,
            RiskDimension::SomaticRisk => self.somatic_risk,
            RiskDimension::NeuroRights => self.neuro_rights,
        }
    }
}

/// Lyapunov-style residual V_t = sum_j w_j * r_j^2.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LyapunovResidual {
    pub value: f64,
}

impl LyapunovResidual {
    pub fn zero() -> Self {
        Self { value: 0.0 }
    }

    pub fn from_risk(risks: &[RiskCoordinate], weights: &RiskWeights) -> Self {
        let mut v = 0.0;
        for r in risks {
            let rc = r.clamped();
            let w = weights.weight(rc.dimension);
            v += w * rc.value * rc.value;
        }
        Self { value: v }
    }
}

/// Stability verdict for a corridor slice, given V_t and band thresholds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StabilityVerdict {
    /// All risk coordinates and V_t are inside safe bands.
    Safe,
    /// Soft violation; can act only within derated envelopes or with human-in-loop.
    SoftViolation,
    /// Hard violation; no autonomous act allowed (“no corridor, no act”).
    HardViolation,
}

/// Per-dimension risk band threshold.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiskBand {
    pub soft_limit: f64,
    pub hard_limit: f64,
}

impl RiskBand {
    pub fn new(soft: f64, hard: f64) -> Self {
        Self {
            soft_limit: soft.abs(),
            hard_limit: hard.abs(),
        }
    }

    pub fn classify(&self, value: f64) -> StabilityVerdict {
        let v = value.abs();
        if v > self.hard_limit {
            StabilityVerdict::HardViolation
        } else if v > self.soft_limit {
            StabilityVerdict::SoftViolation
        } else {
            StabilityVerdict::Safe
        }
    }
}

/// Per-corridor Lyapunov bands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResidualBand {
    pub soft_limit: f64,
    pub hard_limit: f64,
}

impl ResidualBand {
    pub fn new(soft: f64, hard: f64) -> Self {
        Self {
            soft_limit: soft.abs(),
            hard_limit: hard.abs(),
        }
    }

    pub fn classify(&self, v: LyapunovResidual) -> StabilityVerdict {
        let x = v.value;
        if x > self.hard_limit {
            StabilityVerdict::HardViolation
        } else if x > self.soft_limit {
            StabilityVerdict::SoftViolation
        } else {
            StabilityVerdict::Safe
        }
    }
}

/// Aggregated risk bands for all dimensions in a corridor slice.
#[derive(Clone, Debug, PartialEq)]
pub struct CorridorRiskBands {
    pub hydro_safety: RiskBand,
    pub water_quality: RiskBand,
    pub asset_integrity: RiskBand,
    pub treaty_compliance: RiskBand,
    pub biotic_continuity: RiskBand,
    pub thermal_stress: RiskBand,
    pub somatic_risk: RiskBand,
    pub neuro_rights: RiskBand,
    pub residual_band: ResidualBand,
}

impl CorridorRiskBands {
    pub fn band_for(&self, dim: RiskDimension) -> RiskBand {
        match dim {
            RiskDimension::HydroSafety => self.hydro_safety,
            RiskDimension::WaterQuality => self.water_quality,
            RiskDimension::AssetIntegrity => self.asset_integrity,
            RiskDimension::TreatyCompliance => self.treaty_compliance,
            RiskDimension::BioticContinuity => self.biotic_continuity,
            RiskDimension::ThermalStress => self.thermal_stress,
            RiskDimension::SomaticRisk => self.somatic_risk,
            RiskDimension::NeuroRights => self.neuro_rights,
        }
    }
}

/// A time window along a corridor where particular trajectories are allowed (e.g., monsoon pulse).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClimaxWindow {
    pub corridor: CorridorId,
    pub starts_at: u64,
    pub ends_at: u64,
}

/// KER scoring for a single actuation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KERScore {
    /// Knowledge impact (e.g., learning value, model calibration).
    pub k_delta: f64,
    /// Eco / ecological impact (positive for regeneration, negative for damage).
    pub e_delta: f64,
    /// Risk impact (positive increases risk, negative reduces).
    pub r_delta: f64,
}

/// Status of an actuation decision in a forward-only lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionStatus {
    Proposed,
    Authorized,
    Executed,
    Derated,
    Halted,
}

/// Unique identifier for a decision shard in the trust layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DecisionId(pub u64);

/// Result of checking whether an actuation is allowed under the corridor pattern.
#[derive(Clone, Debug, PartialEq)]
pub struct ActuationGuardResult {
    pub allowed: bool,
    pub reason: GuardFailureReason,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GuardFailureReason {
    None,
    CorridorUnknown,
    OutsideClimaxWindow,
    HardViolationResidual,
    HardViolationDimension(RiskDimension),
    MissingGovernanceHooks,
}

/// Basic physical characterization of a corridor slice.
#[derive(Clone, Debug, PartialEq)]
pub struct CorridorSlice {
    pub capital: CapitalKind,
    pub corridor: CorridorId,
    pub reach: ReachId,
    pub side_cell: Option<SideCellId>,
    pub upstream: Vec<ReachId>,
    pub downstream: Vec<ReachId>,
    pub governance: GovernanceHooks,
    pub risk_weights: RiskWeights,
    pub risk_bands: CorridorRiskBands,
}

/// Snapshot of risk for one corridor slice at a given time.
#[derive(Clone, Debug, PartialEq)]
pub struct CorridorRiskSnapshot {
    pub corridor: CorridorId,
    pub timestamp_ns: u64,
    pub risks: Vec<RiskCoordinate>,
    pub lyapunov: LyapunovResidual,
    pub verdict: StabilityVerdict,
}

/// A hydro-robotic actuation event, with KER and residual before/after.
#[derive(Clone, Debug, PartialEq)]
pub struct ActuationEvent {
    pub id: DecisionId,
    pub status: DecisionStatus,
    pub capital: CapitalKind,
    pub corridor: CorridorId,
    pub reach: ReachId,
    pub side_cell: Option<SideCellId>,
    pub governance: GovernanceHooks,
    pub proposed_at_ns: u64,
    pub executed_at_ns: Option<u64>,
    pub ker_score: KERScore,
    pub residual_before: LyapunovResidual,
    pub residual_after: LyapunovResidual,
    pub risk_before: Vec<RiskCoordinate>,
    pub risk_after: Vec<RiskCoordinate>,
    /// True if this actuation is flagged as irreversible at the governance level.
    pub irreversible: bool,
}

impl fmt::Display for ActuationEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ActuationEvent(id={:?}, status={:?}, capital={:?}, corridor={:?})",
            self.id, self.status, self.capital, self.corridor
        )
    }
}

/// Engine responsible for evaluating corridor stability and guarding actuation requests.
/// This struct deliberately has no hardware-facing traits.
#[derive(Clone, Debug)]
pub struct RiverNervousSystemCore {
    pub slices: Vec<CorridorSlice>,
}

impl RiverNervousSystemCore {
    pub fn new() -> Self {
        Self { slices: Vec::new() }
    }

    pub fn add_slice(&mut self, slice: CorridorSlice) {
        self.slices.push(slice);
    }

    pub fn find_slice(&self, corridor: CorridorId) -> Option<&CorridorSlice> {
        self.slices.iter().find(|s| s.corridor == corridor)
    }

    /// Evaluate risk snapshot and produce verdict given corridor configuration and current r_j.
    pub fn evaluate_snapshot(
        &self,
        corridor: CorridorId,
        timestamp_ns: u64,
        risks: Vec<RiskCoordinate>,
    ) -> Option<CorridorRiskSnapshot> {
        let slice = self.find_slice(corridor)?;
        let lyapunov = LyapunovResidual::from_risk(&risks, &slice.risk_weights);
        let mut verdict = slice.risk_bands.residual_band.classify(lyapunov);

        if verdict != StabilityVerdict::HardViolation {
            for r in &risks {
                let band = slice.risk_bands.band_for(r.dimension);
                let dim_verdict = band.classify(r.value);
                verdict = Self::max_verdict(verdict, dim_verdict);
                if verdict == StabilityVerdict::HardViolation {
                    break;
                }
            }
        }

        Some(CorridorRiskSnapshot {
            corridor,
            timestamp_ns,
            risks,
            lyapunov,
            verdict,
        })
    }

    fn max_verdict(a: StabilityVerdict, b: StabilityVerdict) -> StabilityVerdict {
        use StabilityVerdict::*;
        match (a, b) {
            (HardViolation, _) | (_, HardViolation) => HardViolation,
            (SoftViolation, _) | (_, SoftViolation) => SoftViolation,
            _ => Safe,
        }
    }

    /// Core guard: implements “no corridor, no act” and band checks.
    pub fn guard_actuation(
        &self,
        corridor: CorridorId,
        snapshot: &CorridorRiskSnapshot,
        now_ns: u64,
        window: Option<&ClimaxWindow>,
    ) -> ActuationGuardResult {
        let slice = match self.find_slice(corridor) {
            Some(s) => s,
            None => {
                return ActuationGuardResult {
                    allowed: false,
                    reason: GuardFailureReason::CorridorUnknown,
                }
            }
        };

        if slice.governance.irreversible && slice.governance.indigenous_territory.is_none() {
            return ActuationGuardResult {
                allowed: false,
                reason: GuardFailureReason::MissingGovernanceHooks,
            };
        }

        if let Some(win) = window {
            if now_ns < win.starts_at || now_ns > win.ends_at {
                return ActuationGuardResult {
                    allowed: false,
                    reason: GuardFailureReason::OutsideClimaxWindow,
                };
            }
        }

        if slice
            .risk_bands
            .residual_band
            .classify(snapshot.lyapunov)
            == StabilityVerdict::HardViolation
        {
            return ActuationGuardResult {
                allowed: false,
                reason: GuardFailureReason::HardViolationResidual,
            };
        }

        for r in &snapshot.risks {
            let band = slice.risk_bands.band_for(r.dimension);
            if band.classify(r.value) == StabilityVerdict::HardViolation {
                return ActuationGuardResult {
                    allowed: false,
                    reason: GuardFailureReason::HardViolationDimension(r.dimension),
                };
            }
        }

        ActuationGuardResult {
            allowed: true,
            reason: GuardFailureReason::None,
        }
    }

    /// Utility to construct an actuation event skeleton once guard passes; caller fills in
    /// optimization-derived fields like ker_score and residual_after.
    pub fn make_proposed_event(
        &self,
        id: DecisionId,
        corridor: CorridorId,
        reach: ReachId,
        side_cell: Option<SideCellId>,
        snapshot: &CorridorRiskSnapshot,
        governance: GovernanceHooks,
        irreversible: bool,
        proposed_at_ns: u64,
    ) -> Option<ActuationEvent> {
        let slice = self.find_slice(corridor)?;
        Some(ActuationEvent {
            id,
            status: DecisionStatus::Proposed,
            capital: slice.capital,
            corridor,
            reach,
            side_cell,
            governance,
            proposed_at_ns,
            executed_at_ns: None,
            ker_score: KERScore {
                k_delta: 0.0,
                e_delta: 0.0,
                r_delta: 0.0,
            },
            residual_before: snapshot.lyapunov,
            residual_after: snapshot.lyapunov,
            risk_before: snapshot.risks.clone(),
            risk_after: snapshot.risks.clone(),
            irreversible,
        })
    }

    /// Mark an event as executed in a forward-only fashion (no deletion).
    pub fn mark_executed(event: &mut ActuationEvent, executed_at_ns: u64) {
        event.status = DecisionStatus::Executed;
        event.executed_at_ns = Some(executed_at_ns);
    }

    /// Derate an event (e.g., throttle intensity) as a forward-only correction.
    pub fn mark_derated(event: &mut ActuationEvent) {
        event.status = DecisionStatus::Derated;
    }

    /// Halt an event (e.g., blocked by treaty or safety) as a forward-only outcome.
    pub fn mark_halted(event: &mut ActuationEvent) {
        event.status = DecisionStatus::Halted;
    }

    /// Helper for constructing a KER score from optimization outputs.
    pub fn ker(k_delta: f64, e_delta: f64, r_delta: f64) -> KERScore {
        KERScore {
            k_delta,
            e_delta,
            r_delta,
        }
    }

    /// Simple timeout helper for climax windows (e.g., monsoon pulse durations).
    pub fn window_from_now(corridor: CorridorId, now_ns: u64, duration: Duration) -> ClimaxWindow {
        let delta_ns = duration
            .as_secs()
            .saturating_mul(1_000_000_000)
            .saturating_add(duration.subsec_nanos() as u64);
        ClimaxWindow {
            corridor,
            starts_at: now_ns,
            ends_at: now_ns.saturating_add(delta_ns),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bands() -> CorridorRiskBands {
        CorridorRiskBands {
            hydro_safety: RiskBand::new(0.4, 0.8),
            water_quality: RiskBand::new(0.4, 0.8),
            asset_integrity: RiskBand::new(0.4, 0.8),
            treaty_compliance: RiskBand::new(0.2, 0.5),
            biotic_continuity: RiskBand::new(0.3, 0.7),
            thermal_stress: RiskBand::new(0.4, 0.8),
            somatic_risk: RiskBand::new(0.4, 0.8),
            neuro_rights: RiskBand::new(0.1, 0.3),
            residual_band: ResidualBand::new(0.5, 1.5),
        }
    }

    #[test]
    fn lyapunov_residual_basic() {
        let weights = RiskWeights::unit();
        let risks = vec![
            RiskCoordinate {
                dimension: RiskDimension::HydroSafety,
                value: 0.5,
            },
            RiskCoordinate {
                dimension: RiskDimension::WaterQuality,
                value: -0.5,
            },
        ];
        let v = LyapunovResidual::from_risk(&risks, &weights);
        assert!((v.value - 0.5).abs() < 1e-9);
    }

    #[test]
    fn guard_blocks_hard_violation() {
        let corridor = CorridorId(1);
        let mut core = RiverNervousSystemCore::new();
        core.add_slice(CorridorSlice {
            capital: CapitalKind::Water,
            corridor,
            reach: ReachId(10),
            side_cell: None,
            upstream: vec![],
            downstream: vec![],
            governance: GovernanceHooks {
                indigenous_territory: Some(IndigenousTerritoryId(7)),
                biotic_treaties: vec![],
                governance_corridor: None,
                smart_chain: None,
                irreversible: false,
            },
            risk_weights: RiskWeights::unit(),
            risk_bands: sample_bands(),
        });

        let risks = vec![RiskCoordinate {
            dimension: RiskDimension::HydroSafety,
            value: 1.0,
        }];
        let snapshot = core
            .evaluate_snapshot(corridor, 0, risks)
            .expect("snapshot");
        let guard = core.guard_actuation(corridor, &snapshot, 0, None);

        assert!(!guard.allowed);
        matches!(guard.reason, GuardFailureReason::HardViolationResidual | GuardFailureReason::HardViolationDimension(_));
    }
}
