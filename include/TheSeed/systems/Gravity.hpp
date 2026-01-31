#pragma once

#include "TheSeed/components/Transform.hpp"
#include "TheSeed/ecs/ISystem.hpp"
#include <iostream>

namespace TheSeed::systems {
class Gravity : public ecs::ISystem {
public:
  using ecs::ISystem::ISystem;

  std::string getId() override { return "TheSeed::systems::Gravity"; }
  void update() override {
    auto entities = this->scene->query({"TheSeed::components::Transform"});
    for (const auto &e : entities) {
      auto transform_opt = e->getComponent("TheSeed::components::Transform");
      auto transform = transform_opt.value();
      auto transform_ptr = dynamic_cast<components::Transform *>(transform);
      transform_ptr->z -= 1.0;
    }
  }
};

class PrintTransform : public ecs::ISystem {
public:
  using ecs::ISystem::ISystem;

  std::string getId() override { return "TheSeed::systems::PrintTransform"; }
  void update() override {
    auto entities = this->scene->query({"TheSeed::components::Transform"});
    for (const auto &e : entities) {
      auto transform_opt = e->getComponent("TheSeed::components::Transform");
      auto transform = transform_opt.value();
      auto transform_ptr = dynamic_cast<components::Transform *>(transform);
      std::cout << "X: " << transform_ptr->x << ", Y: " << transform_ptr->y
                << ", Z: " << transform_ptr->z << std::endl;
    }
  }
};
} // namespace TheSeed::systems
