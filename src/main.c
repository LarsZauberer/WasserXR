#include "TheSeed/ecs/Scene.h"
#include <stdio.h>

int main() {
  TS_Scene_t *scene = ts_create_scene();

  int status = ts_load_plugin(scene, "build/libtheseed_components.so");
  printf("Status: %d\n", status);
  status = ts_load_plugin(scene, "build/libtheseed_systems.so");
  printf("Status: %d\n", status);

  size_t window = ts_add_entity(scene);
  status = ts_add_component(scene, window, "TS_Window");
  printf("Status: %d\n", status);

  status = ts_add_system(scene, "ts_window_renderer", 1000);
  printf("Status: %d\n", status);

  status = ts_add_system(scene, "ts_window_quiter", 100);
  printf("Status: %d\n", status);

  status = ts_add_system(scene, "ts_window_reloader", 1010);
  printf("Status: %d\n", status);

  while (1) {
    ts_tick_scene(scene);
  }
}
