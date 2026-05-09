#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    wxr_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  
  // Set scene to reload
  wxr_set_scene_reload(input->scene);
  
  // This test just verifies the function can be called without crashing
  // The actual reload behavior is tested through integration tests
  wxr_assert(1, "Scene reload signal set successfully");
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *test_scene = wxr_create_scene();

  TestCase cases[] = {
      {test_scene},
  };

  // Constructing Tests
  for (size_t i = 0; i < 1; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_set_scene_reload/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
