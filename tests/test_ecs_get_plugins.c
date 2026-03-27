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
  char **plugins = ts_get_plugins(&count, input->scene);
  
  ts_assert(count == input->expected_count,
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

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *empty_scene = ts_create_scene();

  TS_Scene *scene_one_plugin = ts_create_scene();
  ts_assert(0 == ts_load_plugin(scene_one_plugin, "./libtheseed_components.so"),
            "Failed to load the plugin");

  TestCase cases[] = {
      {empty_scene, 0},
      {scene_one_plugin, 1},
  };

  // Constructing Tests
  for (size_t i = 0; i < 2; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_get_plugins/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
