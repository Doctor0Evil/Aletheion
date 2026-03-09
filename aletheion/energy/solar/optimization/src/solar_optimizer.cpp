#ifndef SOLAR_OPTIMIZER_HPP
#define SOLAR_OPTIMIZER_HPP

#include <cstdint>
#include <cstddef>
#include <array>
#include <cmath>

constexpr uint32_t SOLAR_OPTIMIZER_VERSION = 20260310;
constexpr size_t MAX_SOLAR_ARRAYS = 1024;
constexpr size_t MAX_INVERTERS = 512;
constexpr size_t MAX_BATTERY_BANKS = 256;
constexpr size_t MAX_WEATHER_STATIONS = 128;
constexpr double PHOENIX_LATITUDE = 33.4484;
constexpr double PHOENIX_LONGITUDE = -112.0740;
constexpr double STANDARD_TEST_CONDITIONS_WM2 = 1000.0;
constexpr double CELL_TEMP_NOMINAL_C = 45.0;

enum class PanelType : uint8_t {
    MONOCRYSTALLINE = 0, POLYCRYSTALLINE = 1, THIN_FILM = 2,
    PERC = 3, BIFACIAL = 4, HETEROJUNCTION = 5
};

enum class MountingType : uint8_t {
    FIXED = 0, SINGLE_AXIS = 1, DUAL_AXIS = 2, TRACKING = 3
};

struct SolarArray {
    uint64_t array_id;
    uint32_t microgrid_id;
    PanelType panel_type;
    MountingType mounting_type;
    double rated_capacity_kw;
    double actual_output_kw;
    double efficiency_0_1;
    double tilt_angle_deg;
    double azimuth_angle_deg;
    double surface_area_m2;
    double panel_temperature_c;
    double irradiance_wm2;
    uint64_t installation_date_ns;
    uint64_t last_cleaning_ns;
    double degradation_pct;
    bool operational;
};

struct Inverter {
    uint64_t inverter_id;
    uint64_t array_id;
    double rated_capacity_kw;
    double current_output_kw;
    double efficiency_0_1;
    double input_voltage_v;
    double output_voltage_v;
    double frequency_hz;
    double temperature_c;
    uint64_t last_maintenance_ns;
    bool operational;
    bool grid_tied;
    bool island_capable;
};

struct BatteryBank {
    uint64_t bank_id;
    uint32_t microgrid_id;
    double rated_capacity_kwh;
    double current_soc_kwh;
    double max_charge_rate_kw;
    double max_discharge_rate_kw;
    double round_trip_efficiency;
    double cycle_count;
    double state_of_health_0_1;
    double temperature_c;
    uint64_t installation_date_ns;
    uint64_t last_cycle_ns;
    bool operational;
    bool charging;
    bool discharging;
};

struct WeatherStation {
    uint64_t station_id;
    double latitude;
    double longitude;
    double irradiance_wm2;
    double ambient_temperature_c;
    double wind_speed_ms;
    double wind_direction_deg;
    double humidity_pct;
    double cloud_cover_0_1;
    uint64_t last_reading_ns;
    bool operational;
};

struct SolarForecast {
    uint64_t forecast_id;
    uint64_t array_id;
    uint64_t forecast_time_ns;
    uint64_t created_ns;
    double predicted_irradiance_wm2;
    double predicted_output_kw;
    double confidence_0_1;
    double cloud_cover_0_1;
    double temperature_c;
};

class SolarMicrogridOptimizer {
private:
    uint64_t optimizer_id_;
    char city_code_[8];
    SolarArray arrays_[MAX_SOLAR_ARRAYS];
    size_t array_count_;
    Inverter inverters_[MAX_INVERTERS];
    size_t inverter_count_;
    BatteryBank batteries_[MAX_BATTERY_BANKS];
    size_t battery_count_;
    WeatherStation weather_stations_[MAX_WEATHER_STATIONS];
    size_t weather_station_count_;
    SolarForecast forecasts_[1024];
    size_t forecast_count_;
    double total_solar_capacity_kw_;
    double total_current_output_kw_;
    double total_battery_capacity_kwh_;
    double total_battery_soc_kwh_;
    uint64_t total_energy_generated_kwh_;
    uint64_t total_energy_stored_kwh_;
    uint64_t total_energy_discharged_kwh_;
    uint64_t optimization_cycles_;
    uint64_t audit_checksum_;
    uint64_t last_optimization_ns_;
    
    void UpdateAuditChecksum() {
        uint64_t sum = 0;
        sum ^= array_count_ * inverter_count_ * battery_count_;
        sum ^= total_solar_capacity_kw_ * 1000;
        sum ^= total_current_output_kw_ * 1000;
        sum ^= optimization_cycles_;
        for (size_t i = 0; i < array_count_; ++i) {
            sum ^= arrays_[i].array_id * static_cast<uint64_t>(arrays_[i].operational);
        }
        audit_checksum_ = sum;
    }
    
    double ComputeSolarIrradiance(double hour_of_day, double day_of_year, double cloud_cover) {
        const double solar_constant = 1361.0;
        const double declination = 23.45 * sin(2 * M_PI * (284 + day_of_year) / 365.0);
        const double latitude_rad = PHOENIX_LATITUDE * M_PI / 180.0;
        const double declination_rad = declination * M_PI / 180.0;
        const double hour_angle = (hour_of_day - 12.0) * 15.0 * M_PI / 180.0;
        const double solar_elevation = asin(sin(latitude_rad) * sin(declination_rad) +
                                           cos(latitude_rad) * cos(declination_rad) * cos(hour_angle));
        if (solar_elevation <= 0) return 0.0;
        const double atmospheric_loss = 0.7 + 0.3 * sin(solar_elevation);
        const double clear_sky_irradiance = solar_constant * atmospheric_loss * sin(solar_elevation);
        return clear_sky_irradiance * (1.0 - cloud_cover * 0.8);
    }
    
    double ComputePanelTemperature(double irradiance, double ambient_temp, double wind_speed) {
        return ambient_temp + (irradiance / 800.0) * (CELL_TEMP_NOMINAL_C - 20.0) * (1.0 - wind_speed * 0.05);
    }
    
public:
    SolarMicrogridOptimizer(uint64_t optimizer_id, const char* city_code, uint64_t init_ns)
        : optimizer_id_(optimizer_id), array_count_(0), inverter_count_(0),
          battery_count_(0), weather_station_count_(0), forecast_count_(0),
          total_solar_capacity_kw_(0.0), total_current_output_kw_(0.0),
          total_battery_capacity_kwh_(0.0), total_battery_soc_kwh_(0.0),
          total_energy_generated_kwh_(0), total_energy_stored_kwh_(0),
          total_energy_discharged_kwh_(0), optimization_cycles_(0),
          audit_checksum_(0), last_optimization_ns_(init_ns) {
        for (int i = 0; i < 8 && city_code[i] != '\0'; ++i) {
            city_code_[i] = city_code[i];
        }
    }
    
    bool RegisterSolarArray(const SolarArray& array) {
        if (array_count_ >= MAX_SOLAR_ARRAYS) return false;
        arrays_[array_count_] = array;
        total_solar_capacity_kw_ += array.rated_capacity_kw;
        array_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterInverter(const Inverter& inverter) {
        if (inverter_count_ >= MAX_INVERTERS) return false;
        inverters_[inverter_count_] = inverter;
        inverter_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterBatteryBank(const BatteryBank& battery) {
        if (battery_count_ >= MAX_BATTERY_BANKS) return false;
        batteries_[battery_count_] = battery;
        total_battery_capacity_kwh_ += battery.rated_capacity_kwh;
        total_battery_soc_kwh_ += battery.current_soc_kwh;
        battery_count_++;
        UpdateAuditChecksum();
        return true;
    }
    
    bool RegisterWeatherStation(const WeatherStation& station) {
        if (weather_station_count_ >= MAX_WEATHER_STATIONS) return false;
        weather_stations_[weather_station_count_] = station;
        weather_station_count_++;
        return true;
    }
    
    void OptimizeArrayOutput(uint64_t array_id, uint64_t now_ns) {
        for (size_t i = 0; i < array_count_; ++i) {
            if (arrays_[i].array_id == array_id && arrays_[i].operational) {
                double irradiance = arrays_[i].irradiance;
                double panel_temp = ComputePanelTemperature(irradiance, 35.0, 2.0);
                double temp_coefficient = -0.004;
                double temp_loss = 1.0 + temp_coefficient * (panel_temp - 25.0);
                double soiling_loss = 0.98;
                if (now_ns - arrays_[i].last_cleaning_ns > 259200000000000) {
                    soiling_loss = 0.90;
                }
                double degradation_factor = 1.0 - (arrays_[i].degradation_pct / 100.0);
                arrays_[i].actual_output_kw = arrays_[i].rated_capacity_kw *
                    (irradiance / STANDARD_TEST_CONDITIONS_WM2) *
                    arrays_[i].efficiency * temp_loss * soiling_loss * degradation_factor;
                arrays_[i].panel_temperature_c = panel_temp;
                total_current_output_kw_ = 0.0;
                for (size_t j = 0; j < array_count_; ++j) {
                    if (arrays_[j].operational) {
                        total_current_output_kw_ += arrays_[j].actual_output_kw;
                    }
                }
                UpdateAuditChecksum();
                return;
            }
        }
    }
    
    void OptimizeBatteryCharging(uint64_t now_ns, double grid_price_cents_kwh, double solar surplus_kw) {
        for (size_t i = 0; i < battery_count_; ++i) {
            if (!batteries_[i].operational) continue;
            double soc_pct = batteries_[i].current_soc_kwh / batteries_[i].rated_capacity_kwh;
            if (soc_pct < 0.9 && solar_surplus_kw > 0.0) {
                double charge_rate = fmin(batteries_[i].max_charge_rate_kw, solar_surplus_kw);
                batteries_[i].charging = true;
                batteries_[i].discharging = false;
                batteries_[i].current_soc_kwh += charge_rate * 0.1;
                batteries_[i].last_cycle_ns = now_ns;
                batteries_[i].cycle_count += 0.1;
            } else if (soc_pct > 0.2 && grid_price_cents_kwh > 15.0) {
                batteries_[i].charging = false;
                batteries_[i].discharging = true;
            } else {
                batteries_[i].charging = false;
                batteries_[i].discharging = false;
            }
        }
        total_battery_soc_kwh_ = 0.0;
        for (size_t i = 0; i < battery_count_; ++i) {
            total_battery_soc_kwh_ += batteries_[i].current_soc_kwh;
        }
        UpdateAuditChecksum();
    }
    
    SolarForecast CreateSolarForecast(uint64_t array_id, uint64_t forecast_time_ns, uint64_t now_ns) {
        SolarForecast forecast;
        forecast.forecast_id = forecast_count_;
        forecast.array_id = array_id;
        forecast.forecast_time_ns = forecast_time_ns;
        forecast.created_ns = now_ns;
        double hour_of_day = (forecast_time_ns / 3600000000000) % 24;
        double day_of_year = (forecast_time_ns / 86400000000000) % 365;
        double cloud_cover = 0.3;
        for (size_t i = 0; i < weather_station_count_; ++i) {
            if (weather_stations_[i].operational) {
                cloud_cover = weather_stations_[i].cloud_cover_0_1;
                break;
            }
        }
        forecast.predicted_irradiance_wm2 = ComputeSolarIrradiance(hour_of_day, day_of_year, cloud_cover);
        forecast.predicted_output_kw = 0.0;
        forecast.confidence_0_1 = 0.85;
        forecast.cloud_cover_0_1 = cloud_cover;
        forecast.temperature_c = 35.0;
        forecasts_[forecast_count_] = forecast;
        forecast_count_++;
        return forecast;
    }
    
    double ComputeSystemEfficiency() {
        if (total_solar_capacity_kw_ == 0.0) return 0.0;
        double array_efficiency = 0.0;
        size_t operational_arrays = 0;
        for (size_t i = 0; i < array_count_; ++i) {
            if (arrays_[i].operational) {
                array_efficiency += arrays_[i].efficiency;
                operational_arrays++;
            }
        }
        if (operational_arrays == 0) return 0.0;
        array_efficiency /= operational_arrays;
        double inverter_efficiency = 0.0;
        size_t operational_inverters = 0;
        for (size_t i = 0; i < inverter_count_; ++i) {
            if (inverters_[i].operational) {
                inverter_efficiency += inverters_[i].efficiency;
                operational_inverters++;
            }
        }
        if (operational_inverters > 0) {
            inverter_efficiency /= operational_inverters;
        }
        return array_efficiency * inverter_efficiency;
    }
    
    double ComputeBatteryUtilization() {
        if (total_battery_capacity_kwh_ == 0.0) return 0.0;
        return total_battery_soc_kwh_ / total_battery_capacity_kwh_;
    }
    
    struct OptimizerStatus {
        uint64_t optimizer_id;
        char city_code[8];
        size_t total_arrays;
        size_t operational_arrays;
        size_t total_inverters;
        size_t operational_inverters;
        size_t total_batteries;
        size_t operational_batteries;
        double total_solar_capacity_kw;
        double total_current_output_kw;
        double total_battery_capacity_kwh;
        double total_battery_soc_kwh;
        double system_efficiency;
        double battery_utilization;
        uint64_t total_energy_generated_kwh;
        uint64_t optimization_cycles;
        uint64_t last_optimization_ns;
        uint64_t last_update_ns;
    };
    
    OptimizerStatus GetStatus(uint64_t now_ns) {
        OptimizerStatus status;
        status.optimizer_id = optimizer_id_;
        for (int i = 0; i < 8; ++i) status.city_code[i] = city_code_[i];
        status.total_arrays = array_count_;
        status.operational_arrays = 0;
        for (size_t i = 0; i < array_count_; ++i) {
            if (arrays_[i].operational) status.operational_arrays++;
        }
        status.total_inverters = inverter_count_;
        status.operational_inverters = 0;
        for (size_t i = 0; i < inverter_count_; ++i) {
            if (inverters_[i].operational) status.operational_inverters++;
        }
        status.total_batteries = battery_count_;
        status.operational_batteries = 0;
        for (size_t i = 0; i < battery_count_; ++i) {
            if (batteries_[i].operational) status.operational_batteries++;
        }
        status.total_solar_capacity_kw = total_solar_capacity_kw_;
        status.total_current_output_kw = total_current_output_kw_;
        status.total_battery_capacity_kwh = total_battery_capacity_kwh_;
        status.total_battery_soc_kwh = total_battery_soc_kwh_;
        status.system_efficiency = ComputeSystemEfficiency();
        status.battery_utilization = ComputeBatteryUtilization();
        status.total_energy_generated_kwh = total_energy_generated_kwh_;
        status.optimization_cycles = optimization_cycles_;
        status.last_optimization_ns = last_optimization_ns_;
        status.last_update_ns = now_ns;
        return status;
    }
    
    bool VerifyAuditIntegrity() const {
        uint64_t sum = 0;
        sum ^= array_count_ * inverter_count_ * battery_count_;
        sum ^= total_solar_capacity_kw_ * 1000;
        sum ^= total_current_output_kw_ * 1000;
        sum ^= optimization_cycles_;
        for (size_t i = 0; i < array_count_; ++i) {
            sum ^= arrays_[i].array_id * static_cast<uint64_t>(arrays_[i].operational);
        }
        return sum == audit_checksum_;
    }
    
    void IncrementOptimizationCycle(uint64_t now_ns) {
        optimization_cycles_++;
        last_optimization_ns_ = now_ns;
        UpdateAuditChecksum();
    }
};

#endif
