#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  char *data;
  size_t expected_entity_count;
  size_t expected_components;
  char *expected_component_name;
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
  int result = ts_deserialize_entity(input->scene, input->data);

  if (input->should_fail) {
    ts_assert(result == 1, "Should fail for invalid input");
    return;
  }

  ts_assert(result == 0, "Should succeed for valid data");

  size_t entity_count = 0;
  TS_Entity *entities = ts_get_entities(&entity_count, input->scene);

  ts_assert_test(entity_count == input->expected_entity_count, "Expected: %d",
                 input->expected_entity_count, "Got: %ld", entity_count,
                 "Entity count doesn't match");

  if (entities != NULL && entity_count > 0) {
    size_t component_count = 0;
    char **components = ts_get_components_of_entity(&component_count,
                                                    input->scene, entities[0]);

    ts_assert_test(component_count == input->expected_components,
                   "Expected: %d", input->expected_components, "Got: %ld",
                   component_count, "Component count doesn't match");

    if (component_count > 0 && input->expected_component_name) {
      ts_assert_test(strcmp(components[0], input->expected_component_name) == 0,
                     "Expected: %s", input->expected_component_name, "Got: %s",
                     components[0], "Component name doesn't match");
    }

    for (size_t i = 0; i < component_count; i++) {
      free(components[i]);
    }
    free(components);
    free(entities);
  }
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *entity_scene = ts_create_scene();
  ts_assert(0 ==
                ts_load_plugin(entity_scene, "./libtheseed_test_components.so"),
            "Failed to load the plugin");

  TS_Scene *component_scene = ts_create_scene();
  ts_assert(
      0 == ts_load_plugin(component_scene, "./libtheseed_test_components.so"),
      "Failed to load the plugin");

  TS_Scene *multi_entity_scene = ts_create_scene();
  ts_add_entity(multi_entity_scene);
  ts_assert(0 == ts_load_plugin(multi_entity_scene,
                                "./libtheseed_test_components.so"),
            "Failed to load the plugin");

  TS_Scene *multi_entity_scene2 = ts_create_scene();
  ts_add_entity(multi_entity_scene2);
  ts_assert(0 == ts_load_plugin(multi_entity_scene2,
                                "./libtheseed_test_components.so"),
            "Failed to load the plugin");

  TestCase cases[] = {
      {NULL, NULL, 0, 0, NULL, 1},
      {NULL, "", 0, 0, NULL, 1},
      {entity_scene, "\10\0\0\0\0\0\0\0", 1, 0, NULL, 0},
      {component_scene,
       "\65\0\0\0\0\0\0\0\55\0\0\0\0\0\0\0TS_"
       "A\0\16\0\0\0\0\0\0\0x\0\1\0\0\0\22\0\0\0\0\0\0\0extra\0\5\0\0\0",
       1, 1, "TS_A", 0},
      {multi_entity_scene, "\10\0\0\0\0\0\0\0", 2, 0, NULL, 0},
      {multi_entity_scene2, "\10\0\0\0\0\0\0\0", 2, 0, NULL, 0}};

  // Constructing Tests
  for (size_t i = 0; i < 6; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_deserialize_entity/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
