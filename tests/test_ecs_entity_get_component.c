#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  TS_Entity entity;
  char *component;
  int should_exist;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    ts_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  
  void *component = ts_entity_get_component(input->scene, input->entity, input->component);
  
  if (input->should_exist) {
    ts_assert(component != NULL, "Component should exist");
  } else {
    ts_assert(component == NULL, "Component should not exist");
  }
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *null_scene = NULL;

  TS_Scene *empty_scene = ts_create_scene();

  TS_Scene *scene_with_component = ts_create_scene();
  ts_assert(0 == ts_load_plugin(scene_with_component, "./libtheseed_components.so"),
            "Failed to load the plugin");
  TS_Entity entity_with_comp = ts_add_entity(scene_with_component);
  ts_add_component(scene_with_component, entity_with_comp, "TS_Transform");

  TS_Scene *scene_without_component = ts_create_scene();
  ts_assert(0 == ts_load_plugin(scene_without_component, "./libtheseed_components.so"),
            "Failed to load the plugin");
  TS_Entity entity_without_comp = ts_add_entity(scene_without_component);

  TestCase cases[] = {
      {null_scene, 0, NULL, 0},                                    // NULL scene
      {empty_scene, 0, "", 0},                                     // Empty scene
      {scene_with_component, entity_with_comp, "TS_Transform", 1}, // Valid component
      {scene_without_component, entity_without_comp, "TS_Transform", 0}, // Non-existent component
      {scene_with_component, 999, "TS_Transform", 0},              // Invalid entity
  };

  // Constructing Tests
  for (size_t i = 0; i < 5; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_entity_get_component/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
