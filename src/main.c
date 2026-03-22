#include "TheSeed/components/Model.h"
#include "TheSeed/components/Transform.h"
#include "TheSeed/core/logging.h"
#include "TheSeed/core/utils.h"
#include "TheSeed/ecs/Scene.h"
#include <stdio.h>

int main() {
  // Create the logging
  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  ts_info("Logging Initialized!");

  // Create the ecs scene
  TS_Scene *scene = ts_create_scene();

  ts_load_plugin(scene, "build/libtheseed_components.so");
  ts_load_plugin(scene, "build/libtheseed_systems.so");

  size_t console = ts_add_entity(scene);
  ts_add_component(scene, console, "TS_Console");

  // Add the systems
  ts_add_system(scene, "ts_console_system", 100);

  ts_deserialize_scene_from_file(scene, "scenes/main.ts");

  while (ts_tick_scene(scene)) {
  }

  ts_destroy_scene(scene);
}
