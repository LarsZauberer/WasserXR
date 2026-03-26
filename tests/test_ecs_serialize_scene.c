#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  size_t length;
  char *out;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;

  ts_destroy_scene(input->scene);
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  char *data = ts_serialize_scene(input->scene);
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
            "Failed to load the plugin (test_components)");
  ts_add_entity(entity_scene);

  TS_Scene *full_scene = ts_create_scene();
  ts_assert(0 == ts_load_plugin(full_scene, "./libtheseed_test_components.so"),
            "Failed to load the plugin (test components)");
  ts_assert(0 == ts_load_plugin(full_scene, "./libtheseed_test_systems.so"),
            "Failed to load the plugin (systems)");
  TS_Entity entity_id_component = ts_add_entity(full_scene);
  ts_assert(ts_add_component(full_scene, entity_id_component, "TS_A") != NULL,
            "Failed to add component");
  ts_assert(0 == ts_add_system(full_scene, "ts_system_a", 100),
            "Failed to add system");

  ts_debug(
      "Length of last test case: %ld",
      sizeof(size_t) + (sizeof(size_t) + sizeof(size_t) + sizeof(size_t)) +
          (sizeof(size_t) + strlen("./libtheseed_test_components.so") + 1) +
          (sizeof(size_t) + strlen("./libtheseed_systems.so") + 1) +
          (sizeof(size_t) + strlen("ts_console_system") + 1 + sizeof(int)) +
          (sizeof(size_t) + sizeof(TS_Entity) + sizeof(size_t) +
           strlen("TS_A") + 1 + sizeof(size_t) + strlen("x") + 1 + sizeof(int) +
           sizeof(size_t) + strlen("extra") + 1 + sizeof(int)));

  TestCase cases[] = {
      {NULL, 0, NULL},
      {NULL, 0, NULL},
      {empty_scene,
       sizeof(size_t) + sizeof(size_t) + sizeof(size_t) + sizeof(size_t),
       "\40\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"},
      {entity_scene,
       sizeof(size_t) + sizeof(size_t) + sizeof(size_t) + sizeof(size_t) +
           sizeof(size_t) + strlen("./libtheseed_test_components.so") + 1 +
           sizeof(size_t),
       "\120\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\50\0\0\0\0\0\0\0./libtheseed_test_components.so\0"
       "\10\0\0\0\0\0\0\0"},
      {full_scene,
       sizeof(size_t) + (sizeof(size_t) + sizeof(size_t) + sizeof(size_t)) +
           (sizeof(size_t) + strlen("./libtheseed_test_components.so") + 1) +
           (sizeof(size_t) + strlen("./libtheseed_test_systems.so") + 1) +
           (sizeof(size_t) + strlen("ts_system_a") + 1 + sizeof(int)) +
           (sizeof(size_t) + sizeof(size_t) + strlen("TS_A") + 1 +
            sizeof(size_t) + strlen("x") + 1 + sizeof(int) + sizeof(size_t) +
            strlen("extra") + 1 + sizeof(int)),
       "\272\0\0\0\0\0\0\0"
       "\2\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\50\0\0\0\0\0\0\0./libtheseed_test_components.so\0"
       "\45\0\0\0\0\0\0\0./libtheseed_test_systems.so\0"
       "\30\0\0\0\0\0\0\0ts_system_a\0\144\0\0\0"
       "\65\0\0\0\0\0\0\0\55\0\0\0\0\0\0\0TS_"
       "A\0\16\0\0\0\0\0\0\0x\0\1\0\0\0\22\0\0\0\0\0\0\0extra\0\5\0\0\0"},
  };

  // Constructing Tests

  for (size_t i = 0; i < 5; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_serialize_scene/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
