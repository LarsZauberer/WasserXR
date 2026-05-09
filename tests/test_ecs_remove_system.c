#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  char *system_name;
  int expected_result;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    wxr_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;

  int result = wxr_remove_system(input->scene, input->system_name);
  wxr_assert(result == input->expected_result,
             "Remove system result should match expected");
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *null_scene = NULL;

  WXR_Scene *empty_scene = wxr_create_scene();

  WXR_Scene *scene_with_system = wxr_create_scene();
  wxr_assert(
      0 == wxr_load_plugin(scene_with_system, "./libwasserxr_test_systems.so"),
      "Failed to load the plugin");
  wxr_add_system(scene_with_system, "wxr_system_a", 10);

  WXR_Scene *scene_without_system = wxr_create_scene();
  wxr_assert(0 == wxr_load_plugin(scene_without_system,
                                  "./libwasserxr_test_systems.so"),
             "Failed to load the plugin");

  WXR_Scene *scene_with_system2 = wxr_create_scene();
  wxr_assert(
      0 == wxr_load_plugin(scene_with_system2, "./libwasserxr_test_systems.so"),
      "Failed to load the plugin");
  wxr_add_system(scene_with_system2, "wxr_system_a", 10);

  TestCase cases[] = {
      {null_scene, NULL, 1},                     // NULL scene
      {empty_scene, "", 1},                      // Empty scene
      {scene_with_system, "wxr_system_a", 0},    // Valid removal
      {scene_without_system, "wxr_system_a", 1}, // System not added
      {scene_with_system2, "NonExistent", 1},    // Non-existent system
  };

  // Constructing Tests
  for (size_t i = 0; i < 5; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_remove_system/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
