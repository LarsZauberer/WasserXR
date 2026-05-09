#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>
#include <stdlib.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  size_t expected_count;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    wxr_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;

  size_t count = 0;
  char **systems = wxr_get_systems(&count, input->scene);

  wxr_assert(count == input->expected_count,
             "System count should match expected");

  if (count > 0 && systems != NULL) {
    for (size_t i = 0; i < count; i++) {
      free(systems[i]);
    }
    free(systems);
  }
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *empty_scene = wxr_create_scene();

  WXR_Scene *scene_one_system = wxr_create_scene();
  wxr_assert(
      0 == wxr_load_plugin(scene_one_system, "./libwasserxr_test_systems.so"),
      "Failed to load the plugin");
  wxr_add_system(scene_one_system, "wxr_system_a", 10);

  TestCase cases[] = {
      {empty_scene, 0},
      {scene_one_system, 1},
  };

  // Constructing Tests
  for (size_t i = 0; i < 2; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_get_systems/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
