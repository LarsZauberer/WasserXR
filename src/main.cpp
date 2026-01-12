#include "theSeed/Window.hpp"

#include <memory>

int main() {
  std::unique_ptr<theSeed::core::Window> window =
      std::make_unique<theSeed::core::Window>();

  window->initialize();
  window->create("TheSeed", 800, 600);

  while (!glfwWindowShouldClose(window->get_window())) {
    glfwSwapBuffers(window->get_window());
    glfwPollEvents();
  }

  return 0;
}
