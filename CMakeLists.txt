cmake_minimum_required(VERSION 3.10)
project(TheSeed LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED True)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_C_COMPILER_LAUNCHER ccache)
set(CMAKE_CXX_COMPILER_LAUNCHER ccache)

add_compile_options(-Wall -Wextra -O0)

# Set global include directories
include_directories(include)

# OpenGL
add_library(glad STATIC src/glad/gl.c)

add_library(theSeed SHARED src/TheSeed/Window.cpp src/TheSeed/Entity.cpp src/TheSeed/Scene.cpp)
target_link_libraries(theSeed PRIVATE glad glfw)

# Generate main executable
add_executable(main src/main.cpp)
target_link_libraries(main PRIVATE theSeed glfw)

install(TARGETS main glad
    RUNTIME DESTINATION bin
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
)
