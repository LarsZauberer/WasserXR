#include "TheSeed/ecs/logging.h"
#include "TheSeed/ecs/utils.h"
#include "TheSeed/ecs/Scene.h"
#include <stdio.h>

int main() {
  // Create the logging
  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  ts_info("Logging Initialized!");

  // Create the ecs scene
  TS_Scene *scene = ts_create_scene();

#ifndef TS_STATIC
  ts_load_plugin(scene, "build/libtheseed_components.so");
  ts_load_plugin(scene, "build/libtheseed_systems.so");
#endif

  TS_Entity console = ts_add_entity(scene);
  ts_add_component(scene, console, "TS_Console");

  TS_Entity window = ts_add_entity(scene);
  ts_add_component(scene, window, "TS_Window");

  TS_Entity camera = ts_add_entity(scene);
  ts_add_component(scene, camera, "TS_Camera");
  ts_add_component(scene, camera, "TS_Transform");

  // Add the systems
  ts_add_system(scene, "ts_console_system", 100);

  ts_add_system(scene, "ts_window_pre_renderer", 50);
  ts_add_system(scene, "ts_window_post_renderer", 150);

  ts_add_system(scene, "ts_window_quiter", 200);

  ts_add_system(scene, "ts_window_reloader", 100);

  ts_add_system(scene, "ts_mesh_renderer", 100);

  while (ts_tick_scene(scene)) {
  }

  ts_destroy_scene(scene);
}
