#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/ecs/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  TS_Entity entity;
  char *data;
  char *expected_component;
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
  int result =
      ts_deserialize_component(input->scene, input->entity, input->data);

  if (input->should_fail) {
    ts_assert(result == 1, "Should fail for invalid input");
    return;
  }

  ts_assert(result == 0, "Should succeed for valid data");

  if (input->expected_component) {
    size_t component_count = 0;
    char **components = ts_get_components_of_entity(
        &component_count, input->scene, input->entity);

    ts_assert_test(component_count == 1, "Expected: %d", 1, "Got: %ld",
                   component_count, "Component count doesn't match");
    ts_assert(components != NULL, "Components array should not be NULL");
    ts_assert_test(strcmp(components[0], input->expected_component) == 0,
                   "Expected: %s", input->expected_component, "Got: %s",
                   components[0], "Component name doesn't match");

    // Verify we can get the component
    void *comp = ts_entity_get_component(input->scene, input->entity,
                                         input->expected_component);
    ts_assert(comp != NULL, "Component data should not be NULL");

    // Verify component fields using schema
    TS_Component_Schema *schema =
        ts_get_schema_of_component(input->scene, comp);
    ts_assert(schema != NULL, "Schema should not be NULL");

    if (strcmp(input->expected_component, "TS_A") == 0) {
      // Get field values
      const void *x_value = ts_get(input->scene, comp, "x");
      ts_assert(x_value != NULL, "x field value should not be NULL");
      const int x_int = *(const int *)x_value;
      ts_assert_test(x_int == 1, "Expected: %d", 1, "Got: %d", x_int,
                     "x field value doesn't match");

      const void *extra_value = ts_get(input->scene, comp, "extra");
      ts_assert(extra_value != NULL, "extra field value should not be NULL");
      const int extra = *(const int *)extra_value;
      ts_assert_test(extra == 5, "Expected: %d", 5, "Got: %d", extra,
                     "extra field value doesn't match");
    } else if (strcmp(input->expected_component, "TS_B") == 0) {
      const void *name_value = ts_get(input->scene, comp, "name");
      ts_assert(name_value, "name field value should not be NULL");
      const char *name_string = (const char *)name_value;
      ts_assert(strcmp(name_string, "Hello World!") == 0,
                "Name is not `Hello World!`. It is: %s", name_string);
    }

    for (size_t i = 0; i < component_count; i++) {
      free(components[i]);
    }

    free(components);
  }
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *empty_scene = ts_create_scene();
  ts_assert(0 == ts_load_plugin(empty_scene, "./libtheseed_test_components.so"),
            "Failed to load the plugin");
  TS_Entity dummy_entity = ts_add_entity(empty_scene);

  TS_Scene *invalid_entity_scene = ts_create_scene();
  ts_assert(0 == ts_load_plugin(invalid_entity_scene,
                                "./libtheseed_test_components.so"),
            "Failed to load the plugin");

  TS_Scene *valid_scene = ts_create_scene();
  ts_assert(0 == ts_load_plugin(valid_scene, "./libtheseed_test_components.so"),
            "Failed to load the plugin");
  TS_Entity valid_entity = ts_add_entity(valid_scene);

  TS_Scene *valid_string_scene = ts_create_scene();
  ts_assert(0 == ts_load_plugin(valid_string_scene,
                                "./libtheseed_test_components.so"),
            "Failed to load the plugin");
  TS_Entity valid_string_entity = ts_add_entity(valid_string_scene);

  TestCase cases[] = {
      {NULL, 0, NULL, NULL, 1},
      {NULL, 0, "", NULL, 1},
      {empty_scene, dummy_entity, NULL, NULL, 1},
      {invalid_entity_scene, 999,
       "\55\0\0\0\0\0\0\0"
       "TS_A\0"
       "\16\0\0\0\0\0\0\0x\0\1\0\0\0"
       "\22\0\0\0\0\0\0\0extra\0\5\0\0\0",
       NULL, 1},
      {valid_scene, valid_entity,
       "\55\0\0\0\0\0\0\0"
       "TS_A\0"
       "\16\0\0\0\0\0\0\0x\0\1\0\0\0"
       "\22\0\0\0\0\0\0\0extra\0\5\0\0\0",
       "TS_A", 0},
      {valid_string_scene, valid_string_entity,
       "\47\0\0\0\0\0\0\0"
       "TS_B\0"
       "\32\0\0\0\0\0\0\0name\0Hello World!\0",
       "TS_B", 0},
  };

  // Constructing Tests
  for (size_t i = 0; i < 6; i++) {
    char *path =
        g_strdup_printf("/theseed/test_ecs_deserialize_component/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
