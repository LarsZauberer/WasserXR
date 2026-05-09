#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  size_t length;
  char *out;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;

  wxr_destroy_scene(input->scene);
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  char *data = wxr_serialize_scene(input->scene);
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

  WXR_Scene *entity_scene = wxr_create_scene();
  wxr_assert(0 ==
                wxr_load_plugin(entity_scene, "./libwasserxr_test_components.so"),
            "Failed to load the plugin (test_components)");
  wxr_add_entity(entity_scene);

  WXR_Scene *full_scene = wxr_create_scene();
  wxr_assert(0 == wxr_load_plugin(full_scene, "./libwasserxr_test_components.so"),
            "Failed to load the plugin (test components)");
  wxr_assert(0 == wxr_load_plugin(full_scene, "./libwasserxr_test_systems.so"),
            "Failed to load the plugin (systems)");
  WXR_Entity entity_id_component = wxr_add_entity(full_scene);
  wxr_assert(wxr_add_component(full_scene, entity_id_component, "WXR_A") != NULL,
            "Failed to add component");
  wxr_assert(0 == wxr_add_system(full_scene, "wxr_system_a", 100),
            "Failed to add system");

  TestCase cases[] = {
      {NULL, 0, NULL},
      {NULL, 0, NULL},
      {empty_scene,
       sizeof(size_t) + sizeof(size_t) + sizeof(size_t),
       "\30\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"},
      {entity_scene,
       sizeof(size_t) + sizeof(size_t) + sizeof(size_t) + sizeof(size_t),
       "\40\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\10\0\0\0\0\0\0\0"},
      {full_scene,
       sizeof(size_t) + (sizeof(size_t) + sizeof(size_t)) +
           (sizeof(size_t) + strlen("wxr_system_a") + 1 + sizeof(int)) +
           (sizeof(size_t) + sizeof(size_t) + strlen("WXR_A") + 1 +
            sizeof(size_t) + strlen("x") + 1 + sizeof(int) + sizeof(size_t) +
            strlen("extra") + 1 + sizeof(int)),
       "\147\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\31\0\0\0\0\0\0\0wxr_system_a\0\144\0\0\0"
        "\66\0\0\0\0\0\0\0\56\0\0\0\0\0\0\0WXR_"
        "A\0\16\0\0\0\0\0\0\0x\0\1\0\0\0\22\0\0\0\0\0\0\0extra\0\5\0\0\0"},
  };

  // Constructing Tests

  for (size_t i = 0; i < 5; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_serialize_scene/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
