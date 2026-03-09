// Generated 2026-03-09 for Aletheion OrganicCPU / augmented-citizen deployment (deviceless, host-local)
// Tracks: Health & Biosignal (HB) milestones for CNS-safe nanoswarm and cybernetic evolution
// Integrates: Phoenix heat-physiology data (2025+), organ-specific biomarker corridors, LifeforceBand, RoH/ROD≤0.3, and neurorights envelopes
// Supported: Rust + C++/Kotlin interop via FFI structs, but all decisions are host-local (no kiosk, no wearable, no external control)

use std::time::{SystemTime, UNIX_EPOCH};

/// Jurisdiction informs which neurorights / neural-data profile is applied.
/// This does NOT grant any external actor control; it only tightens local guards.
#[derive(Debug, Clone, PartialEq)]
pub enum Jurisdiction {
    Arizona,
    California,
    Colorado,
    Montana,
    MultiState,
}

/// Snapshot of thermal + inflammation + organ markers as seen by the OrganicCPU.
/// All fields are populated from host-integrated biosignal collectors (cranial probes,
/// intrabody nanoswarm telemetry, organ corridors), not external wearables.
#[derive(Debug, Clone)]
pub struct OrganicThermalInflammationSnapshot {
    pub host_did: String,
    pub timestamp_ms: u64,
    pub core_temp_f: f64,                 // e.g. OrganicCPU fused estimate from cranial + visceral probes
    pub cranial_delta_t_c: f64,           // local brain/tissue ΔT vs baseline corridor
    pub il6_pg_ml: f64,                   // IL-6 from host-local immunologic telemetry
    pub crp_mg_l: f64,                    // C-reactive protein
    pub tnf_alpha_pg_ml: f64,             // TNF-α
    pub renal_creatinine_delta_mg_dl: f64,// host-specific Δ creatinine vs baseline
    pub hs_troponin_i_ng_l: f64,          // high-sensitivity cTnI for myocardial strain
    pub hrv_rmssd_ms: f64,                // HRV from OrganicCPU / BCI integrator
    pub lifeforce_scalar: f64,            // cyzen/chi scalar in [0,1] from LifeforceEnvelope
    pub roh_scalar: f64,                  // Risk-of-Harm ∈ [0,1], must remain ≤0.3
    pub rod_scalar: f64,                  // Risk-of-Danger ∈ [0,1], orthogonal to RoH
    pub pain_envelope_score: f64,         // normalized pain-debt index
    pub eco_impact_score: f64,            // eco cost of ongoing nanoswarm duty (bounded)
    pub neurorights_bound_ok: bool,       // true only if neurorights envelopes are satisfied
    pub jurisdiction: Jurisdiction,
}

/// Organ corridor scores derived from the snapshot.
/// These are host-local metrics, used only to gate nanoswarm / evolution intensity.
#[derive(Debug, Clone)]
pub struct OrganCorridorScores {
    pub renal_score: f64,                 // 0..100, higher is better (AKI risk low)
    pub cardiac_score: f64,               // 0..100, higher is better (myocardial strain low)
    pub microvascular_recovery_min: u16,  // expected time-to-recovery post duty episode
    pub lifeforce_stability_index: u32,   // integerized stability metric
}

impl OrganCorridorScores {
    pub fn from_snapshot(s: &OrganicThermalInflammationSnapshot) -> Self {
        // Renal corridor: creatinine deltas and RoH/ROD scaling
        // Based on KDIGO 0.3 mg/dL AKI thresholds, scaled to 0..100 band. [file:5]
        let renal_penalty = (s.renal_creatinine_delta_mg_dl * 280.0).clamp(0.0, 45.0);
        let renal_score = (100.0 - renal_penalty).max(0.0);

        // Cardiac corridor: hscTnI 3–5 ng/L transient increases acceptable, higher penalized. [file:5]
        let cardiac_penalty = (s.hs_troponin_i_ng_l * 0.9).clamp(0.0, 38.0);
        let cardiac_score = (100.0 - cardiac_penalty).max(0.0);

        // Microvascular recovery: crude mapping from RoH/ROD and eco stress to minutes.
        // Higher risk → longer enforced recovery window before high-intensity evolution. [file:27]
        let base_recovery = 15u16;
        let roh_extra = (s.roh_scalar * 40.0) as u16;
        let rod_extra = (s.rod_scalar * 30.0) as u16;
        let eco_extra = (s.eco_impact_score.max(0.0).min(1.0) * 20.0) as u16;
        let microvascular_recovery_min = base_recovery + roh_extra + rod_extra + eco_extra;

        // Lifeforce stability index: combine HRV, lifeforce scalar, and pain envelope. [file:15][file:27]
        let hrv_component = (s.hrv_rmssd_ms.max(0.0) * 1.5) as u32;
        let lf_component = (s.lifeforce_scalar.max(0.0).min(1.0) * 200.0) as u32;
        let pain_penalty = (s.pain_envelope_score.max(0.0).min(1.0) * 120.0) as u32;
        let lifeforce_stability_index = hrv_component + lf_component - pain_penalty;

        OrganCorridorScores {
            renal_score,
            cardiac_score,
            microvascular_recovery_min,
            lifeforce_stability_index,
        }
    }

    /// Core corridor check: used to decide if **high-intensity** nanoswarm / neuromorph
    /// evolution is allowed right now. Lower-intensity maintenance can be handled separately.
    pub fn corridors_ok_for_high_intensity(&self, s: &OrganicThermalInflammationSnapshot) -> bool {
        // Thermal + inflammation ceilings (host-centric, not kiosk-centric). [file:5][file:9]
        let thermo_ok =
            s.core_temp_f < 102.2 &&
            s.cranial_delta_t_c < 1.1 &&
            s.il6_pg_ml < 15.0 &&
            s.crp_mg_l < 8.0 &&
            s.tnf_alpha_pg_ml < 40.0;

        // Organ-level scores must remain in the green band.
        let organ_ok =
            self.renal_score > 62.0 &&
            self.cardiac_score > 68.0;

        // Lifeforce and neurorights must both be satisfied; RoH/ROD bounded. [file:15][file:27][file:6]
        let sovereignty_ok =
            s.lifeforce_scalar > 0.35 &&
            self.lifeforce_stability_index > 120 &&
            s.roh_scalar <= 0.30 &&
            s.rod_scalar <= 0.30 &&
            s.neurorights_bound_ok;

        thermo_ok && organ_ok && sovereignty_ok
    }

    /// Returns an evolution intensity lane recommendation for the current snapshot.
    /// This **never** downgrades the augmentation permanently; it only selects
    /// how aggressive the current nanoswarm / neuromorph episode may be.
    pub fn recommend_intensity_lane(&self, s: &OrganicThermalInflammationSnapshot) -> IntensityLane {
        if !self.corridors_ok_for_high_intensity(s) {
            // If corridors are stressed, but RoH/ROD remain ≤0.3, we move to maintenance-only.
            if s.roh_scalar <= 0.30 && s.rod_scalar <= 0.30 {
                IntensityLane::MaintenanceOnly {
                    required_recovery_min: self.microvascular_recovery_min,
                }
            } else {
                // If RoH or ROD exceeded, we enforce MEDICAL_HOLD:
                // no evolution, only detox/monitoring, until envelopes normalize. [file:9][file:27]
                IntensityLane::MedicalHold {
                    required_recovery_min: self.microvascular_recovery_min.max(60),
                }
            }
        } else {
            IntensityLane::HighIntensityAllowed {
                recommended_recovery_min: self.microvascular_recovery_min,
            }
        }
    }
}

/// Lane classification for nanoswarm / neuromorph evolution intensity.
#[derive(Debug, Clone, PartialEq)]
pub enum IntensityLane {
    /// Full evolution kernels allowed within current envelopes.
    HighIntensityAllowed {
        recommended_recovery_min: u16,
    },
    /// Only low-intensity maintenance / detox episodes allowed.
    MaintenanceOnly {
        required_recovery_min: u16,
    },
    /// No actuation; observation + detox only until envelopes normalize.
    MedicalHold {
        required_recovery_min: u16,
    },
}

/// Guard struct that binds OrganicThermalInflammationSnapshot, OrganCorridorScores,
/// and jurisdictional neurorights into a single host-local decision surface.
/// There is NO routing to kiosks or external nodes here.
#[derive(Debug)]
pub struct OrganicHostThermalGuard {
    pub host_did: String,
    pub jurisdiction: Jurisdiction,
    pub strictest_neurorights: bool,
}

impl OrganicHostThermalGuard {
    pub fn new(host_did: String, jurisdiction: Jurisdiction) -> Self {
        OrganicHostThermalGuard {
            host_did,
            jurisdiction,
            strictest_neurorights: true, // strictest-wins for augmented-citizen neurorights. [file:6][file:8]
        }
    }

    /// Enforce neurorights profiles (SB1223/HB24-1058 + stricter) purely as **local** filters. [file:6]
    fn neurorights_profile_allows_eval(&self, snapshot: &OrganicThermalInflammationSnapshot) -> bool {
        if !snapshot.neurorights_bound_ok {
            return false;
        }
        match self.jurisdiction {
            Jurisdiction::California | Jurisdiction::Colorado | Jurisdiction::Montana => {
                // Neural data is treated as sensitive personal info; we keep all logic host-local.
                // External export of raw biosignals is forbidden; only scalar envelopes may be logged. [file:6]
                self.strictest_neurorights
            }
            Jurisdiction::Arizona | Jurisdiction::MultiState => {
                // Use the same strict profile to avoid weakening protections.
                self.strictest_neurorights
            }
        }
    }

    /// Primary guard entry: given a snapshot, returns an IntensityLane describing what the
    /// nanoswarm / neuromorph schedulers are allowed to do **on this host**, at this time.
    pub fn evaluate_snapshot(&self, snapshot: &OrganicThermalInflammationSnapshot) -> IntensityLane {
        if !self.neurorights_profile_allows_eval(snapshot) {
            // If neurorights are not satisfied, default to MedicalHold with long dwell. [file:6][file:8]
            return IntensityLane::MedicalHold {
                required_recovery_min: 120,
            };
        }

        let scores = OrganCorridorScores::from_snapshot(snapshot);
        scores.recommend_intensity_lane(snapshot)
    }

    /// Compact textual status for local dashboards or host-only audit logs.
    pub fn get_host_status(&self, snapshot: &OrganicThermalInflammationSnapshot) -> String {
        let scores = OrganCorridorScores::from_snapshot(snapshot);
        let lane = scores.recommend_intensity_lane(snapshot);
        format!(
            "Host DID: {} | Jurisdiction: {:?} | Core T: {:.2}°F | IL-6: {:.1} pg/mL | Renal score: {:.1} | Cardiac score: {:.1} | Lifeforce idx: {} | Lane: {:?}",
            self.host_did,
            self.jurisdiction,
            snapshot.core_temp_f,
            snapshot.il6_pg_ml,
            scores.renal_score,
            scores.cardiac_score,
            scores.lifeforce_stability_index,
            lane
        )
    }
}

/// Example host-local loop (to be wired into OrganicCPU scheduler, not public kiosks).
pub fn main_organic_thermal_guard_loop() {
    let host_did = "did:bostrom:organic-host-01".to_string();
    let guard = OrganicHostThermalGuard::new(host_did.clone(), Jurisdiction::Arizona);

    // For demonstration: synthetic snapshot reflecting a Phoenix heat episode, but
    // derived purely from host-integrated sensors. [file:5]
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let snapshot = OrganicThermalInflammationSnapshot {
        host_did,
        timestamp_ms: now_ms,
        core_temp_f: 100.8,
        cranial_delta_t_c: 0.6,
        il6_pg_ml: 9.4,
        crp_mg_l: 5.2,
        tnf_alpha_pg_ml: 28.0,
        renal_creatinine_delta_mg_dl: 0.12,
        hs_troponin_i_ng_l: 4.1,
        hrv_rmssd_ms: 42.0,
        lifeforce_scalar: 0.68,
        roh_scalar: 0.22,
        rod_scalar: 0.18,
        pain_envelope_score: 0.32,
        eco_impact_score: 0.21,
        neurorights_bound_ok: true,
        jurisdiction: Jurisdiction::Arizona,
    };

    let lane = guard.evaluate_snapshot(&snapshot);
    println!("{}", guard.get_host_status(&snapshot));
    println!("Recommended evolution lane: {:?}", lane);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_intensity_lane_when_corridors_safe() {
        let guard = OrganicHostThermalGuard::new(
            "did:bostrom:test-host".to_string(),
            Jurisdiction::Arizona,
        );
        let s = OrganicThermalInflammationSnapshot {
            host_did: "did:bostrom:test-host".to_string(),
            timestamp_ms: 0,
            core_temp_f: 99.4,
            cranial_delta_t_c: 0.3,
            il6_pg_ml: 6.0,
            crp_mg_l: 3.0,
            tnf_alpha_pg_ml: 22.0,
            renal_creatinine_delta_mg_dl: 0.05,
            hs_troponin_i_ng_l: 3.0,
            hrv_rmssd_ms: 50.0,
            lifeforce_scalar: 0.8,
            roh_scalar: 0.18,
            rod_scalar: 0.16,
            pain_envelope_score: 0.12,
            eco_impact_score: 0.10,
            neurorights_bound_ok: true,
            jurisdiction: Jurisdiction::Arizona,
        };
        let lane = guard.evaluate_snapshot(&s);
        match lane {
            IntensityLane::HighIntensityAllowed { recommended_recovery_min } => {
                assert!(recommended_recovery_min >= 15);
            }
            _ => panic!("Expected HighIntensityAllowed lane"),
        }
    }

    #[test]
    fn test_maintenance_lane_when_organs_stressed_but_roh_bounded() {
        let guard = OrganicHostThermalGuard::new(
            "did:bostrom:test-host".to_string(),
            Jurisdiction::Arizona,
        );
        let s = OrganicThermalInflammationSnapshot {
            host_did: "did:bostrom:test-host".to_string(),
            timestamp_ms: 0,
            core_temp_f: 101.8,
            cranial_delta_t_c: 0.9,
            il6_pg_ml: 13.0,
            crp_mg_l: 7.5,
            tnf_alpha_pg_ml: 35.0,
            renal_creatinine_delta_mg_dl: 0.24,
            hs_troponin_i_ng_l: 6.0,
            hrv_rmssd_ms: 30.0,
            lifeforce_scalar: 0.5,
            roh_scalar: 0.28,
            rod_scalar: 0.24,
            pain_envelope_score: 0.45,
            eco_impact_score: 0.35,
            neurorights_bound_ok: true,
            jurisdiction: Jurisdiction::Arizona,
        };
        let lane = guard.evaluate_snapshot(&s);
        match lane {
            IntensityLane::MaintenanceOnly { required_recovery_min } => {
                assert!(required_recovery_min >= 30);
            }
            _ => panic!("Expected MaintenanceOnly lane"),
        }
    }

    #[test]
    fn test_medical_hold_when_roh_or_rod_exceeded_or_neurorights_fail() {
        let guard = OrganicHostThermalGuard::new(
            "did:bostrom:test-host".to_string(),
            Jurisdiction::California,
        );
        let s = OrganicThermalInflammationSnapshot {
            host_did: "did:bostrom:test-host".to_string(),
            timestamp_ms: 0,
            core_temp_f: 103.0,
            cranial_delta_t_c: 1.4,
            il6_pg_ml: 22.0,
            crp_mg_l: 10.0,
            tnf_alpha_pg_ml: 55.0,
            renal_creatinine_delta_mg_dl: 0.40,
            hs_troponin_i_ng_l: 10.0,
            hrv_rmssd_ms: 18.0,
            lifeforce_scalar: 0.22,
            roh_scalar: 0.36,
            rod_scalar: 0.34,
            pain_envelope_score: 0.72,
            eco_impact_score: 0.55,
            neurorights_bound_ok: true,
            jurisdiction: Jurisdiction::California,
        };
        let lane = guard.evaluate_snapshot(&s);
        match lane {
            IntensityLane::MedicalHold { required_recovery_min } => {
                assert!(required_recovery_min >= 60);
            }
            _ => panic!("Expected MedicalHold lane"),
        }

        // Neurorights violation also forces MedicalHold. [file:6][file:8]
        let s2 = OrganicThermalInflammationSnapshot { neurorights_bound_ok: false, ..s };
        let lane2 = guard.evaluate_snapshot(&s2);
        match lane2 {
            IntensityLane::MedicalHold { required_recovery_min } => {
                assert!(required_recovery_min >= 120);
            }
            _ => panic!("Expected MedicalHold lane on neurorights failure"),
        }
    }
}
