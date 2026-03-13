cmake_minimum_required(VERSION 3.10)
set(CMAKE_C_COMPILER clang)
project(TheSeed LANGUAGES C)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CLANG_TIDY clang-tidy)

option(BUILD_DEBUG "Debug build" OFF)
option(TS_NO_ASSERTS "Asserts are removed" OFF)

add_compile_options(-std=c99 -Wall -Wextra -Wno-unused-parameter)

if (BUILD_DEBUG)
  add_compile_options(-O1 -g -fno-omit-frame-pointer)
  add_compile_options(-fsanitize=address -fsanitize=undefined)
  add_link_options(-fsanitize=address -fsanitize=undefined)
else()
  add_compile_options(-O3)
endif()

if (TS_NO_ASSERTS)
  add_compile_definitions(TS_NO_ASSERTS)
endif()

find_package(PkgConfig REQUIRED)
pkg_check_modules(GLIB REQUIRED glib-2.0)
pkg_check_modules(CGLM REQUIRED cglm)
pkg_check_modules(ASSIMP REQUIRED assimp)

# OpenGL
add_library(glad SHARED src/glad/gl.c)
target_include_directories(glad PUBLIC include)

# ECS
file(GLOB_RECURSE ECS_SRC src/TheSeed/ecs/*.c)
add_library(theseed_ecs SHARED ${ECS_SRC})
target_include_directories(theseed_ecs PUBLIC include)
target_include_directories(theseed_ecs PRIVATE ${GLIB_INCLUDE_DIRS} src/TheSeed/ecs)
target_link_libraries(theseed_ecs PRIVATE ${GLIB_LIBRARIES})

# Core
file(GLOB CORE_SRC src/TheSeed/core/*.c)
add_library(theseed_core SHARED ${CORE_SRC})
target_include_directories(theseed_core PUBLIC include)
target_include_directories(theseed_core PRIVATE ${GLIB_INCLUDE_DIRS} ${CGLM_INCLUDE_DIRS} ${ASSIMP_INCLUDE_DIRS})
target_link_libraries(theseed_core PRIVATE theseed_ecs glad glfw ${GLIB_LIBRARIES} ${CGLM_LIBRARIES} ${ASSIMP_LIBRARIES})

# Components
file(GLOB COMPONENTS_SRC src/TheSeed/components/*.c)
add_library(theseed_components SHARED ${COMPONENTS_SRC})
target_include_directories(theseed_components PUBLIC include)
target_link_libraries(theseed_components PRIVATE glad glfw theseed_core)

# Systems
file(GLOB SYSTEMS_SRC src/TheSeed/systems/*.c)
add_library(theseed_systems SHARED ${SYSTEMS_SRC})
target_include_directories(theseed_systems PUBLIC include ${GLIB_INCLUDE_DIRS})
target_link_libraries(theseed_systems PRIVATE glad glfw pthread ${GLIB_LIBRARIES})

# Generate main executable
add_executable(theseed src/main.c)
target_link_libraries(theseed PRIVATE theseed_core theseed_ecs)

install(TARGETS theseed
    RUNTIME DESTINATION bin
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
)

# Doxygen documentation
file(GLOB_RECURSE HEADER_FILES include/TheSeed/*.h)
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

add_executable(test_utils tests/test_utils.c)
target_include_directories(test_utils PRIVATE ${GLIB_INCLUDE_DIRS})
target_link_libraries(test_utils PRIVATE theseed_core ${GLIB_LIBRARIES})

add_executable(test_ecs_add_component tests/test_ecs_add_component.c)
target_include_directories(test_ecs_add_component PRIVATE ${GLIB_INCLUDE_DIRS})
target_link_libraries(test_ecs_add_component PRIVATE theseed_ecs theseed_core ${GLIB_LIBRARIES})

add_test(NAME test_utils COMMAND test_utils)
add_test(NAME test_ecs_add_component COMMAND test_ecs_add_component)
