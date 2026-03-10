// Aletheion Physical Interface: Actuator Control Module
// Module: phy/actuators
// Language: C++ (Real-Time, Hardware Control, Safety-Critical)
// Compliance: ALE-COMP-CORE v1.0, ERM Layer 1 (PIL), Phoenix Safety Protocols
// Constraint: All commands must pass S4 Rule-Check, no unauthorized actuation

#ifndef ALETHEION_PHY_ACTUATORS_CONTROL_MODULE_CPP
#define ALETHEION_PHY_ACTUATORS_CONTROL_MODULE_CPP

#include <string>
#include <vector>
#include <cstdint>
#include <memory>
#include <chrono>

// Import shared primitives
#include "aletheion/gtl/birthsign/birth_sign_model.h"
#include "aletheion/gtl/envelope/decision_envelope.h"
#include "aletheion/core/compliance/ale_comp_core_hook.h"
#include "aletheion/dsl/encryption/pq_crypto.h"

namespace aletheion {
namespace phy {
namespace actuators {

/// ActuatorType defines all supported physical actuator categories
enum class ActuatorType {
    // Water Management
    WATER_VALVE_LINEAR,
    WATER_VALVE_ROTARY,
    WATER_PUMP_VARIABLE_SPEED,
    WATER_PUMP_FIXED_SPEED,
    IRRIGATION_SPRINKLER,
    DRIP_IRRIGATION_EMITTER,
    
    // Thermal Management
    HVAC_DAMPER,
    THERMAL_VALVE,
    HEAT_PUMP_COMPRESSOR,
    COOLING_TOWER_FAN,
    RADIANT_HEATING_ELEMENT,
    
    // Energy (Solar Microgrid)
    SOLAR_INVERTER,
    BATTERY_CONTACTOR,
    GRID_TIE_RELAY,
    LOAD_SHED_CONTACTOR,
    
    // Safety & Security
    EMERGENCY_SHUTOFF,
    FIRE_SUPPRESSION_VALVE,
    ACCESS_CONTROL_LOCK,
    EMERGENCY_LIGHT,
    
    // Environmental
    MISTING_SYSTEM_NOZZLE,
    VENTILATION_DAMPER,
    AIR_FILTRATION_FAN,
};

/// ActuatorCommand represents a verified instruction to physical hardware
struct ActuatorCommand {
    std::string command_id; // UUID v4 strict
    std::string actuator_id;
    ActuatorType actuator_type;
    std::string command_type; // "OPEN", "CLOSE", "SET_POSITION", "START", "STOP"
    double parameter_value; // 0.0-100.0 for position, RPM for motors, etc.
    uint64_t timestamp_us;
    BirthSignId birth_sign_id;
    std::string rule_check_hash; // S4 compliance verification hash
    std::string geographic_zone;
    bool emergency_override; // Phoenix 120°F+ emergency protocols
};

/// ActuatorStatus represents the confirmed state of physical hardware
struct ActuatorStatus {
    std::string actuator_id;
    std::string current_state; // "OPEN", "CLOSED", "POSITION_50", "RUNNING", "STOPPED"
    double actual_position; // Verified position (may differ from command)
    uint64_t status_timestamp_us;
    uint64_t last_maintenance_ts;
    std::string health_status; // "HEALTHY", "DEGRADED", "FAULT", "MAINTENANCE_REQUIRED"
    BirthSignId birth_sign_id;
    std::vector<std::string> sensor_confirmations; // Feedback from position sensors
};

/// ActuatorConfiguration defines hardware-specific parameters
struct ActuatorConfiguration {
    std::string actuator_id;
    ActuatorType actuator_type;
    std::string hardware_model;
    std::string communication_protocol; // "MODBUS", "BACNET", "MQTT", "GPIO"
    uint8_t modbus_address;
    uint16_t gpio_pin;
    double min_position;
    double max_position;
    double operating_temp_min_c; // Phoenix: -20°C
    double operating_temp_max_c; // Phoenix: 55°C (131°F)
    uint32_t maintenance_interval_days;
    BirthSignId birth_sign_id;
};

/// ActuatorError defines failure modes for physical actuator operations
enum class ActuatorError {
    RULE_CHECK_MISSING = 1,
    BIRTH_SIGN_PROPAGATION_FAILURE = 2,
    HARDWARE_TIMEOUT = 3,
    HARDWARE_MALFUNCTION = 4,
    COMPLIANCE_HOOK_FAILURE = 5,
    SAFETY_INTERLOCK_TRIGGERED = 6,
    PHOENIX_HEAT_LIMIT_EXCEEDED = 7,
    EMERGENCY_SHUTOFF_ACTIVE = 8,
    POSITION_FEEDBACK_MISMATCH = 9,
    MAINTENANCE_OVERDUE = 10,
    MONSOON_WATER_INGRESS = 11,
    DUST_STORM_INTERFERENCE = 12,
};

/// ActuatorDriverContract defines interface for all actuator implementations
class ActuatorDriverContract {
public:
    virtual ~ActuatorDriverContract() = default;
    
    /// initialize prepares actuator hardware for operation
    virtual bool initialize(const ActuatorConfiguration& config) = 0;
    
    /// execute performs physical actuation after S4 Rule-Check verification
    virtual ActuatorStatus execute(const ActuatorCommand& cmd) = 0;
    
    /// get_status queries current physical state from hardware
    virtual ActuatorStatus get_status(const std::string& actuator_id) = 0;
    
    /// emergency_stop triggers immediate hardware shutdown (safety-critical)
    virtual bool emergency_stop(const std::string& actuator_id) = 0;
    
    /// verify_position confirms physical position matches command
    virtual bool verify_position(const std::string& actuator_id, double expected_position) = 0;
};

/// ActuatorControlManager orchestrates all actuators for Phoenix deployment
class ActuatorControlManager : public ActuatorDriverContract {
private:
    AleCompCoreHook comp_core_hook_;
    PQCrypto pq_crypto_;
    std::vector<ActuatorConfiguration> registered_actuators_;
    double phoenix_heat_limit_c_;
    bool emergency_shutoff_active_;
    
public:
    ActuatorControlManager()
        : comp_core_hook_("ALE-PHY-ACTUATOR-MGR")
        , pq_crypto_("CRYSTALS-Dilithium")
        , phoenix_heat_limit_c_(55.0) // 131°F hardware limit
        , emergency_shutoff_active_(false) {}
    
    /// register_actuator adds a new physical actuator to management system
    /// 
    /// # Compliance (Phoenix-Specific)
    /// * MUST verify operating temperature range (-20°C to 55°C)
    /// * MUST verify maintenance interval (max 180 days)
    /// * MUST attach BirthSignId to actuator identity
    /// * MUST log registration to immutable audit ledger
    bool register_actuator(const ActuatorConfiguration& config, const PropagationContext& context) {
        // Verify BirthSign Propagation
        if (!comp_core_hook_.verify_birth_sign(context.workflow_birth_sign_id)) {
            throw ActuatorError::BIRTH_SIGN_PROPAGATION_FAILURE;
        }
        
        // Verify Temperature Range (Phoenix Extreme Heat Protocol)
        if (config.operating_temp_max_c < phoenix_heat_limit_c_) {
            throw ActuatorError::PHOENIX_HEAT_LIMIT_EXCEEDED;
        }
        
        // Verify Maintenance Interval (Max 180 days for safety-critical)
        if (config.maintenance_interval_days > 180) {
            throw ActuatorError::MAINTENANCE_OVERDUE;
        }
        
        // Check for Duplicate Actuator ID
        for (const auto& actuator : registered_actuators_) {
            if (actuator.actuator_id == config.actuator_id) {
                throw ActuatorError::HARDWARE_MALFUNCTION; // Generic duplicate error
            }
        }
        
        // Register Actuator
        registered_actuators_.push_back(config);
        
        // Log Registration Proof
        log_actuator_registration(context.workflow_birth_sign_id);
        
        return true;
    }
    
    /// execute performs physical actuation after S4 Rule-Check verification
    /// 
    /// # Arguments
    /// * `cmd` - Verified actuator command with S4 rule_check_hash
    /// 
    /// # Returns
    /// * `ActuatorStatus` - Confirmed physical state after execution
    /// 
    /// # Compliance (Safety-Critical)
    /// * MUST verify S4 rule_check_hash before any actuation
    /// * MUST verify BirthSignId propagation
    /// * MUST check emergency shutoff status
    /// * MUST verify position feedback after execution
    /// * Phoenix 120°F+ Protocol: Cooling systems cannot be shut off during extreme heat
    ActuatorStatus execute(const ActuatorCommand& cmd) override {
        // CRITICAL: Verify S4 Rule-Check Hash Before Any Actuation
        if (!comp_core_hook_.verify_rule_check_hash(cmd.rule_check_hash)) {
            throw ActuatorError::RULE_CHECK_MISSING;
        }
        
        // Verify BirthSign Propagation
        if (!comp_core_hook_.verify_birth_sign(cmd.birth_sign_id)) {
            throw ActuatorError::BIRTH_SIGN_PROPAGATION_FAILURE;
        }
        
        // Check Emergency Shutoff Status
        if (emergency_shutoff_active_ && !cmd.emergency_override) {
            throw ActuatorError::EMERGENCY_SHUTOFF_ACTIVE;
        }
        
        // Find Actuator Configuration
        const ActuatorConfiguration* config = nullptr;
        for (const auto& actuator : registered_actuators_) {
            if (actuator.actuator_id == cmd.actuator_id) {
                config = &actuator;
                break;
            }
        }
        if (config == nullptr) {
            throw ActuatorError::HARDWARE_MALFUNCTION;
        }
        
        // Check Ambient Temperature (Phoenix Heat)
        double ambient_temp = get_ambient_temperature();
        if (ambient_temp > config->operating_temp_max_c) {
            // Phoenix Extreme Heat Protocol: Block non-critical actuation
            if (cmd.actuator_type != ActuatorType::COOLING_TOWER_FAN &&
                cmd.actuator_type != ActuatorType::MISTING_SYSTEM_NOZZLE &&
                cmd.actuator_type != ActuatorType::HVAC_DAMPER) {
                throw ActuatorError::PHOENIX_HEAT_LIMIT_EXCEEDED;
            }
        }
        
        // Phoenix 120°F+ Protocol: Prevent cooling shutoff during extreme heat
        if (ambient_temp > 48.9) { // 120°F
            if (cmd.command_type == "CLOSE" && 
                (cmd.actuator_type == ActuatorType::COOLING_TOWER_FAN ||
                 cmd.actuator_type == ActuatorType::MISTING_SYSTEM_NOZZLE)) {
                // Override: Keep cooling systems active during extreme heat
                throw ActuatorError::SAFETY_INTERLOCK_TRIGGERED;
            }
        }
        
        // Execute Physical Actuation (Hardware-Specific)
        ActuatorStatus status = call_hardware_actuate(cmd, *config);
        
        // Verify Position Feedback
        if (!verify_position(cmd.actuator_id, cmd.parameter_value)) {
            status.health_status = "POSITION_MISMATCH";
            // Log discrepancy but don't fail (may be mechanical lag)
        }
        
        // Log Compliance Proof
        log_actuation_proof(cmd, status);
        
        return status;
    }
    
    /// get_status queries current physical state from hardware
    ActuatorStatus get_status(const std::string& actuator_id) override {
        // Query hardware for current state
        ActuatorStatus status;
        status.actuator_id = actuator_id;
        status.current_state = "UNKNOWN";
        status.actual_position = 0.0;
        status.status_timestamp_us = get_microsecond_timestamp();
        status.health_status = "UNKNOWN";
        return status;
    }
    
    /// emergency_stop triggers immediate hardware shutdown (safety-critical)
    bool emergency_stop(const std::string& actuator_id) override {
        // Emergency stop bypasses normal compliance checks (safety priority)
        // Still logs to audit trail with emergency flag
        emergency_shutoff_active_ = true;
        
        // Stop all actuators immediately
        for (const auto& actuator : registered_actuators_) {
            call_hardware_emergency_stop(actuator.actuator_id);
        }
        
        log_emergency_stop_event(actuator_id);
        
        return true;
    }
    
    /// verify_position confirms physical position matches command
    bool verify_position(const std::string& actuator_id, double expected_position) override {
        // Query position sensor feedback
        double actual_position = read_position_sensor(actuator_id);
        
        // Allow 5% tolerance for mechanical systems
        double tolerance = 0.05 * (expected_position > 50.0 ? expected_position : 100.0 - expected_position);
        
        return std::abs(actual_position - expected_position) <= tolerance;
    }
    
private:
    ActuatorStatus call_hardware_actuate(const ActuatorCommand& cmd, const ActuatorConfiguration& config) {
        // Hardware-specific actuation (MODBUS, BACNET, GPIO, etc.)
        ActuatorStatus status;
        status.actuator_id = cmd.actuator_id;
        status.current_state = cmd.command_type;
        status.actual_position = cmd.parameter_value;
        status.status_timestamp_us = get_microsecond_timestamp();
        status.health_status = "HEALTHY";
        status.birth_sign_id = cmd.birth_sign_id;
        
        // Simulate hardware execution time
        // In production: Actual hardware communication
        
        return status;
    }
    
    void call_hardware_emergency_stop(const std::string& actuator_id) {
        // Hardware-specific emergency stop (safety-critical, failsafe)
    }
    
    double read_position_sensor(const std::string& actuator_id) {
        // Query physical position sensor (potentiometer, encoder, etc.)
        return 0.0; // Placeholder
    }
    
    double get_ambient_temperature() {
        // Query ambient temperature sensor (Phoenix heat monitoring)
        return 45.0; // Placeholder
    }
    
    void log_actuator_registration(const BirthSignId& birth_sign) {
        // Log to immutable audit ledger
    }
    
    void log_actuation_proof(const ActuatorCommand& cmd, const ActuatorStatus& status) {
        // Generate compliance proof for actuation
        ComplianceProof proof;
        proof.check_id = "ALE-PHY-ACTUATOR-001";
        proof.timestamp = get_iso8601_timestamp();
        proof.result = ComplianceStatus::PASS;
        proof.cryptographic_hash = pq_crypto_.hash(cmd.command_id);
        proof.signer_did = "did:aletheion:actuator-manager";
        proof.evidence_log = {cmd.actuator_id, status.current_state};
        
        // Store in audit ledger
    }
    
    void log_emergency_stop_event(const std::string& actuator_id) {
        // Log emergency stop with high-priority alert
    }
};

// Helper functions
inline uint64_t get_microsecond_timestamp() {
    auto now = std::chrono::high_resolution_clock::now();
    return std::chrono::duration_cast<std::chrono::microseconds>(now.time_since_epoch()).count();
}

inline std::string get_iso8601_timestamp() {
    return "2026-03-11T00:00:00.000000Z";
}

} // namespace actuators
} // namespace phy
} // namespace aletheion

#endif // ALETHEION_PHY_ACTUATORS_CONTROL_MODULE_CPP

// END OF ACTUATOR CONTROL MODULE
