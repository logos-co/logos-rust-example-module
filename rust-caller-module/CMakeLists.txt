cmake_minimum_required(VERSION 3.14)
project(RustCallerModulePlugin LANGUAGES CXX)

if(DEFINED ENV{LOGOS_MODULE_BUILDER_ROOT})
    include($ENV{LOGOS_MODULE_BUILDER_ROOT}/cmake/LogosModule.cmake)
else()
    message(FATAL_ERROR "LogosModule.cmake not found. LOGOS_MODULE_BUILDER_ROOT is not set.")
endif()

configure_file(${CMAKE_CURRENT_SOURCE_DIR}/metadata.json
               ${CMAKE_CURRENT_BINARY_DIR}/metadata.json COPYONLY)

logos_module(
    NAME rust_caller_module
    INCLUDE_DIRS
        ${CMAKE_CURRENT_SOURCE_DIR}/lib
)

# ── Rust static library ───────────────────────────────────────────────────────
# Contains rust_caller_* functions + logos-rust-sdk rlib (inlined by Rust linker).
# logos_sdk_* symbols are unresolved here — satisfied below by logos_module_client.
find_library(LIBRUST_CALLER
    NAMES librust_caller.a rust_caller
    PATHS ${CMAKE_CURRENT_SOURCE_DIR}/lib
    NO_DEFAULT_PATH
)

if(NOT LIBRUST_CALLER)
    message(FATAL_ERROR
        "Rust static library (librust_caller.a) not found in ${CMAKE_CURRENT_SOURCE_DIR}/lib. "
        "Ensure the preConfigure hook ran 'cargo build --release'."
    )
endif()

message(STATUS "Found Rust caller static library: ${LIBRUST_CALLER}")
target_link_libraries(rust_caller_module_module_plugin PRIVATE ${LIBRUST_CALLER})

# ── logos-module-client ───────────────────────────────────────────────────────
# Resolves logos_sdk_* symbols declared in logos-rust-sdk's extern "C" blocks.
# LOGOS_MODULE_CLIENT_ROOT is set by the Nix build via cmakeFlags.
if(NOT DEFINED LOGOS_MODULE_CLIENT_ROOT OR LOGOS_MODULE_CLIENT_ROOT STREQUAL "")
    # Fall back to environment variable (set by Nix preConfigure hook)
    if(DEFINED ENV{LOGOS_MODULE_CLIENT_ROOT})
        set(LOGOS_MODULE_CLIENT_ROOT "$ENV{LOGOS_MODULE_CLIENT_ROOT}")
    else()
        message(FATAL_ERROR
            "LOGOS_MODULE_CLIENT_ROOT is not set. "
            "Pass -DLOGOS_MODULE_CLIENT_ROOT=<path> or set LOGOS_MODULE_CLIENT_ROOT env var."
        )
    endif()
endif()

get_filename_component(LOGOS_MODULE_CLIENT_ROOT "${LOGOS_MODULE_CLIENT_ROOT}" ABSOLUTE)
message(STATUS "Using logos-module-client at ${LOGOS_MODULE_CLIENT_ROOT}")

find_library(LOGOS_MODULE_CLIENT_LIBRARY
    NAMES logos_module_client
    PATHS "${LOGOS_MODULE_CLIENT_ROOT}/lib"
    NO_DEFAULT_PATH
)

if(NOT LOGOS_MODULE_CLIENT_LIBRARY)
    message(FATAL_ERROR "liblogos_module_client not found in ${LOGOS_MODULE_CLIENT_ROOT}/lib")
endif()

add_library(logos_module_client SHARED IMPORTED)
set_target_properties(logos_module_client PROPERTIES
    IMPORTED_LOCATION "${LOGOS_MODULE_CLIENT_LIBRARY}"
    INTERFACE_INCLUDE_DIRECTORIES "${LOGOS_MODULE_CLIENT_ROOT}/include"
)

target_link_libraries(rust_caller_module_module_plugin PRIVATE logos_module_client)

# ── Platform-specific Rust runtime deps ──────────────────────────────────────
if(APPLE)
    target_link_libraries(rust_caller_module_module_plugin PRIVATE
        "-framework CoreFoundation"
        "-framework Security"
    )
else()
    target_link_libraries(rust_caller_module_module_plugin PRIVATE pthread dl)
endif()
