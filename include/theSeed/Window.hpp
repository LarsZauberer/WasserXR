#pragma once

#include <GLFW/glfw3.h>
#include <memory>
namespace theSeed::core {
class Window {
  struct Impl;
  std::unique_ptr<Impl> pImpl;

public:
  Window();
  ~Window();

  int initialize();
  int create(std::string name, int width, int height);
  void terminate();

  GLFWwindow *get_window();
};
} // namespace theSeed::core
