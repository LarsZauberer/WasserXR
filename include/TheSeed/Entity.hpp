#pragma once

#include "TheSeed/IComponent.hpp"
#include <memory>
#include <optional>

namespace TheSeed::ecs {
class Entity {
  struct Impl;
  std::unique_ptr<Impl> pimpl;

public:
  Entity();
  ~Entity();

  bool addComponent(std::unique_ptr<IComponent>);
  std::optional<IComponent *> getComponent(std::string);
  bool removeComponent(std::string);
};
} // namespace TheSeed::ecs
