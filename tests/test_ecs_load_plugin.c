#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  char *plugin_path;
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
  
  int result = ts_load_plugin(input->scene, input->plugin_path);
  ts_assert(result == input->expected_result,
            "Load plugin result should match expected");
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *null_scene = NULL;

  TS_Scene *valid_scene = ts_create_scene();

  TS_Scene *valid_scene2 = ts_create_scene();

  TestCase cases[] = {
      {null_scene, NULL, 1},                                   // NULL scene
      {valid_scene, "", 1},                                    // Empty path
      {valid_scene2, "./libtheseed_components.so", 0},         // Valid plugin
  };

  // Constructing Tests
  for (size_t i = 0; i < 3; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_load_plugin/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
