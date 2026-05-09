#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>
#include <string.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  char *data;
  size_t expected_systems;
  size_t expected_entities;
  int should_fail;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;

  if (input->scene) {
    wxr_destroy_scene(input->scene);
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
  int result = wxr_deserialize_scene(input->scene, input->data);

  if (input->should_fail) {
    wxr_assert(result == 1, "Should fail for invalid input");
    return;
  }

  wxr_assert(result == 0, "Should succeed for valid data");

  // Verify systems
  size_t system_count = 0;
  char **systems = wxr_get_systems(&system_count, input->scene);
  wxr_assert_test(system_count == input->expected_systems,
                  "Expected systems: %d", input->expected_systems, "Got: %ld",
                  system_count, "System count doesn't match");

  if (system_count > 0) {
    wxr_assert_test(strcmp(systems[0], "wxr_system_a") == 0, "Expected: %s",
                    "wxr_system_a", "Got: %s", systems[0],
                    "System name doesn't match");
  }

  destroy_char_array(systems, system_count);

  // Verify entities
  size_t entity_count = 0;
  WXR_Entity *entities = wxr_get_entities(&entity_count, input->scene);
  wxr_assert_test(entity_count == input->expected_entities,
                  "Expected entities: %d", input->expected_entities, "Got: %ld",
                  entity_count, "Entity count doesn't match");

  // Verify entity components if there are entities
  if (entity_count > 0) {
    size_t component_count = 0;
    char **components = wxr_get_components_of_entity(&component_count,
                                                     input->scene, entities[0]);

    if (input->expected_systems > 0) {
      // Full scene case - should have component
      wxr_assert_test(component_count == 1, "Expected: %d", 1, "Got: %ld",
                      component_count, "Component count doesn't match");
      if (component_count > 0) {
        wxr_assert_test(strcmp(components[0], "WXR_A") == 0, "Expected: %s",
                        "WXR_A", "Got: %s", components[0],
                        "Component name doesn't match");
      }
    } else {
      // Entity scene case - no components
      wxr_assert_test(component_count == 0, "Expected: %d", 0, "Got: %ld",
                      component_count, "Component count doesn't match");
    }

    destroy_char_array(components, component_count);
  }

  free(entities);
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *empty_scene = wxr_create_scene();
  WXR_Scene *entity_scene = wxr_create_scene();
  WXR_Scene *full_scene = wxr_create_scene();

  wxr_load_plugin(full_scene, "./libwasserxr_test_components.so");
  wxr_load_plugin(full_scene, "./libwasserxr_test_systems.so");

  TestCase cases[] = {
      {NULL, NULL, 0, 0, 1},
      {NULL, "", 0, 0, 1},
      {empty_scene,
       "\30\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0",
       0, 0, 0},
      {entity_scene,
       "\40\0\0\0\0\0\0\0"
       "\0\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\10\0\0\0\0\0\0\0",
       0, 1, 0},
      {full_scene,
       "\147\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\1\0\0\0\0\0\0\0"
       "\31\0\0\0\0\0\0\0wxr_system_a\0\144\0\0\0"
       "\66\0\0\0\0\0\0\0\56\0\0\0\0\0\0\0WXR_"
       "A\0\16\0\0\0\0\0\0\0x\0\1\0\0\0\22\0\0\0\0\0\0\0extra\0\5\0\0\0",
       1, 1, 0}};

  // Constructing Tests
  for (size_t i = 0; i < 5; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_deserialize_scene/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
