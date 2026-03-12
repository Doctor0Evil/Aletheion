# Aletheion Sensor Driver Build Configuration
# Status: Production | Security: PQ-Secure | Compliance: No-Python
cmake_minimum_required(VERSION 3.22)
project(Aletheion_Sensor_Drivers VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Rust Integration via Corrosion (No Python Toolchain)
include(FetchContent)
FetchContent_Declare(
Corrosion
GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
GIT_TAG v0.4.0
)
FetchContent_MakeAvailable(Corrosion)

# Import Rust Core Modules
corrosion_import_crate(MANIFEST_PATH ../Cargo.toml CRATES aletheion-sec aletheion-mob)

# C++ Sensor Drivers (Embedded Patterns)
add_library(sensor_drivers STATIC
src/drivers/temperature_sensor.cpp
src/drivers/humidity_sensor.cpp
src/drivers/airspace_radar.cpp
src/drivers/ffi_bridge.cpp
)

target_include_directories(sensor_drivers PUBLIC include)
target_link_libraries(sensor_drivers PRIVATE aletheion-sec aletheion-mob)

# LuaJIT Integration for Calibration (No Python)
find_package(LuaJIT REQUIRED)
target_link_libraries(sensor_drivers PRIVATE LuaJIT::LuaJIT)

# Build Constraints
if(PYTHON_FOUND)
message(FATAL_ERROR "Python is blacklisted in Aletheion build pipeline")
endif()

# Offline Capability
set(OFFLINE_BUILD TRUE)
set(BUFFER_HOURS 72)

# Install Targets
install(TARGETS sensor_drivers DESTINATION lib)
install(DIRECTORY include/ DESTINATION include)
