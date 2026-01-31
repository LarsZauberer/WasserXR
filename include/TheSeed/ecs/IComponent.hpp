#pragma once

#include <string>

namespace TheSeed::ecs {
class IComponent {
public:
  IComponent() = default;
  ~IComponent() = default;

  virtual std::string getId() = 0;
};
} // namespace TheSeed::ecs
