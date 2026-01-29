#include "TheSeed/Window.hpp"

#include <memory>

int main() {
  std::unique_ptr<TheSeed::core::Window> window =
      std::make_unique<TheSeed::core::Window>();

  window->initialize();
  window->create("TheSeed", 800, 600);

  while (!glfwWindowShouldClose(window->get_window())) {
    glfwSwapBuffers(window->get_window());
    glfwPollEvents();
  }

  return 0;
}
