#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/ecs/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  char *data;
  char *expected_plugin;
  int should_fail;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;

  if (input->scene) {
    ts_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  int result = ts_deserialize_plugin(input->scene, input->data);

  if (input->should_fail) {
    ts_assert(result == 1, "Should fail for invalid input");
    return;
  }

  ts_assert(result == 0, "Should succeed for valid data");

  if (input->expected_plugin) {
    size_t plugin_count = 0;
    char **plugins = ts_get_plugins(&plugin_count, input->scene);

    ts_assert_test(plugin_count == 1, "Expected: %d", 1, "Got: %ld",
                   plugin_count, "Plugin count doesn't match");
    ts_assert(plugins != NULL, "Plugins array should not be NULL");
    ts_assert_test(strcmp(plugins[0], input->expected_plugin) == 0,
                   "Expected: %s", input->expected_plugin, "Got: %s",
                   plugins[0], "Plugin name doesn't match");

    for (size_t i = 0; i < plugin_count; i++) {
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
  TS_Scene *valid_scene = ts_create_scene();

  TestCase cases[] = {{NULL, NULL, NULL, 1},
                      {NULL, "", NULL, 1},
                      {valid_scene,
                       "\47\0\0\0\0\0\0\0./libtheseed_test_components.so\0",
                       "./libtheseed_test_components.so", 0}};

  // Constructing Tests
  for (size_t i = 0; i < 3; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_deserialize_plugin/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
