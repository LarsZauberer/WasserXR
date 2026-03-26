#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  TS_Entity entity;
  char *component;
  void *out;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  ts_destroy_scene(input->scene);
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  void *component =
      ts_add_component(input->scene, input->entity, input->component);
  if (input->out) {
    ts_assert(component != NULL, "Component is NULL (should not be NULL)");
  } else {
    ts_assert(component == NULL, "Component is not NULL (should be NULL)");
  }
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *empty_scene = ts_create_scene();

  TS_Scene *entity_scene = ts_create_scene();
  TS_Entity entity = ts_add_entity(entity_scene);

  TS_Scene *entity_plugin_scene_invalid = ts_create_scene();
  ts_assert(0 == ts_load_plugin(entity_plugin_scene_invalid,
                                "./libtheseed_components.so"),
            "Failed to load the plugin");
  TS_Entity entity2 = ts_add_entity(entity_plugin_scene_invalid);

  TS_Scene *entity_plugin_scene = ts_create_scene();
  ts_assert(
      0 == ts_load_plugin(entity_plugin_scene, "./libtheseed_components.so"),
      "Failed to load the plugin");
  TS_Entity entity3 = ts_add_entity(entity_plugin_scene);

  TestCase cases[] = {
      {NULL, 0, NULL, NULL},
      {NULL, 0, "", NULL},
      {empty_scene, 0, "", NULL},
      {entity_scene, entity, "", NULL},
      {entity_plugin_scene_invalid, entity2, "Ahh", NULL},
      {entity_plugin_scene, entity3, "TS_Transform", (void *)1},
  };

  // Constructing Tests

  for (size_t i = 0; i < 6; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_add_component/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
