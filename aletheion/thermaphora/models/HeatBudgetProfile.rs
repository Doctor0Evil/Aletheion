// Thermaphora core schemas: HeatBudgetProfile (no algorithms).
// All engines must import from this crate instead of redefining structures.

#![allow(dead_code)]

pub mod thermaphora_models_v01 {

    /// HeatBudgetProfile encodes a per-person, per-interval heat balance
    /// using fields directly grounded in human heat-budget literature and
    /// Phoenix microclimate conditions.
    #[derive(Clone, Debug, PartialEq)]
    pub struct HeatBudgetProfile {
        /// Unique identifier for this profile instance (person, day, scenario).
        pub id: String,

        /// Metabolic rate (W/m^2).
        pub metabolic_rate_w_m2: f32,

        /// Clothing insulation (clo).
        pub clothing_insulation_clo: f32,

        /// Air temperature (°C) at the person’s location.
        pub air_temp_c: f32,

        /// Mean radiant temperature (°C).
        pub mean_radiant_temp_c: f32,

        /// Wind speed (m/s).
        pub wind_speed_ms: f32,

        /// Relative humidity (%).
        pub relative_humidity_pct: f32,

        /// Duration that this profile segment covers (seconds).
        pub duration_s: u32,

        /// Computed heat strain index (dimensionless, 0–1 band recommended).
        pub heat_strain_index: f32,

        /// Probability estimate that safe thresholds are exceeded in this segment.
        pub exceedance_prob: f32,
    }
}
