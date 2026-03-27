#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  char *system_name;
  int priority;
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

  int result = ts_add_system(input->scene, input->system_name, input->priority);
  ts_assert(result == input->expected_result,
            "Add system result should match expected");
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
      0 == ts_load_plugin(scene_with_plugin, "./libtheseed_test_systems.so"),
      "Failed to load the plugin");

  TS_Scene *scene_with_plugin2 = ts_create_scene();
  ts_assert(
      0 == ts_load_plugin(scene_with_plugin2, "./libtheseed_test_systems.so"),
      "Failed to load the plugin");

  TestCase cases[] = {
      {null_scene, NULL, 0, 1},                   // NULL scene
      {empty_scene, "ts_system_a", 0, 1},         // Empty scene without plugin
      {scene_with_plugin, "asdf", 5, 1},          // Non-existent system
      {scene_with_plugin2, "ts_system_a", 10, 0}, // Valid system add
  };

  // Constructing Tests
  for (size_t i = 0; i < 4; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_add_system/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
