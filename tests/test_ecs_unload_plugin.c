#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/ecs/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  char *plugin_name;
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

  int result = ts_unload_plugin(input->scene, input->plugin_name);
  ts_assert(result == input->expected_result,
            "Unload plugin result should match expected");
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *null_scene = NULL;

  TS_Scene *empty_scene = ts_create_scene();

  TS_Scene *scene_with_plugin = ts_create_scene();
  ts_assert(
      0 == ts_load_plugin(scene_with_plugin, "./libtheseed_core.so"),
      "Failed to load the plugin");

  TS_Scene *empty_scene2 = ts_create_scene();

  TestCase cases[] = {
      {null_scene, NULL, 1},                                     // NULL scene
      {empty_scene, "", 1},                                      // Empty scene
      {scene_with_plugin, "./libtheseed_core.so", 0}, // Valid unload
      {empty_scene2, "NonExistent", 1}, // Non-existent plugin
  };

  // Constructing Tests
  for (size_t i = 0; i < 4; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_unload_plugin/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
