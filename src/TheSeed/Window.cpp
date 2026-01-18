#include <glad/gl.h>

#include <GLFW/glfw3.h>
#include <TheSeed/Window.hpp>
#include <iostream>
#include <string>

namespace TheSeed::core {
struct Window::Impl {
  std::string name;

  GLFWwindow *window;
};

static void setViewport(GLFWwindow *window, int width, int height) {
  glViewport(0, 0, width, height);
}

Window::Window() : pImpl(std::make_unique<Impl>()) {}
Window::~Window() = default;

int Window::create(std::string name, int width, int height) {
  this->pImpl->name = std::move(name);

  this->pImpl->window =
      glfwCreateWindow(width, height, this->pImpl->name.c_str(), NULL, NULL);
  if (this->pImpl->window == NULL) {
    std::cerr << "Failed to create GLFW window" << std::endl;
    this->terminate();
    return -1;
  }

  glfwMakeContextCurrent(this->pImpl->window);

  if (!gladLoadGL(glfwGetProcAddress)) {
    std::cerr << "Failed to initialize GLAD" << std::endl;
    return -1;
  }

  // Width and height are set within the setViewport
  setViewport(this->pImpl->window, width, height);

  glfwSetFramebufferSizeCallback(this->pImpl->window, setViewport);

  return 0;
}

int Window::initialize() {
  glfwInit();
  glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
  glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
  glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

  return 0;
}

GLFWwindow *Window::get_window() { return this->pImpl->window; }

void Window::terminate() { glfwTerminate(); }

bool Window::should_terminate() {
  return glfwWindowShouldClose(this->get_window());
}

} // namespace TheSeed::core
