#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  int should_terminate;
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
  
  if (input->should_terminate) {
    wxr_set_scene_terminate(input->scene);
  }
  
  int result = wxr_tick_scene(input->scene);
  wxr_assert(result == input->expected_result,
            "Tick scene result should match expected");
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *normal_scene = wxr_create_scene();

  WXR_Scene *terminate_scene = wxr_create_scene();

  TestCase cases[] = {
      {normal_scene, 0, 1},       // Normal tick
      {terminate_scene, 1, 0},    // Terminated tick
  };

  // Constructing Tests
  for (size_t i = 0; i < 2; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_tick_scene/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
