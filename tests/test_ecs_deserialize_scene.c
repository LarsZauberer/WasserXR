#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  char *data;
  size_t expected_systems;
  size_t expected_entities;
  int should_fail;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;

  if (input->scene) {
    ts_destroy_scene(input->scene);
  }
}

static void destroy_char_array(char **ptr, const size_t size) {
  for (size_t i = 0; i < size; i++) {
    free(ptr[i]);
  }
  free(ptr);
}

// NOLINTNEXTLINE(readability-function-cognitive-complexity)
static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  int result = ts_deserialize_scene(input->scene, input->data);

  if (input->should_fail) {
    ts_assert(result == 1, "Should fail for invalid input");
    return;
  }

  ts_assert(result == 0, "Should succeed for valid data");

  // Verify systems
  size_t system_count = 0;
  char **systems = ts_get_systems(&system_count, input->scene);
  ts_assert_test(system_count == input->expected_systems,
                 "Expected systems: %d", input->expected_systems, "Got: %ld",
                 system_count, "System count doesn't match");

  if (system_count > 0) {
    ts_assert_test(strcmp(systems[0], "ts_console_system") == 0, "Expected: %s",
                   "ts_console_system", "Got: %s", systems[0],
                   "System name doesn't match");
  }

  destroy_char_array(systems, system_count);

  // Verify entities
  size_t entity_count = 0;
  TS_Entity *entities = ts_get_entities(&entity_count, input->scene);
  ts_assert_test(entity_count == input->expected_entities,
                 "Expected entities: %d", input->expected_entities, "Got: %ld",
                 entity_count, "Entity count doesn't match");

  // Verify entity components if there are entities
  if (entity_count > 0) {
    size_t component_count = 0;
    char **components = ts_get_components_of_entity(&component_count,
                                                    input->scene, entities[0]);

    if (input->expected_systems > 0) {
      // Full scene case - should have component
      ts_assert_test(component_count == 1, "Expected: %d", 1, "Got: %ld",
                     component_count, "Component count doesn't match");
      if (component_count > 0) {
        ts_assert_test(strcmp(components[0], "TS_A") == 0, "Expected: %s",
                       "TS_A", "Got: %s", components[0],
                       "Component name doesn't match");
      }
    } else {
      // Entity scene case - no components
      ts_assert_test(component_count == 0, "Expected: %d", 0, "Got: %ld",
                     component_count, "Component count doesn't match");
    }

    destroy_char_array(components, component_count);
  }

  free(entities);
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *empty_scene = ts_create_scene();
  TS_Scene *entity_scene = ts_create_scene();
  TS_Scene *full_scene = ts_create_scene();

  ts_load_plugin(full_scene, "./libtheseed_test_components.so");
  ts_load_plugin(full_scene, "./libtheseed_systems.so");

  TestCase cases[] = {
      {NULL, NULL, 0, 0, 1},
      {NULL, "", 0, 0, 1},
      {empty_scene,
       "\40\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0",
       0, 0, 0},
      {entity_scene,
       "\120\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\50\0\0\0\0\0\0\0./libtheseed_test_components.so\0"
       "\10\0\0\0\0\0\0\0",
       0, 1, 0},
      {full_scene,
       "\273\0\0\0\0\0\0\0"
       "\2\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\50\0\0\0\0\0\0\0./libtheseed_test_components.so\0"
       "\40\0\0\0\0\0\0\0./libtheseed_systems.so\0"
       "\36\0\0\0\0\0\0\0ts_console_system\0\144\0\0\0"
       "\67\0\0\0\0\0\0\0\55\0\0\0\0\0\0\0TS_"
       "A\0\16\0\0\0\0\0\0\0x\0\1\0\0\0\22\0\0\0\0\0\0\0extra\0\5\0\0\0",
       1, 1, 0}};

  // Constructing Tests
  for (size_t i = 0; i < 5; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_deserialize_scene/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
