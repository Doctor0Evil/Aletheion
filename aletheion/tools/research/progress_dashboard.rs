pub struct ResearchProgressDashboard {
    pub total_research_gaps: u32,
    pub resolved_gaps: u32,
    pub pending_gaps: u32,
    pub critical_blockers: u32,
    pub files_blocked: u32,
    pub files_ready: u32,
    pub fpic_consultations: Vec<FPICConsultation>,
    pub data_acquisitions: Vec<ResearchDataAcquisition>,
}

impl ResearchProgressDashboard {
    pub fn generate_summary_report(&self) -> Result<String, DashboardError>;
    pub fn calculate_completion_percentage(&self) -> f32;
    pub fn identify_critical_path(&self) -> Vec<ResearchGap>;
    pub fn export_to_registry(&self) -> Result<(), DashboardError>;
}
