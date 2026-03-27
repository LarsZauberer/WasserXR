#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    ts_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  
  // Set scene to terminate
  ts_set_scene_terminate(input->scene);
  
  // Next tick should return 0 (terminate signal)
  int result = ts_tick_scene(input->scene);
  ts_assert(result == 0, "Tick should return 0 after terminate");
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *test_scene = ts_create_scene();

  TestCase cases[] = {
      {test_scene},
  };

  // Constructing Tests
  for (size_t i = 0; i < 1; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_set_scene_terminate/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
