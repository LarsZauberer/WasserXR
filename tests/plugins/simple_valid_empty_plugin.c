#include <stddef.h>

struct Version {
  size_t major;
  size_t minor;
  size_t patch;
};

struct PluginDefinition {
  const char *name;
  struct Version engine_version;
  const void *components;
  size_t component_count;
};

const struct PluginDefinition wxr_plugin = {
  .name = "MyPlugin",
  .engine_version = {0, 2, 0},
  .components = NULL,
  .component_count = 0,
};
