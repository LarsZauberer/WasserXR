#include "WasserXR/ecs/Scene.h"
#include "WasserXR/ecs/logging.h"
#include "WasserXR/ecs/utils.h"

int main() {
  // Create the logging
  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  wxr_info("Logging Initialized!");

  // Create the ecs scene
  WXR_Scene *scene = wxr_create_scene();

#ifndef WXR_STATIC
  wxr_load_plugin(scene, "build/libwasserxr_core.so");
#endif

  WXR_Entity console = wxr_add_entity(scene);
  wxr_add_component(scene, console, "WXR_Console");

  WXR_Entity window = wxr_add_entity(scene);
  wxr_add_component(scene, window, "WXR_Window");

  WXR_Entity camera = wxr_add_entity(scene);
  wxr_add_component(scene, camera, "WXR_Camera");
  wxr_add_component(scene, camera, "WXR_Transform");

  // Add the systems
  wxr_add_system(scene, "wxr_console_system", 100);

  wxr_add_system(scene, "wxr_window_pre_renderer", 50);
  wxr_add_system(scene, "wxr_window_post_renderer", 150);

  wxr_add_system(scene, "wxr_window_quiter", 200);

  wxr_add_system(scene, "wxr_window_reloader", 100);

  wxr_add_system(scene, "wxr_mesh_renderer", 100);

  while (wxr_tick_scene(scene)) {
  }

  wxr_destroy_scene(scene);
}
