#pragma once

#include "TheSeed/Entity.hpp"
#include "TheSeed/ISystem.hpp"
#include <memory>
#include <optional>
#include <vector>

namespace TheSeed::ecs {

class ISystem;

class Scene {
  struct Impl;
  std::unique_ptr<Impl> pimpl;

public:
  Scene();
  ~Scene();

  Entity *createEntity();
  std::vector<Entity *> query(std::vector<std::string>);

  bool addSystem(std::unique_ptr<ISystem>);
  std::optional<ISystem *> getSystem(std::string);
  bool removeSystem(std::string);

  void update();
};
} // namespace TheSeed::ecs
