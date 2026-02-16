cmake_minimum_required(VERSION 3.10)
project(TheSeed LANGUAGES C)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_compile_options(-std=c99 -Wall -Wextra -O0 -g -fno-omit-frame-pointer)

find_package(PkgConfig REQUIRED)
pkg_check_modules(GLIB REQUIRED glib-2.0)
pkg_check_modules(CGLM REQUIRED cglm)

# OpenGL
add_library(glad SHARED src/glad/gl.c)
target_include_directories(glad PUBLIC include)

# Core
file(GLOB CORE_SRC src/TheSeed/core/*.c)
add_library(theseed_core SHARED ${CORE_SRC})
target_include_directories(theseed_core PUBLIC include)
target_include_directories(theseed_core PRIVATE ${GLIB_INCLUDE_DIRS} ${CGLM_LIBRARIES})
target_link_libraries(theseed_core PRIVATE glad glfw ${GLIB_LIBRARIES} ${CGLM_LIBRARIES})

# ECS
file(GLOB ECS_SRC src/TheSeed/ecs/*.c)
add_library(theseed_ecs SHARED ${ECS_SRC})
target_include_directories(theseed_ecs PUBLIC include)
target_include_directories(theseed_ecs PRIVATE ${GLIB_INCLUDE_DIRS})
target_link_libraries(theseed_ecs PRIVATE ${GLIB_LIBRARIES})

# Components
file(GLOB COMPONENTS_SRC src/TheSeed/components/*.c)
add_library(theseed_components SHARED ${COMPONENTS_SRC})
target_include_directories(theseed_components PUBLIC include)
target_link_libraries(theseed_components PRIVATE glad glfw)

# Systems
file(GLOB SYSTEMS_SRC src/TheSeed/systems/*.c)
add_library(theseed_systems SHARED ${SYSTEMS_SRC})
target_include_directories(theseed_systems PUBLIC include)
target_link_libraries(theseed_systems PRIVATE glad glfw)

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
