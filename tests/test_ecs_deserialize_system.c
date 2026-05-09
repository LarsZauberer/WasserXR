#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  char *data;
  char *expected_system;
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
  int result = wxr_deserialize_system(input->scene, input->data);

  if (input->should_fail) {
    wxr_assert(result == 1, "Should fail for invalid input");
    return;
  }

  wxr_assert(result == 0, "Should succeed for valid data");

  if (input->expected_system) {
    size_t system_count = 0;
    char **systems = wxr_get_systems(&system_count, input->scene);

    wxr_assert_test(system_count == 1, "Expected: %d", 1, "Got: %ld",
                   system_count, "System count doesn't match");
    wxr_assert(systems != NULL, "Systems array should not be NULL");
    wxr_assert_test(strcmp(systems[0], input->expected_system) == 0,
                   "Expected: %s", input->expected_system, "Got: %s",
                   systems[0], "System name doesn't match");

    for (size_t i = 0; i < system_count; i++) {
      free(systems[i]);
    }
    free(systems);
  }
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *empty_scene = wxr_create_scene();

  WXR_Scene *valid_scene = wxr_create_scene();
  wxr_assert(0 == wxr_load_plugin(valid_scene, "./libwasserxr_test_systems.so"),
            "Failed to load the plugin");

  TestCase cases[] = {
      {NULL, NULL, NULL, 1},
      {NULL, "", NULL, 1},
      {empty_scene, "\23\0\0\0\0\0\0\0wxr_system_a\0\144\0\0\0", NULL, 1},
      {valid_scene, "\23\0\0\0\0\0\0\0wxr_system_a\0\144\0\0\0",
       "wxr_system_a", 0}};

  // Constructing Tests
  for (size_t i = 0; i < 4; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_deserialize_system/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
