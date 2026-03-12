#![no_std]

#[repr(C)]
pub struct NeuromorphicHint {
    pub tsn_enabled: u8,
    pub snn_backend: u8,
    pub power_budget_mw: u16,
    pub reserved: u16,
}
