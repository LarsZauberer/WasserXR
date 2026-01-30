#pragma once

#include "TheSeed/IComponent.hpp"
#include <string>

namespace TheSeed::components {
class Transform : public ecs::IComponent {
public:
public:
  float x;
  float y;
  float z;

  std::string getId() override { return "TheSeed::components::Transform"; }
};
} // namespace TheSeed::components
