cmake_minimum_required(VERSION 3.10)
project(TheSeed LANGUAGES C)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_C_COMPILER_LAUNCHER ccache)
set(CMAKE_CXX_COMPILER_LAUNCHER ccache)

add_compile_options(-Wall -Wextra -O0 -g)

find_package(PkgConfig REQUIRED)
pkg_check_modules(GLIB REQUIRED glib-2.0)

# OpenGL
add_library(glad STATIC src/glad/gl.c)
target_include_directories(glad PUBLIC include)

# Core
file(GLOB CORE_SRC src/TheSeed/core/*.c)
add_library(theseed_core SHARED ${CORE_SRC})
target_include_directories(theseed_core PUBLIC include)
target_link_libraries(theseed_core PRIVATE glad glfw)

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
target_link_libraries(theseed_components PRIVATE)

# Systems
file(GLOB SYSTEMS_SRC src/TheSeed/systems/*.c)
add_library(theseed_systems SHARED ${SYSTEMS_SRC})
target_include_directories(theseed_systems PUBLIC include)
target_link_libraries(theseed_systems PRIVATE)

# Generate main executable
add_executable(theseed src/main.c)
target_link_libraries(theseed PRIVATE theseed_core theseed_ecs theseed_components theseed_systems glfw)

install(TARGETS theseed
    RUNTIME DESTINATION bin
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
)
