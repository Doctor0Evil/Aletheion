# LuaJIT Calibration API Specification
**Version:** 1.0.0 | **Status:** Production | **Compliance:** FPIC-Logged

## Overview
LuaJIT provides dynamic sensor calibration scripting without recompiling firmware. All calibration events are logged with FPIC consent metadata.

## FFI Interface Contract
```lua
-- Load C++ Sensor Driver
local sensor = ffi.load("libsensor_drivers.so")

-- Calibration Function Signature
-- @param sensor_id: uint32_t
-- @param offset: double
-- @param gain: double
-- @param fpic_token: string (Indigenous Consent Token)
function calibrate_sensor(sensor_id, offset, gain, fpic_token)
local result = sensor.calibrate(sensor_id, offset, gain, fpic_token)
if result ~= 0 then
error("Calibration Failed: " .. ffi.string(sensor.get_error()))
end
-- Log to Rust Core via FFI
sensor.log_calibration(sensor_id, os.time(), fpic_token)
end
```

## Data Sovereignty Requirements
1.  **FPIC Token:** Every calibration call must include a valid Indigenous Consent Token.
2.  **Logging:** All calibration events are immutably logged to the Aletheion ledger.
3.  **Privacy:** Sensor data is encrypted via Homomorphic Context before transmission.

## Security Constraints
-   **No Python:** LuaJIT only. Python interpreter is blacklisted.
-   **PQ-Secure:** All tokens are signed with Post-Quantum signatures (Dilithium).
-   **Offline Buffer:** Calibration logs are buffered for 72h if mesh is disconnected.

## Example Usage
```lua
local fpic_token = "PIIPAASH-CONSENT-2026-001"
calibrate_sensor(1, 0.5, 1.02, fpic_token)
```
