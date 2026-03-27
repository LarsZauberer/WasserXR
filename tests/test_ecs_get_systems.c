#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>
#include <stdlib.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  size_t expected_count;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    ts_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  
  size_t count = 0;
  char **systems = ts_get_systems(&count, input->scene);
  
  ts_assert(count == input->expected_count,
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

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *empty_scene = ts_create_scene();

  TS_Scene *scene_one_system = ts_create_scene();
  ts_assert(0 == ts_load_plugin(scene_one_system, "./libtheseed_systems.so"),
            "Failed to load the plugin");
  ts_add_system(scene_one_system, "TS_TestSystem", 10);

  TestCase cases[] = {
      {empty_scene, 0},
      {scene_one_system, 1},
  };

  // Constructing Tests
  for (size_t i = 0; i < 2; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_get_systems/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
