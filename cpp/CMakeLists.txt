# ============================================================================
# BUILD: Aletheion C++ Edge Computing Core
# PURPOSE: High-performance evidence kernel for edge nodes
# COMPLIANCE: GDPR, HIPAA, EU AI Act 2024, Neurorights Charter v1
# OWNER: did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
# ============================================================================

cmake_minimum_required(VERSION 3.20)
project(aletheion_edge_core
    VERSION 1.0.0
    DESCRIPTION "High-performance evidence kernel for Aletheion GOD-City edge nodes"
    LANGUAGES CXX
)

# Compliance Metadata
set(OWNER_DID "did:aln:bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7")
set(SAFETY_KERNEL_REF "VitalNetSafetyKernel:1.0.0")
set(NEURORIGHTS_POLICY "AugmentedHumanRights:v1")
set(MIN_EVIDENCE_COMPLETENESS 0.86)

# C++ Standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Compiler Warnings (Strict)
if(CMAKE_CXX_COMPILER_ID MATCHES "GNU|Clang")
    add_compile_options(
        -Wall
        -Wextra
        -Wpedantic
        -Werror
        -Wno-unused-parameter
        -fstack-protector-strong
    )
endif()

# Security Flags
add_compile_definitions(
    _FORTIFY_SOURCE=2
    ALETHEION_VERSION="1.0.0"
    OWNER_DID="${OWNER_DID}"
    SAFETY_KERNEL_REF="${SAFETY_KERNEL_REF}"
    NEURORIGHTS_POLICY="${NEURORIGHTS_POLICY}"
    MIN_EVIDENCE_COMPLETENESS=${MIN_EVIDENCE_COMPLETENESS}
)

# Find Dependencies
find_package(OpenSSL REQUIRED)
find_package(Threads REQUIRED)
find_package(nlohmann_json 3.11.0 REQUIRED)

# Source Files
set(SOURCES
    src/evidence_core.cpp
    src/row_ledger.cpp
    src/neurorights_guard.cpp
    src/biofield_monitor.cpp
    src/consciousness_preservation.cpp
)

set(HEADERS
    include/aletheion/evidence_core.hpp
    include/aletheion/row_ledger.hpp
    include/aletheion/neurorights_guard.hpp
    include/aletheion/biofield_monitor.hpp
    include/aletheion/consciousness_preservation.hpp
    include/aletheion/types.hpp
    include/aletheion/constants.hpp
)

# Library Target
add_library(aletheion_edge_core STATIC ${SOURCES} ${HEADERS})

target_include_directories(aletheion_edge_core
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:include>
)

target_link_libraries(aletheion_edge_core
    PUBLIC
        OpenSSL::Crypto
        OpenSSL::SSL
        Threads::Threads
        nlohmann_json::nlohmann_json
)

# Executable Target (for edge node deployment)
add_executable(aletheion_edge_node src/main.cpp)
target_link_libraries(aletheion_edge_node PRIVATE aletheion_edge_core)

# Test Target
enable_testing()
add_executable(aletheion_tests tests/test_evidence_core.cpp)
target_link_libraries(aletheion_tests PRIVATE aletheion_edge_core)
add_test(NAME AletheionTests COMMAND aletheion_tests)

# Installation
install(TARGETS aletheion_edge_core aletheion_edge_node
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
    RUNTIME DESTINATION bin
)

install(DIRECTORY include/ DESTINATION include)

# Package Configuration
include(CMakePackageConfigHelpers)
write_basic_package_version_file(
    "${CMAKE_CURRENT_BINARY_DIR}/AletheionEdgeCoreConfigVersion.cmake"
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY SameMajorVersion
)

configure_package_config_file(
    "${CMAKE_CURRENT_SOURCE_DIR}/cmake/AletheionEdgeCoreConfig.cmake.in"
    "${CMAKE_CURRENT_BINARY_DIR}/AletheionEdgeCoreConfig.cmake"
    INSTALL_DESTINATION lib/cmake/AletheionEdgeCore
)

install(FILES
    "${CMAKE_CURRENT_BINARY_DIR}/AletheionEdgeCoreConfig.cmake"
    "${CMAKE_CURRENT_BINARY_DIR}/AletheionEdgeCoreConfigVersion.cmake"
    DESTINATION lib/cmake/AletheionEdgeCore
)

# CPack for Packaging
set(CPACK_PACKAGE_NAME "aletheion-edge-core")
set(CPACK_PACKAGE_VERSION ${PROJECT_VERSION})
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "Aletheion GOD-City Edge Computing Core")
set(CPACK_PACKAGE_VENDOR "Aletheion GOD-City Contributors")
set(CPACK_PACKAGE_CONTACT "core@aletheion.city")
set(CPACK_RESOURCE_FILE_LICENSE "${CMAKE_CURRENT_SOURCE_DIR}/LICENSE")
include(CPack)
