#include "TheSeed/ecs/Scene.h"
#include <stdio.h>

int main() {
  TS_Scene_t *scene = ts_create_scene();

  int status = ts_load_plugin(scene, "build/libtheseed_components.so");
  status = ts_load_plugin(scene, "build/libtheseed_systems.so");

  size_t window = ts_add_entity(scene);
  status = ts_add_component(scene, window, "TS_Window");

  status = ts_add_system(scene, "ts_window_renderer", 100);

  status = ts_add_system(scene, "ts_window_quiter", 1000);

  status = ts_add_system(scene, "ts_window_reloader", 0);

  while (1) {
    ts_tick_scene(scene);
  }
}
