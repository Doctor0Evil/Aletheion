#![no_std]

#[repr(C)]
pub struct IDEISSessionToken {
    pub session_id: [u8; 32],
    pub subject_did: [u8; 64],
    pub device_id: [u8; 32],
    pub issued_at_unix: i64,
    pub expires_at_unix: i64,
    pub jurisdiction: [u8; 16],
    pub policy_profile: [u8; 32],
    pub auth_context: [u8; 32],
}
