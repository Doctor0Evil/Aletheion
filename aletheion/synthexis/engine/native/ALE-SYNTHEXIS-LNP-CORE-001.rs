// Native Rust stub for LightNoisePesticidePlanner.
// Provides a JSON-in / JSON-out FFI surface compatible with the Lua binding.
//
// ERM Layers: L2 (reads state), L4 (planning semantics).
// This is a scaffold; planning logic can be filled in with real constraints later.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct PlannerInput {
    date: String,
    base_state: serde_json::Value,
    species_activity: serde_json::Value,
    comfort_thresholds: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct PlannerOutput {
    envelopes: Vec<serde_json::Value>,
    violations: Vec<serde_json::Value>,
    meta: serde_json::Value,
}

fn compute_envelopes_internal(input: PlannerInput) -> PlannerOutput {
    // TODO: Implement real multi-objective planning respecting BioticTreaties
    // and human comfort thresholds. For now, echo a placeholder structure.
    PlannerOutput {
        envelopes: vec![serde_json::json!({
            "segment_id": "example-block-001",
            "lighting": {
                "max_lux": 5.0,
                "allowed_spectra": ["warm_white"],
                "quiet_hours": ["21:00-05:00"]
            },
            "noise": {
                "max_db": 45.0,
                "quiet_hours": ["21:00-07:00"]
            },
            "pesticide": {
                "allowed_windows": ["00:00-03:00"],
                "no_spray_zones": ["pollinator-corridor-7"]
            }
        })],
        violations: vec![],
        meta: serde_json::json!({
            "date": input.date,
            "status": "ok",
            "notes": "placeholder planner output"
        }),
    }
}

/// FFI entrypoint: accepts JSON string, returns JSON string.
/// The caller (Lua) is responsible for treating the returned pointer as read-only.
#[no_mangle]
pub extern "C" fn alethion_synthexis_lnp_compute(json_input: *const c_char) -> *const c_char {
    if json_input.is_null() {
        let out = PlannerOutput {
            envelopes: vec![],
            violations: vec![],
            meta: serde_json::json!({
                "error": "null_input"
            }),
        };
        let json = serde_json::to_string(&out).unwrap_or_else(|_| "{}".into());
        return CString::new(json).unwrap().into_raw();
    }

    let c_str = unsafe { CStr::from_ptr(json_input) };
    let input_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            let out = PlannerOutput {
                envelopes: vec![],
                violations: vec![],
                meta: serde_json::json!({ "error": "invalid_utf8" }),
            };
            let json = serde_json::to_string(&out).unwrap_or_else(|_| "{}".into());
            return CString::new(json).unwrap().into_raw();
        }
    };

    let parsed: Result<PlannerInput, _> = serde_json::from_str(input_str);
    let output = match parsed {
        Ok(p) => compute_envelopes_internal(p),
        Err(e) => PlannerOutput {
            envelopes: vec![],
            violations: vec![],
            meta: serde_json::json!({
                "error": "parse_failed",
                "detail": e.to_string()
            }),
        },
    };

    let json = serde_json::to_string(&output).unwrap_or_else(|_| "{}".into());
    CString::new(json).unwrap().into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_round_trip_smoke() {
        let input = serde_json::json!({
            "date": "2026-03-08",
            "base_state": {},
            "species_activity": {},
            "comfort_thresholds": {}
        });
        let s = serde_json::to_string(&input).unwrap();
        let c_in = CString::new(s).unwrap();
        let out_ptr = alethion_synthexis_lnp_compute(c_in.as_ptr());
        assert!(!out_ptr.is_null());
        let out_cstr = unsafe { CStr::from_ptr(out_ptr) };
        let out_str = out_cstr.to_str().unwrap();
        let parsed: PlannerOutput = serde_json::from_str(out_str).unwrap();
        assert_eq!(parsed.meta["status"], "ok");
    }
}
