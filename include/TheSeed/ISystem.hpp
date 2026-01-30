#pragma once

#include "TheSeed/Scene.hpp"

namespace TheSeed::ecs {

class Scene;

class ISystem {
  Scene *scene;

public:
  ISystem(Scene *scene) { this->scene = scene; }
  ~ISystem() = default;

  virtual std::string getId();
  virtual void update();
};
} // namespace TheSeed::ecs
