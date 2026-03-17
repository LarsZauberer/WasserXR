#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  char *data;
  int expected_plugins;
  int expected_systems;
  int expected_entities;
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
  int result = ts_deserialize_scene(input->scene, input->data);
  
  if (input->should_fail) {
    ts_assert(result == 1, "Should fail for invalid input");
    return;
  }
  
  ts_assert(result == 0, "Should succeed for valid data");
  
  // Verify plugins
  size_t plugin_count = 0;
  char **plugins = ts_get_plugins(&plugin_count, input->scene);
  ts_assert_test(plugin_count == input->expected_plugins,
                 "Expected plugins: %d", input->expected_plugins,
                 "Got: %ld", plugin_count,
                 "Plugin count doesn't match");
  
  if (plugin_count == 1) {
    ts_assert_test(strcmp(plugins[0], "./libtheseed_test_components.so") == 0,
                   "Expected: %s", "./libtheseed_test_components.so",
                   "Got: %s", plugins[0],
                   "Plugin name doesn't match");
  } else if (plugin_count == 2) {
    ts_assert_test(strcmp(plugins[0], "./libtheseed_test_components.so") == 0,
                   "Expected: %s", "./libtheseed_test_components.so",
                   "Got: %s", plugins[0],
                   "First plugin name doesn't match");
    ts_assert_test(strcmp(plugins[1], "./libtheseed_systems.so") == 0,
                   "Expected: %s", "./libtheseed_systems.so",
                   "Got: %s", plugins[1],
                   "Second plugin name doesn't match");
  }
  
  if (plugins != NULL) {
    free(plugins);
  }
  
  // Verify systems
  size_t system_count = 0;
  char **systems = ts_get_systems(&system_count, input->scene);
  ts_assert_test(system_count == input->expected_systems,
                 "Expected systems: %d", input->expected_systems,
                 "Got: %ld", system_count,
                 "System count doesn't match");
  
  if (system_count > 0) {
    ts_assert_test(strcmp(systems[0], "ts_console_system") == 0,
                   "Expected: %s", "ts_console_system",
                   "Got: %s", systems[0],
                   "System name doesn't match");
  }
  
  if (systems != NULL) {
    free(systems);
  }
  
  // Verify entities
  size_t entity_count = 0;
  TS_Entity *entities = ts_get_entities(&entity_count, input->scene);
  ts_assert_test(entity_count == input->expected_entities,
                 "Expected entities: %d", input->expected_entities,
                 "Got: %ld", entity_count,
                 "Entity count doesn't match");
  
  // Verify entity components if there are entities
  if (entity_count > 0 && input->expected_plugins > 0) {
    size_t comp_count = 0;
    char **components = ts_get_components_of_entity(&comp_count, input->scene, entities[0]);
    
    if (input->expected_systems > 0) {
      // Full scene case - should have component
      ts_assert_test(comp_count == 1, "Expected: %d", 1, "Got: %ld",
                     comp_count, "Component count doesn't match");
      if (comp_count > 0) {
        ts_assert_test(strcmp(components[0], "TS_A") == 0,
                       "Expected: %s", "TS_A",
                       "Got: %s", components[0],
                       "Component name doesn't match");
      }
    } else {
      // Entity scene case - no components
      ts_assert_test(comp_count == 0, "Expected: %d", 0, "Got: %ld",
                     comp_count, "Component count doesn't match");
    }
    
    if (components != NULL) {
      free(components);
    }
  }
  
  if (entities != NULL) {
    free(entities);
  }
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *empty_scene = ts_create_scene();
  TS_Scene *empty_scene2 = ts_create_scene();
  TS_Scene *entity_scene = ts_create_scene();
  TS_Scene *full_scene = ts_create_scene();

  TestCase cases[] = {
      {NULL, NULL, 0, 0, 0, 1},
      {NULL, "", 0, 0, 0, 1},
      {empty_scene,
       "\40\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0",
       0, 0, 0, 0},
      {empty_scene2, "", 0, 0, 0, 1},
      {entity_scene,
       "\130\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\50\0\0\0\0\0\0\0./libtheseed_test_components.so\0"
       "\20\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
       1, 0, 1, 0},
      {full_scene,
       "\303\0\0\0\0\0\0\0"
       "\2\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\50\0\0\0\0\0\0\0./libtheseed_test_components.so\0"
       "\40\0\0\0\0\0\0\0./libtheseed_systems.so\0"
       "\36\0\0\0\0\0\0\0ts_console_system\0\144\0\0\0"
       "\75\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\55\0\0\0\0\0\0\0TS_"
       "A\0\16\0\0\0\0\0\0\0x\0\1\0\0\0\22\0\0\0\0\0\0\0extra\0\5\0\0\0",
       2, 1, 1, 0}};

  // Constructing Tests
  for (size_t i = 0; i < 6; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_deserialize_scene/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
