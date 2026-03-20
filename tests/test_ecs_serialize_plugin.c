#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  char *plugin;
  size_t length;
  char *out;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;

  ts_destroy_scene(input->scene);
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  char *data = ts_serialize_plugin(input->scene, input->plugin);
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
  ts_assert(0 == ts_load_plugin(plugin_scene, "./libtheseed_components.so"),
            "Failed to load the plugin");

  TS_Scene *plugin_scene2 = ts_create_scene();
  ts_assert(0 == ts_load_plugin(plugin_scene2, "./libtheseed_components.so"),
            "Failed to load the plugin");

  TestCase cases[] = {
      {NULL, NULL, 0, NULL},
      {NULL, "", 0, NULL},
      {empty_scene, "", 0, NULL},
      {empty_scene2, "./libtheseed_components.so", 0, NULL},
      {plugin_scene, "", 0, NULL},
      {plugin_scene2, "./libtheseed_components.so",
       strlen("./libtheseed_components.so") + sizeof(size_t) + 1,
       "\43\0\0\0\0\0\0\0./libtheseed_components.so\0"}};

  // Constructing Tests

  for (size_t i = 0; i < 6; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_serialize_plugin/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
