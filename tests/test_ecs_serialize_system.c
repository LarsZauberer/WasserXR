#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  char *system;
  size_t length;
  char *out;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;

  wxr_destroy_scene(input->scene);
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  char *data = wxr_serialize_system(input->scene, input->system);
  if (!input->out) {
    wxr_assert(data == NULL, "Data should be NULL");
    return;
  }
  wxr_assert(data != NULL, "Data is NULL");
  size_t length = 0;
  memcpy(&length, data, sizeof(size_t));
  wxr_assert_test(length == input->length, "%ld", input->length, "%ld", length,
                 "The size of the data returned doesn't match!");
  for (size_t i = 0; i < length; i++) {
    char byte_should = input->out[i];
    char byte_out = data[i];
    wxr_assert_test(byte_should == byte_out, "Should: %d", byte_should,
                   "Output: %d", byte_out, "The Byte at index %d is not equal",
                   i);
  }

  free(data);
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *empty_scene = wxr_create_scene();
  WXR_Scene *empty_scene2 = wxr_create_scene();

  WXR_Scene *plugin_scene = wxr_create_scene();
  wxr_assert(0 == wxr_load_plugin(plugin_scene, "./libwasserxr_test_systems.so"),
            "Failed to load the plugin");

  WXR_Scene *system_scene = wxr_create_scene();
  wxr_assert(0 == wxr_load_plugin(system_scene, "./libwasserxr_test_systems.so"),
            "Failed to load the plugin");
  wxr_assert(0 == wxr_add_system(system_scene, "wxr_system_a", 100),
            "Failed to add the system");

  TestCase cases[] = {{NULL, NULL, 0, NULL},
                      {NULL, "", 0, NULL},
                      {empty_scene, "", 0, NULL},
                      {empty_scene2, "wxr_system_a", 0, NULL},
                      {plugin_scene, "wxr_system_a", 0, NULL},
                      {system_scene, "wxr_system_a",
                       sizeof(size_t) + strlen("wxr_system_a") + 1 + sizeof(int),
                       "\31\0\0\0\0\0\0\0wxr_system_a\0\144\0\0\0"}};

  // Constructing Tests

  for (size_t i = 0; i < 6; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_serialize_system/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
