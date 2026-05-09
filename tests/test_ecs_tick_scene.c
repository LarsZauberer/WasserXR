#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/ecs/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  int should_terminate;
  int expected_result;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    ts_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  
  if (input->should_terminate) {
    ts_set_scene_terminate(input->scene);
  }
  
  int result = ts_tick_scene(input->scene);
  ts_assert(result == input->expected_result,
            "Tick scene result should match expected");
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *normal_scene = ts_create_scene();

  TS_Scene *terminate_scene = ts_create_scene();

  TestCase cases[] = {
      {normal_scene, 0, 1},       // Normal tick
      {terminate_scene, 1, 0},    // Terminated tick
  };

  // Constructing Tests
  for (size_t i = 0; i < 2; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_tick_scene/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
