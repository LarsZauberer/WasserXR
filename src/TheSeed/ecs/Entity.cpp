#include "TheSeed/ecs/Entity.hpp"
#include "TheSeed/ecs/IComponent.hpp"
#include <map>
#include <memory>
#include <optional>

namespace TheSeed::ecs {

struct Entity::Impl {
  std::map<std::string, std::unique_ptr<IComponent>> components;
};

Entity::Entity() { this->pimpl = std::make_unique<Impl>(); }

Entity::~Entity() {}

bool Entity::addComponent(std::unique_ptr<IComponent> c) {
  if (this->pimpl->components.contains(c->getId())) {
    return false;
  }
  this->pimpl->components[c->getId()] = std::move(c);
  return true;
}

std::optional<IComponent *> Entity::getComponent(std::string s) {
  if (!this->pimpl->components.contains(s)) {
    return {};
  }
  return this->pimpl->components[s].get();
}

bool Entity::removeComponent(std::string s) {
  if (!this->pimpl->components.contains(s)) {
    return false;
  }
  this->pimpl->components.erase(s);
  return true;
}

} // namespace TheSeed::ecs
