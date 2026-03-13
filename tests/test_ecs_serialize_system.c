#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  char *system;
  size_t length;
  char *out;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;

  ts_destroy_scene(input->scene);
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  char *data = ts_serialize_system(input->scene, input->system);
  if (!input->out) {
    ts_assert(data == NULL, "Data should be NULL");
    return;
  }
  ts_assert(data != NULL, "Data is NULL");
  size_t length = 0;
  memcpy(&length, data, sizeof(size_t));
  ts_assert_test(length == input->length, "%ld", input->length, "%ld", length,
                 "The size of the data returned doesn't match!");
  for (size_t i = 0; i < length; i++) {
    char byte_should = input->out[i];
    char byte_out = data[i];
    ts_assert_test(byte_should == byte_out, "Should: %d", byte_should,
                   "Output: %d", byte_out, "The Byte at index %d is not equal",
                   i);
  }

  free(data);
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *empty_scene = ts_create_scene();
  TS_Scene *empty_scene2 = ts_create_scene();

  TS_Scene *plugin_scene = ts_create_scene();
  ts_assert(0 == ts_load_plugin(plugin_scene, "./libtheseed_systems.so"),
            "Failed to load the plugin");

  TS_Scene *system_scene = ts_create_scene();
  ts_assert(0 == ts_load_plugin(system_scene, "./libtheseed_systems.so"),
            "Failed to load the plugin");
  ts_assert(0 == ts_add_system(system_scene, "ts_console_system", 100),
            "Failed to add the system");

  TestCase cases[] = {
      {NULL, NULL, 0, NULL},
      {NULL, "", 0, NULL},
      {empty_scene, "", 0, NULL},
      {empty_scene2, "ts_console_system", 0, NULL},
      {plugin_scene, "ts_console_system", 0, NULL},
      {system_scene, "ts_console_system",
       sizeof(size_t) + strlen("ts_console_system") + 1 + sizeof(int),
       "\36\0\0\0\0\0\0\0ts_console_system\0\144\0\0\0"}};

  // Constructing Tests

  for (size_t i = 0; i < 6; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_add_component/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
