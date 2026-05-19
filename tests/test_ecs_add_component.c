#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  WXR_Entity entity;
  char *component;
  void *out;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  wxr_destroy_scene(input->scene);
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  void *component =
      wxr_add_component(input->scene, input->entity, input->component);
  if (input->out) {
    wxr_assert(component != NULL, "Component is NULL (should not be NULL)");
  } else {
    wxr_assert(component == NULL, "Component is not NULL (should be NULL)");
  }
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *empty_scene = wxr_create_scene();

  WXR_Scene *entity_scene = wxr_create_scene();
  WXR_Entity entity = wxr_add_entity(entity_scene);

  WXR_Scene *entity_plugin_scene_invalid = wxr_create_scene();
  wxr_assert(0 == wxr_load_plugin(entity_plugin_scene_invalid,
                                  "./libwasserxr_test_components.so"),
             "Failed to load the plugin");
  WXR_Entity entity2 = wxr_add_entity(entity_plugin_scene_invalid);

  WXR_Scene *entity_plugin_scene = wxr_create_scene();
  wxr_assert(0 == wxr_load_plugin(entity_plugin_scene,
                                  "./libwasserxr_test_components.so"),
             "Failed to load the plugin");
  WXR_Entity entity3 = wxr_add_entity(entity_plugin_scene);

  TestCase cases[] = {
      {NULL, 0, NULL, NULL},
      {NULL, 0, "", NULL},
      {empty_scene, 0, "", NULL},
      {entity_scene, entity, "", NULL},
      {entity_plugin_scene_invalid, entity2, "Ahh", NULL},
      {entity_plugin_scene, entity3, "WXR_A", (void *)1},
  };

  // Constructing Tests

  for (size_t i = 0; i < 6; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_add_component/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
