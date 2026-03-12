#![no_std]

use crate::manifest::IDEISProjectManifest;
use crate::session::IDEISSessionToken;

#[repr(C)]
pub struct POSNonFinancialEvent {
    pub event_id: [u8; 32],
    pub sku: [u8; 32],
    pub quantity: i32,
    pub actor_did: [u8; 64],
    pub device_id: [u8; 32],
    pub timestamp_unix: i64,
    pub policy_profile: [u8; 32],
}

pub fn safe_pos_record(
    _manifest: &IDEISProjectManifest,
    payload: &POSNonFinancialEvent,
) -> Result<(), ()> {
    if payload.quantity == 0 {
        return Err(());
    }
    Ok(())
}

pub fn swarm_aggregate_inventory(
    _manifest: &IDEISProjectManifest,
    _tok: &IDEISSessionToken,
    _payload: &POSNonFinancialEvent,
) -> Result<(), ()> {
    Ok(())
}
