#include "TheSeed/components/Transform.h"
#include "TheSeed/ecs/Scene.h"
#include <stdio.h>

int main() {
  TS_Scene_t *scene = ts_create_scene();

  int status = ts_load_plugin(scene, "build/libtheseed_components.so");
  status = ts_load_plugin(scene, "build/libtheseed_systems.so");

  size_t window = ts_add_entity(scene);
  status = ts_add_component(scene, window, "TS_Window");

  size_t camera = ts_add_entity(scene);
  status = ts_add_component(scene, camera, "TS_Camera");
  status = ts_add_component(scene, camera, "TS_Transform");

  TS_Transform_t *camera_transform =
      ts_entity_get_component(scene, camera, "TS_Transform");
  camera_transform->position[2] = 3.0f;

  size_t triangle = ts_add_entity(scene);
  ts_add_component(scene, triangle, "TS_Transform");
  ts_add_component(scene, triangle, "TS_Mesh");
  TS_Transform_t *triangle_transform =
      ts_entity_get_component(scene, triangle, "TS_Transform");
  triangle_transform->position[0] = 1.0f;
  triangle_transform->rotation[2] = 180.0f;

  size_t triangle2 = ts_add_entity(scene);
  ts_add_component(scene, triangle2, "TS_Transform");
  ts_add_component(scene, triangle2, "TS_Mesh");

  TS_Transform_t *triangle2_transform =
      ts_entity_get_component(scene, triangle2, "TS_Transform");
  triangle2_transform->position[0] = -1.0f;

  // Add the systems
  status = ts_add_system(scene, "ts_window_pre_renderer", 50);
  status = ts_add_system(scene, "ts_window_post_renderer", 150);

  status = ts_add_system(scene, "ts_window_quiter", 100);

  status = ts_add_system(scene, "ts_window_reloader", 100);

  status = ts_add_system(scene, "ts_mesh_renderer", 100);

  while (1) {
    ts_tick_scene(scene);
  }
}
