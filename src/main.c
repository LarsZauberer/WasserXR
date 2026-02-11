#include "TheSeed/ecs/Scene.h"
#include <stdio.h>

int main() {
  TS_Scene_t *scene = ts_create_scene();

  int status = ts_load_plugin(scene, "build/libtheseed_components.so");
  status = ts_load_plugin(scene, "build/libtheseed_systems.so");

  size_t window = ts_add_entity(scene);
  status = ts_add_component(scene, window, "TS_Window");

  status = ts_add_system(scene, "ts_window_pre_renderer", 50);
  status = ts_add_system(scene, "ts_window_post_renderer", 150);

  status = ts_add_system(scene, "ts_window_quiter", 100);

  status = ts_add_system(scene, "ts_window_reloader", 100);

  status = ts_add_system(scene, "ts_mesh_renderer", 100);

  size_t triangle = ts_add_entity(scene);

  ts_add_component(scene, triangle, "TS_Transform");
  ts_add_component(scene, triangle, "TS_Mesh");

  while (1) {
    ts_tick_scene(scene);
  }
}
