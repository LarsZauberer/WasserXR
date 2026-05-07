#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/ecs/logging.h>
#include <stdlib.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  TS_Entity entity;
  size_t expected_count;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    ts_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  
  size_t count = 0;
  char **components = ts_get_components_of_entity(&count, input->scene, input->entity);
  
  ts_assert(count == input->expected_count,
            "Component count should match expected");
  
  if (count > 0 && components != NULL) {
    for (size_t i = 0; i < count; i++) {
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
  TS_Scene *scene_empty_entity = ts_create_scene();
  TS_Entity empty_entity = ts_add_entity(scene_empty_entity);

  TS_Scene *scene_with_component = ts_create_scene();
  ts_assert(0 == ts_load_plugin(scene_with_component, "./libtheseed_components.so"),
            "Failed to load the plugin");
  TS_Entity entity_with_comp = ts_add_entity(scene_with_component);
  ts_add_component(scene_with_component, entity_with_comp, "TS_Transform");

  TestCase cases[] = {
      {scene_empty_entity, empty_entity, 0},
      {scene_with_component, entity_with_comp, 1},
  };

  // Constructing Tests
  for (size_t i = 0; i < 2; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_get_components_of_entity/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
