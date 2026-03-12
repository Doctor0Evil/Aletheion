#![no_std]

use crate::manifest::IDEISProjectManifest;
use crate::session::IDEISSessionToken;

pub const TARGET_COMPLIANCE_SCORE: f32 = 0.99;

pub fn validate_session_token(
    tok: &IDEISSessionToken,
    _m: &IDEISProjectManifest,
) -> Result<(), ()> {
    if tok.expires_at_unix <= tok.issued_at_unix {
        return Err(());
    }
    Ok(())
}

pub fn verify_manifest_integrity(_m: &IDEISProjectManifest) -> Result<(), ()> {
    Ok(())
}

pub fn check_action_compliance(_m: &IDEISProjectManifest, action: &str) -> bool {
    if action.contains("PAY")
        || action.contains("TOKEN")
        || action.contains("CREDIT")
        || action.contains("DEBIT")
        || action.contains("WAGER")
        || action.contains("BET")
    {
        return false;
    }
    true
}
