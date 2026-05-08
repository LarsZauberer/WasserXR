#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/ecs/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  void *component;
  size_t length;
  char *out;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;

  ts_destroy_scene(input->scene);
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  char *data = ts_serialize_component(input->scene, input->component);
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

  TS_Scene *entity_scene = ts_create_scene();
  ts_assert(0 ==
                ts_load_plugin(entity_scene, "./libtheseed_test_components.so"),
            "Failed to load the plugin");
  ts_add_entity(entity_scene);

  TS_Scene *component_scene = ts_create_scene();
  ts_assert(
      0 == ts_load_plugin(component_scene, "./libtheseed_test_components.so"),
      "Failed to load the plugin");
  TS_Entity entity_id_component = ts_add_entity(component_scene);
  void *component =
      ts_add_component(component_scene, entity_id_component, "TS_A");
  ts_assert(component != NULL, "Failed to add component");

  TS_Scene *empty_component_scene = ts_create_scene();
  ts_assert(0 == ts_load_plugin(empty_component_scene,
                                "./libtheseed_test_components.so"),
            "Failed to load the plugin");
  TS_Entity entity_id_empty_component = ts_add_entity(empty_component_scene);
  void *empty_component = ts_add_component(
      empty_component_scene, entity_id_empty_component, "TS_C_Empty");
  ts_assert(empty_component != NULL, "Failed to add component");

  TestCase cases[] = {
      {NULL, NULL, 0, NULL},
      {NULL, NULL, 0, NULL},
      {empty_scene, NULL, 0, NULL},
      {entity_scene, NULL, 0, NULL},
      {component_scene, component,
       sizeof(size_t) + strlen("TS_A") + 1 + sizeof(size_t) + strlen("x") + 1 +
           sizeof(int) + sizeof(size_t) + strlen("extra") + 1 + sizeof(int),
       "\55\0\0\0\0\0\0\0TS_"
       "A\0\16\0\0\0\0\0\0\0x\0\1\0\0\0\22\0\0\0\0\0\0\0extra\0\5\0\0\0"},
      {empty_component_scene, empty_component,
       sizeof(size_t) + strlen("TS_C_Empty") + 1,
       "\23\0\0\0\0\0\0\0TS_C_Empty\0"},
  };

  // Constructing Tests

  for (size_t i = 0; i < 6; i++) {
    char *path =
        g_strdup_printf("/theseed/test_ecs_serialize_component/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
