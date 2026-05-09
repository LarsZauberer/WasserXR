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
  char **plugins = wxr_get_plugins(&count, input->scene);
  
  wxr_assert(count == input->expected_count,
            "Plugin count should match expected");
  
  if (count > 0 && plugins != NULL) {
    for (size_t i = 0; i < count; i++) {
      free(plugins[i]);
    }
    free(plugins);
  }
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *empty_scene = wxr_create_scene();

  WXR_Scene *scene_one_plugin = wxr_create_scene();
  wxr_assert(0 == wxr_load_plugin(scene_one_plugin, "./libwasserxr_test_components.so"),
            "Failed to load the plugin");

  TestCase cases[] = {
      {empty_scene, 0},
      {scene_one_plugin, 1},
  };

  // Constructing Tests
  for (size_t i = 0; i < 2; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_get_plugins/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
