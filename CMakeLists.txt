cmake_minimum_required(VERSION 3.10)
project(TheSeed LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED True)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_compile_options(-Wall -Wextra -Wpedantic -O1 -g)

# Set global include directories
include_directories(include)

# OpenGL
add_library(glad STATIC src/glad/gl.c)

add_library(theSeed SHARED src/TheSeed/Window.cpp)
target_link_libraries(theSeed glad glfw)

# Generate main executable
add_executable(main src/main.cpp)
target_link_libraries(main glad glfw theSeed)

install(TARGETS main glad
    RUNTIME DESTINATION bin
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
)
