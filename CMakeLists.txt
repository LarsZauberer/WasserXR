cmake_minimum_required(VERSION 3.10)
set(CMAKE_C_COMPILER clang)
project(WasserXR LANGUAGES C)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CLANG_TIDY clang-tidy)

option(BUILD_DEBUG "Debug build" OFF)
option(WXR_NO_ASSERTS "Asserts are removed" OFF)
option(WXR_STATIC "Static Linking for Components and Systems" OFF)
option(WXR_TESTS "Build the Tests for TheSeed" ON)

add_compile_options(-std=c99 -Wall -Wextra -Wno-unused-parameter)
add_link_options(-rdynamic)

if (BUILD_DEBUG)
  add_compile_options(-O0 -g -fno-omit-frame-pointer)
  add_compile_options(-fsanitize=address -fsanitize=undefined)
  add_link_options(-fsanitize=address -fsanitize=undefined)
else()
  add_compile_options(-O3)
endif()

if (WXR_NO_ASSERTS)
  add_compile_definitions(WXR_NO_ASSERTS)
endif()

if (WXR_STATIC)
  add_compile_definitions(WXR_STATIC)
endif()

find_package(PkgConfig REQUIRED)
pkg_check_modules(GLIB REQUIRED glib-2.0)
pkg_check_modules(CGLM REQUIRED cglm)
pkg_check_modules(ASSIMP REQUIRED assimp)

# ECS
file(GLOB_RECURSE ECS_SRC src/WasserXR/ecs/*.c)
add_library(wasserxr_ecs STATIC ${ECS_SRC})
target_include_directories(wasserxr_ecs PUBLIC include)
target_include_directories(wasserxr_ecs PRIVATE src/WasserXR/ecs)
target_include_directories(wasserxr_ecs PRIVATE ${GLIB_INCLUDE_DIRS})
target_link_libraries(wasserxr_ecs PRIVATE ${GLIB_LIBRARIES})

# Core (merged Components and Systems with internal Shader, Mesh, Mesh_Data utilities)
file(GLOB CORE_SRC src/WasserXR/core/*.c src/WasserXR/core/internal/*.c)
if(WXR_STATIC)
  add_library(wasserxr_core STATIC ${CORE_SRC})
else()
  add_library(wasserxr_core SHARED ${CORE_SRC})
endif()
target_include_directories(wasserxr_core PUBLIC include)
target_include_directories(wasserxr_core PRIVATE src/WasserXR/core/internal)
target_include_directories(wasserxr_core PRIVATE ${GLIB_INCLUDE_DIRS} ${CGLM_INCLUDE_DIRS} ${ASSIMP_INCLUDE_DIRS})
target_link_libraries(wasserxr_core PRIVATE glfw pthread ${CGLM_LIBRARIES} m ${GLIB_LIBRARIES} ${ASSIMP_LIBRARIES} wasserxr_ecs)

# Generate main executable
add_executable(wasserxr src/main.c src/glad/gl.c)
target_link_libraries(wasserxr PRIVATE wasserxr_ecs)
if(WXR_STATIC)
  target_link_libraries(wasserxr PRIVATE -Wl,--whole-archive 
    wasserxr_core 
    -Wl,--no-whole-archive)
endif()

install(TARGETS wasserxr wasserxr_ecs wasserxr_core
    RUNTIME DESTINATION bin
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
)

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

# Tests
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
    add_executable(${test_name} tests/${test_name}.c src/glad/gl.c)
    target_include_directories(${test_name} PRIVATE ${GLIB_INCLUDE_DIRS})
    target_link_libraries(${test_name} PRIVATE wasserxr_ecs ${GLIB_LIBRARIES})
    add_test(NAME ${test_name} COMMAND ${test_name})
endfunction()

# Define tests (each test requires just one line now)
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
