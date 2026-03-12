#![no_std]

use crate::manifest::IDEISProjectManifest;
use crate::pos_nonfinancial::POSNonFinancialEvent;
use crate::session::IDEISSessionToken;
use crate::ideis_pos_smartcity_core::ideis_handle_pos_request;

#[repr(C)]
pub struct WorkflowRoute {
    pub id: [u8; 32],
    pub source_namespace: [u8; 32],
    pub target_namespace: [u8; 32],
    pub action: [u8; 32],
    pub qos: [u8; 8],
    pub tsn_required: u8,
    pub neuromorphic_hint: [u8; 32],
}

#[repr(C)]
pub struct WorkflowEnvelope {
    pub route: WorkflowRoute,
    pub token: IDEISSessionToken,
    pub manifest: IDEISProjectManifest,
    pub payload: POSNonFinancialEvent,
}

#[no_mangle]
pub extern "C" fn ideis_workflow_bus_route(env_ptr: *const WorkflowEnvelope) -> super::IdeisResponse {
    use core::str;

    if env_ptr.is_null() {
        return super::IdeisResponse {
            code: 99,
            len: 0,
            buf: [0u8; 256],
        };
    }

    let env = unsafe { &*env_ptr };
    let action_raw = &env.route.action;
    let n = action_raw.iter().position(|b| *b == 0).unwrap_or(action_raw.len());
    let action = match str::from_utf8(&action_raw[..n]) {
        Ok(s) => s,
        Err(_) => {
            return super::IdeisResponse {
                code: 99,
                len: 0,
                buf: [0u8; 256],
            }
        }
    };

    unsafe {
        ideis_handle_pos_request(
            &env.token,
            &env.manifest,
            action.as_ptr(),
            action.len() as u32,
            &env.payload,
        )
    }
}
