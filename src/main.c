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

  size_t window = ts_add_entity(scene);
  ts_add_component(scene, window, "TS_Window");

  size_t camera = ts_add_entity(scene);
  ts_add_component(scene, camera, "TS_Camera");
  ts_add_component(scene, camera, "TS_Transform");

  TS_Transform *camera_transform =
      ts_entity_get_component(scene, camera, "TS_Transform");
  float camera_z = 3.0F;
  ts_set(scene, camera_transform, "z", &camera_z);

  size_t triangle = ts_add_entity(scene);
  ts_add_component(scene, triangle, "TS_Transform");
  ts_add_component(scene, triangle, "TS_Model");
  TS_Transform *triangle_transform =
      ts_entity_get_component(scene, triangle, "TS_Transform");
  float triangle_x = 0.0F;
  float triangle_ry = 45.0F;
  float triangle_rz = 45.0F;
  ts_set(scene, triangle_transform, "x", &triangle_x);
  ts_set(scene, triangle_transform, "ry", &triangle_ry);
  ts_set(scene, triangle_transform, "rz", &triangle_rz);

  TS_Model *triangle_model =
      ts_entity_get_component(scene, triangle, "TS_Model");
  ts_set(scene, triangle_model, "model_name", "models/cube.obj");
  ts_set(scene, triangle_model, "shader_name", "shaders/base");

  // Add the systems
  ts_add_system(scene, "ts_console_system", 100);

  ts_add_system(scene, "ts_window_pre_renderer", 50);
  ts_add_system(scene, "ts_window_post_renderer", 150);

  ts_add_system(scene, "ts_window_quiter", 100);

  ts_add_system(scene, "ts_window_reloader", 100);

  ts_add_system(scene, "ts_mesh_renderer", 100);

  while (1) {
    ts_tick_scene(scene);
  }

  ts_destroy_scene(scene);
}
