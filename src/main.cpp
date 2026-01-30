#include "TheSeed/Components/Transform.hpp"
#include "TheSeed/Scene.hpp"
#include "TheSeed/Systems/Gravity.hpp"
#include "TheSeed/Window.hpp"

#include <memory>

int main() {
  std::unique_ptr<TheSeed::core::Window> window =
      std::make_unique<TheSeed::core::Window>();

  window->initialize();
  window->create("TheSeed", 800, 600);

  TheSeed::ecs::Scene scene;

  auto e = scene.createEntity();
  e->addComponent(std::make_unique<TheSeed::components::Transform>());

  scene.addSystem(std::make_unique<TheSeed::systems::Gravity>(&scene));
  scene.addSystem(std::make_unique<TheSeed::systems::PrintTransform>(&scene));

  while (!glfwWindowShouldClose(window->get_window())) {
    glfwSwapBuffers(window->get_window());
    glfwPollEvents();

    scene.update();
  }

  return 0;
}
