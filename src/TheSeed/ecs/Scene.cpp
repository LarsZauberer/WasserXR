#include "TheSeed/ecs/Entity.hpp"
#include "TheSeed/ecs/ISystem.hpp"
#include <TheSeed/ecs/Scene.hpp>
#include <map>
#include <memory>
#include <optional>
#include <vector>

namespace TheSeed::ecs {

struct Scene::Impl {
  std::map<std::string, std::unique_ptr<ISystem>> systems;
  std::vector<std::unique_ptr<Entity>> entities;
};

Scene::Scene() { this->pimpl = std::make_unique<Impl>(); }
Scene::~Scene() {}

Entity *Scene::createEntity() {
  std::unique_ptr<Entity> e = std::make_unique<Entity>();
  this->pimpl->entities.push_back(std::move(e));
  size_t i = this->pimpl->entities.size();
  return this->pimpl->entities[i - 1].get();
}

bool Scene::addSystem(std::unique_ptr<ISystem> s) {
  if (this->pimpl->systems.contains(s->getId())) {
    return false;
  }
  this->pimpl->systems[s->getId()] = std::move(s);
  return true;
}

std::optional<ISystem *> Scene::getSystem(std::string s) {
  if (!this->pimpl->systems.contains(s)) {
    return {};
  }
  return this->pimpl->systems[s].get();
}

bool Scene::removeSystem(std::string s) {
  if (!this->pimpl->systems.contains(s)) {
    return false;
  }
  this->pimpl->systems.erase(s);
  return true;
}

void Scene::update() {
  for (const auto &[id, s] : this->pimpl->systems) {
    s->update();
  }
}

std::vector<Entity *> Scene::query(std::vector<std::string> s) {
  std::vector<Entity *> res;
  for (const auto &e : this->pimpl->entities) {
    bool found = false;
    for (const auto &i : s) {
      if (!e->getComponent(i).has_value()) {
        continue;
      }
      found = true;
    }
    if (found) {
      res.push_back(e.get());
    }
  }
  return res;
}

} // namespace TheSeed::ecs
