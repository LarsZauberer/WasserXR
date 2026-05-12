cmake_minimum_required(VERSION 3.10)
set(CMAKE_C_COMPILER clang)
project(WasserXR LANGUAGES C)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CLANG_TIDY clang-tidy)

option(BUILD_DEBUG "Debug build" OFF)
option(WXR_TESTS "Build the Tests for WasserXR" ON)

add_compile_options(-std=c99 -Wall -Wextra -Wno-unused-parameter)
add_link_options(-rdynamic)

if (BUILD_DEBUG)
  add_compile_options(-O0 -g -fno-omit-frame-pointer)
  add_compile_options(-fsanitize=address -fsanitize=undefined)
  add_link_options(-fsanitize=address -fsanitize=undefined)
else()
  add_compile_options(-O3)
endif()

find_package(PkgConfig REQUIRED)
pkg_check_modules(GLIB REQUIRED glib-2.0)

# ECS
file(GLOB_RECURSE ECS_SRC src/WasserXR/ecs/*.c)
add_library(wasserxr_ecs STATIC ${ECS_SRC})
target_include_directories(wasserxr_ecs PUBLIC include)
target_include_directories(wasserxr_ecs PRIVATE src/WasserXR/ecs)
target_include_directories(wasserxr_ecs PRIVATE ${GLIB_INCLUDE_DIRS})
target_link_libraries(wasserxr_ecs PRIVATE ${GLIB_LIBRARIES})

# ===
# Install
# ===

include(GNUInstallDirs)

install(TARGETS wasserxr_ecs
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
)

# Install include directory
install(DIRECTORY include/ DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})

# Create the .pc file for pkg-config

configure_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/wasserxr_ecs.pc.in
    ${CMAKE_CURRENT_BINARY_DIR}/wasserxr_ecs.pc
    @ONLY
)

install(FILES
    ${CMAKE_CURRENT_BINARY_DIR}/wasserxr_ecs.pc
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/pkgconfig
)

# ===
# Documentation
# ===

# Doxygen documentation
file(GLOB_RECURSE HEADER_FILES include/WasserXR/*.h)
add_custom_command(
    OUTPUT ${CMAKE_BINARY_DIR}/docs/html/index.html ${CMAKE_BINARY_DIR}/docs/man/man3
    COMMAND doxygen ${CMAKE_SOURCE_DIR}/Doxyfile
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    DEPENDS ${HEADER_FILES}
    COMMENT "Generating HTML and man page documentation with Doxygen"
)

add_custom_target(docs ALL
    DEPENDS ${CMAKE_BINARY_DIR}/docs/html/index.html
)

# Install man pages
install(DIRECTORY ${CMAKE_BINARY_DIR}/docs/man/man3
        DESTINATION ${CMAKE_INSTALL_PREFIX}/share/man
        OPTIONAL)

# ===
# Tests
# ===

enable_testing()

# Extra Libraries for testing
add_library(wasserxr_test_components SHARED tests/component_test.c)
target_include_directories(wasserxr_test_components PUBLIC include)
target_include_directories(wasserxr_test_components PRIVATE ${GLIB_INCLUDE_DIRS})
target_link_libraries(wasserxr_test_components PRIVATE wasserxr_ecs ${GLIB_LIBRARIES})

add_library(wasserxr_test_systems SHARED tests/system_test.c)
target_include_directories(wasserxr_test_systems PUBLIC include)
target_include_directories(wasserxr_test_systems PRIVATE ${GLIB_INCLUDE_DIRS})
target_link_libraries(wasserxr_test_systems PRIVATE wasserxr_ecs ${GLIB_LIBRARIES})

# Helper function to simplify adding tests
# Usage: add_wasserxr_test(test_name)
# Assumes test file is located at tests/{test_name}.c
function(add_wasserxr_test test_name)
    add_executable(${test_name} tests/${test_name}.c)
    target_include_directories(${test_name} PRIVATE ${GLIB_INCLUDE_DIRS})
    target_link_libraries(${test_name} PRIVATE wasserxr_ecs ${GLIB_LIBRARIES})
    add_test(NAME ${test_name} COMMAND ${test_name})
endfunction()

# Define tests
if(WXR_TESTS)
  add_wasserxr_test(test_utils)

  add_wasserxr_test(test_ecs_add_entity)
  add_wasserxr_test(test_ecs_remove_entity)

  add_wasserxr_test(test_ecs_add_system)
  add_wasserxr_test(test_ecs_remove_system)

  add_wasserxr_test(test_ecs_add_component)
  add_wasserxr_test(test_ecs_remove_component)

  add_wasserxr_test(test_ecs_load_plugin)
  add_wasserxr_test(test_ecs_unload_plugin)

  add_wasserxr_test(test_ecs_get_plugins)
  add_wasserxr_test(test_ecs_get_entities)
  add_wasserxr_test(test_ecs_get_systems)
  add_wasserxr_test(test_ecs_get_components_of_entity)

  add_wasserxr_test(test_ecs_reset_scene)

  add_wasserxr_test(test_ecs_entity_get_component)

  add_wasserxr_test(test_ecs_tick_scene)
  add_wasserxr_test(test_ecs_set_scene_reload)
  add_wasserxr_test(test_ecs_set_scene_terminate)

  add_wasserxr_test(test_ecs_serialize_plugin)
  add_wasserxr_test(test_ecs_serialize_system)
  add_wasserxr_test(test_ecs_serialize_component)
  add_wasserxr_test(test_ecs_serialize_entity)
  add_wasserxr_test(test_ecs_serialize_scene)

  add_wasserxr_test(test_ecs_deserialize_plugin)
  add_wasserxr_test(test_ecs_deserialize_system)
  add_wasserxr_test(test_ecs_deserialize_component)
  add_wasserxr_test(test_ecs_deserialize_entity)
  add_wasserxr_test(test_ecs_deserialize_scene)
endif()
