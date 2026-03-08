pub trait CorridorService {
    fn id(&self) -> &str;

    fn read_capitals(&self) -> CapitalSnapshot;        // 7-capital vector for this corridor
    fn read_demands(&self) -> DemandSnapshot;          // water, waste, cleaning, mobility loads

    fn propose_plan(&self, horizon_min: u32) -> Plan;  // pure, no side effects

    fn validate_plan(&self, plan: &Plan) -> PlanCheck; // SMART-chain + capital checks

    fn actuate(&self, plan: &Plan) -> ActuationResult; // only executes if PlanCheck::Safe
}
