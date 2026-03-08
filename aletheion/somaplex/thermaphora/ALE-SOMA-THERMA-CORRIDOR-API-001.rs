pub struct DevicelessTraceKey {
    pub trace_id: String,          // matches DevicelessTrace.trace_id
    pub corridor_id: String,
    pub region_id: String,
    pub pattern_id: String,        // PatternIdentityRef.pattern_id
    pub somatic_envelope_id: Option<String>,
}

pub struct HeatRouteEnvelope {
    pub corridor_id: String,
    pub time_window: (u32, u32),   // minutes since midnight
    pub max_heat_cost: f64,        // Thermaphora budget
    pub max_somatic_cost: f64,     // Somaplex cost
    pub smart_chain_id: String,    // e.g. SMART05_SOMAPLEX_ROUTING
}

pub struct RouteRecommendation {
    pub trace_key: DevicelessTraceKey,
    pub envelope: HeatRouteEnvelope,
    pub path_segment_ids: Vec<String>, // graph segments only
    pub fpci_required: bool,
    pub fpci_satisfied: bool,
    pub neurorights_passed: bool,
    pub treaties_passed: bool,
}

pub trait SomaplexThermaphoraCorridorApi {
    fn propose_route_for_trace(
        &self,
        trace: DevicelessTraceKey
    ) -> RouteRecommendation;

    fn check_route_against_envelopes(
        &self,
        route: &RouteRecommendation
    ) -> bool; // true only if all budgets, treaties, neurorights hold
}
