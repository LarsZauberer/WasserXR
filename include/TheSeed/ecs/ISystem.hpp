#pragma once

#include "TheSeed/ecs/Scene.hpp"

namespace TheSeed::ecs {

class Scene;

class ISystem {
protected:
  Scene *scene;

public:
  ISystem(Scene *scene) { this->scene = scene; }
  virtual ~ISystem() = default;

  virtual std::string getId() = 0;
  virtual void update() = 0;
};
} // namespace TheSeed::ecs
