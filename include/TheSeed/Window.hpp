#pragma once

#include <GLFW/glfw3.h>
#include <memory>
namespace TheSeed::core {
class Window {
  struct Impl;
  std::unique_ptr<Impl> pImpl;

public:
  Window();
  ~Window();

  int initialize();
  int create(std::string name, int width, int height);
  void terminate();
  bool should_terminate();

  GLFWwindow *get_window();
};
} // namespace TheSeed::core
