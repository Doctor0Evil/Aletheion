// Thermaphora core schemas: MicroclimateField (no algorithms).

#![allow(dead_code)]

pub mod thermaphora_models_v01 {

    /// MicroclimateField describes the local environmental state for a
    /// specific point or small area in Phoenix.
    #[derive(Clone, Debug, PartialEq)]
    pub struct MicroclimateField {
        /// Spatial identifier (e.g., block, segment, or sensor ID).
        pub location_id: String,

        /// Fraction of time or area in shade (0.0–1.0).
        pub shade_fraction: f32,

        /// Surface albedo (0.0–1.0).
        pub albedo: f32,

        /// Surface temperature (°C).
        pub surface_temp_c: f32,

        /// Air temperature (°C) at ~2 m height.
        pub air_temp_c: f32,

        /// Relative humidity (%).
        pub relative_humidity_pct: f32,

        /// Wind speed (m/s).
        pub wind_speed_ms: f32,

        /// Tags for deployed interventions (trees, cool pavement, misters, etc.).
        pub intervention_tags: Vec<String>,
    }
}
