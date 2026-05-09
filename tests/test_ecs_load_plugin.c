#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  char *plugin_path;
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

  int result = wxr_load_plugin(input->scene, input->plugin_path);
  wxr_assert(result == input->expected_result,
            "Load plugin result should match expected");
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *null_scene = NULL;

  WXR_Scene *valid_scene = wxr_create_scene();

  WXR_Scene *valid_scene2 = wxr_create_scene();

  TestCase cases[] = {
      {null_scene, NULL, 1},                     // NULL scene
      {valid_scene, "", 1},                      // Empty path
      {valid_scene2, "./libwasserxr_core.so", 0}, // Valid plugin
  };

  // Constructing Tests
  for (size_t i = 0; i < 3; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_load_plugin/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
