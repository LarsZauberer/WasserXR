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
  ts_add_component(scene, window, "TS_Window", NULL);

  size_t camera = ts_add_entity(scene);
  ts_add_component(scene, camera, "TS_Camera", NULL);
  ts_add_component(scene, camera, "TS_Transform", NULL);

  TS_Transform *camera_transform =
      ts_entity_get_component(scene, camera, "TS_Transform");
  camera_transform->position[2] = 3.0F;

  size_t triangle = ts_add_entity(scene);
  ts_add_component(scene, triangle, "TS_Transform", NULL);
  ts_add_component(scene, triangle, "TS_Model", NULL);
  TS_Transform *triangle_transform =
      ts_entity_get_component(scene, triangle, "TS_Transform");
  triangle_transform->position[0] = 0.0F;
  triangle_transform->rotation[1] = 45.0F;
  triangle_transform->rotation[2] = 45.0F;

  TS_Model *triangle_model =
      ts_entity_get_component(scene, triangle, "TS_Model");
  triangle_model->model_name = ts_copy_char_ptr("models/cube.obj");
  triangle_model->shader_name = ts_copy_char_ptr("shaders/base");

  // size_t triangle2 = ts_add_entity(scene);
  // ts_add_component(scene, triangle2, "TS_Transform");
  // ts_add_component(scene, triangle2, "TS_Mesh");
  //
  // TS_Transform_t *triangle2_transform =
  //     ts_entity_get_component(scene, triangle2, "TS_Transform");
  // triangle2_transform->position[0] = -1.0f;

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
