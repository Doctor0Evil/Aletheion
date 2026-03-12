#![no_std]

pub mod manifest;
pub mod session;
pub mod pos_nonfinancial;
pub mod dev_tunnel;
pub mod searchtrace;
pub mod compliance;
pub mod neuromorphic_hint;

use manifest::IDEISProjectManifest;
use pos_nonfinancial::POSNonFinancialEvent;
use session::IDEISSessionToken;
use compliance::{check_action_compliance, verify_manifest_integrity, validate_session_token};
use pos_nonfinancial::{safe_pos_record, swarm_aggregate_inventory};
use dev_tunnel::open_cross_repo_tunnel;
use searchtrace::immutable_searchtrace_log;

#[repr(C)]
pub struct IdeisResponse {
    pub code: i32,
    pub len: u32,
    pub buf: [u8; 256],
}

const RESP_OK: i32 = 0;
const RESP_BLOCKED: i32 = 10;
const RESP_UNKNOWN: i32 = 20;
const RESP_ERROR: i32 = 99;

fn write_resp(code: i32, msg: &str) -> IdeisResponse {
    let mut buf = [0u8; 256];
    let bytes = msg.as_bytes();
    let n = if bytes.len() > 256 { 256 } else { bytes.len() };
    buf[..n].copy_from_slice(&bytes[..n]);
    IdeisResponse {
        code,
        len: n as u32,
        buf,
    }
}

#[no_mangle]
pub extern "C" fn ideis_handle_pos_request(
    tok_ptr: *const IDEISSessionToken,
    manifest_ptr: *const IDEISProjectManifest,
    action_ptr: *const u8,
    action_len: u32,
    payload_ptr: *const POSNonFinancialEvent,
) -> IdeisResponse {
    if tok_ptr.is_null() || manifest_ptr.is_null() || action_ptr.is_null() || payload_ptr.is_null()
    {
        return write_resp(RESP_ERROR, "NULL_POINTER");
    }

    let tok = unsafe { &*tok_ptr };
    let manifest = unsafe { &*manifest_ptr };
    let payload = unsafe { &*payload_ptr };
    let action_slice = unsafe { core::slice::from_raw_parts(action_ptr, action_len as usize) };
    let action = match core::str::from_utf8(action_slice) {
        Ok(s) => s,
        Err(_) => return write_resp(RESP_ERROR, "INVALID_ACTION_UTF8"),
    };

    if let Err(_) = validate_session_token(tok, manifest) {
        return write_resp(RESP_BLOCKED, "INVALID_SESSION");
    }

    if let Err(_) = verify_manifest_integrity(manifest) {
        return write_resp(RESP_BLOCKED, "MANIFEST_TAMPERED");
    }

    if !check_action_compliance(manifest, action) {
        return write_resp(RESP_BLOCKED, "Action blocked by IDE-IS compliance engine.");
    }

    match action {
        "POS_SALE" => {
            if let Err(_) = safe_pos_record(manifest, payload) {
                return write_resp(RESP_ERROR, "POS_NONFINANCIAL_ERROR");
            }
            write_resp(RESP_OK, "POS_NONFINANCIAL_OK")
        }
        "INVENTORY_SYNC" => {
            if let Err(_) = swarm_aggregate_inventory(manifest, tok, payload) {
                return write_resp(RESP_ERROR, "INVENTORY_SYNC_ERROR");
            }
            write_resp(RESP_OK, "INVENTORY_SYNC_OK")
        }
        "DEV_TUNNEL_OPEN" => {
            if let Err(_) = open_cross_repo_tunnel(manifest, tok, payload) {
                return write_resp(RESP_ERROR, "DEV_TUNNEL_OPEN_ERROR");
            }
            write_resp(RESP_OK, "DEV_TUNNEL_OPEN_OK")
        }
        "SEARCHTRACE_AUDIT" => {
            if let Err(_) = immutable_searchtrace_log(manifest, payload) {
                return write_resp(RESP_ERROR, "SEARCHTRACE_AUDIT_ERROR");
            }
            write_resp(RESP_OK, "SEARCHTRACE_AUDIT_OK")
        }
        _ => write_resp(RESP_UNKNOWN, "UNKNOWN_ACTION"),
    }
}
