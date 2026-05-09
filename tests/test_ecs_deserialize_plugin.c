#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  char *data;
  char *expected_plugin;
  int should_fail;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;

  if (input->scene) {
    wxr_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  int result = wxr_deserialize_plugin(input->scene, input->data);

  if (input->should_fail) {
    wxr_assert(result == 1, "Should fail for invalid input");
    return;
  }

  wxr_assert(result == 0, "Should succeed for valid data");

  if (input->expected_plugin) {
    size_t plugin_count = 0;
    char **plugins = wxr_get_plugins(&plugin_count, input->scene);

    wxr_assert_test(plugin_count == 1, "Expected: %d", 1, "Got: %ld",
                   plugin_count, "Plugin count doesn't match");
    wxr_assert(plugins != NULL, "Plugins array should not be NULL");
    wxr_assert_test(strcmp(plugins[0], input->expected_plugin) == 0,
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

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *valid_scene = wxr_create_scene();

  TestCase cases[] = {{NULL, NULL, NULL, 1},
                      {NULL, "", NULL, 1},
                      {valid_scene,
                       "\47\0\0\0\0\0\0\0./libwasserxr_test_components.so\0",
                       "./libwasserxr_test_components.so", 0}};

  // Constructing Tests
  for (size_t i = 0; i < 3; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_deserialize_plugin/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
