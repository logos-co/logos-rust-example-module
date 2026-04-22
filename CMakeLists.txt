cmake_minimum_required(VERSION 3.14)
project(RustExampleModulePlugin LANGUAGES CXX)

if(DEFINED ENV{LOGOS_MODULE_BUILDER_ROOT})
    include($ENV{LOGOS_MODULE_BUILDER_ROOT}/cmake/LogosModule.cmake)
else()
    message(FATAL_ERROR "LogosModule.cmake not found. LOGOS_MODULE_BUILDER_ROOT is not set.")
endif()

configure_file(${CMAKE_CURRENT_SOURCE_DIR}/metadata.json
               ${CMAKE_CURRENT_BINARY_DIR}/metadata.json COPYONLY)

logos_module(
    NAME rust_example_module
    INCLUDE_DIRS
        ${CMAKE_CURRENT_SOURCE_DIR}/lib
)

# Link the Rust static library
find_library(LIBRUST_EXAMPLE
    NAMES librust_example.a rust_example
    PATHS ${CMAKE_CURRENT_SOURCE_DIR}/lib
    NO_DEFAULT_PATH
)

if(LIBRUST_EXAMPLE)
    message(STATUS "Found Rust static library: ${LIBRUST_EXAMPLE}")
    target_link_libraries(rust_example_module_module_plugin PRIVATE ${LIBRUST_EXAMPLE})

    if(APPLE)
        # Rust standard library requires these frameworks on macOS
        target_link_libraries(rust_example_module_module_plugin PRIVATE
            "-framework CoreFoundation"
            "-framework Security"
        )
    else()
        # Rust standard library requires pthread and dl on Linux
        target_link_libraries(rust_example_module_module_plugin PRIVATE pthread dl)
    endif()
else()
    message(FATAL_ERROR
        "Rust static library (librust_example.a) not found in ${CMAKE_CURRENT_SOURCE_DIR}/lib. "
        "Ensure the preConfigure hook ran 'cargo build --release' and copied the output."
    )
endif()
