// ============================================================================
// HEADER: biofield_monitor.hpp
// PURPOSE: Biofield load monitoring for BCI safety
// COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
// OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
// ============================================================================

#ifndef ALETHEION_BIOFIELD_MONITOR_HPP
#define ALETHEION_BIOFIELD_MONITOR_HPP

#include "types.hpp"
#include "constants.hpp"
#include <string>
#include <map>
#include <mutex>
#include <chrono>

namespace aletheion {

/**
 * @brief Biofield Monitor for tracking and enforcing biofield load limits
 */
class BiofieldMonitor {
public:
    /**
     * @brief Initialize the biofield monitor
     * @return Result indicating success or error
     */
    Result initialize();

    /**
     * @brief Register a BCI device for monitoring
     * @param device_id Device ID
     * @param neuroclass Neuroclass type
     * @param load_ceiling Load ceiling (W/kg)
     * @return Result indicating success or error
     */
    Result registerDevice(
        const std::string& device_id,
        const std::string& neuroclass,
        double load_ceiling
    );

    /**
     * @brief Update current biofield load for a device
     * @param device_id Device ID
     * @param current_load Current load (W/kg)
     * @return Result indicating success or error
     */
    Result updateLoad(const std::string& device_id, double current_load);

    /**
     * @brief Check if load is within safe limits
     * @param device_id Device ID
     * @return true if load is safe
     */
    bool isLoadSafe(const std::string& device_id) const;

    /**
     * @brief Get current load for a device
     * @param device_id Device ID
     * @return double Current load or -1.0 if not found
     */
    double getCurrentLoad(const std::string& device_id) const;

    /**
     * @brief Get load ceiling for a device
     * @param device_id Device ID
     * @return double Load ceiling or -1.0 if not found
     */
    double getLoadCeiling(const std::string& device_id) const;

    /**
     * @brief Get all monitored devices
     * @return std::vector<std::string> List of device IDs
     */
    std::vector<std::string> getMonitoredDevices() const;

    /**
     * @brief Unregister a device
     * @param device_id Device ID
     * @return Result indicating success or error
     */
    Result unregisterDevice(const std::string& device_id);

private:
    struct DeviceInfo {
        std::string device_id;
        std::string neuroclass;
        double load_ceiling;
        double current_load;
        Timestamp last_update;
        bool is_safe;
    };

    std::map<std::string, DeviceInfo> devices_;
    mutable std::mutex mutex_;
};

} // namespace aletheion

#endif // ALETHEION_BIOFIELD_MONITOR_HPP
