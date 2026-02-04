#include "TheSeed/ecs/Scene.h"
#include <stdio.h>

int main() {
  TS_Scene_t *scene = ts_create_scene();

  size_t a = ts_add_entity(scene);
  size_t b = ts_add_entity(scene);

  ts_print_entities(scene);

  printf("Before Plugin loading\n");

  ts_print_plugins(scene);

  printf("Loading Plugin\n");

  int status = ts_load_plugin(scene, "build/libtheseed_components.so");

  printf("Status of Plugin loading: %d\n", status);
  ts_print_plugins(scene);

  printf("Before Component\n");
  ts_print_components(scene);

  printf("Creating Component\n");
  status = ts_add_component(scene, a, "TS_Transform");

  printf("Status of Transform Creation: %d\n", status);
  ts_print_components(scene);

  ts_destroy_scene(scene);
}
