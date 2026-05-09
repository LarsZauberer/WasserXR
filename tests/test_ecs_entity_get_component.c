#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  WXR_Entity entity;
  char *component;
  int should_exist;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    wxr_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;

  void *component =
      wxr_entity_get_component(input->scene, input->entity, input->component);

  if (input->should_exist) {
    wxr_assert(component != NULL, "Component should exist");
  } else {
    wxr_assert(component == NULL, "Component should not exist");
  }
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *null_scene = NULL;

  WXR_Scene *empty_scene = wxr_create_scene();

  WXR_Scene *scene_with_component = wxr_create_scene();
  wxr_assert(
      0 == wxr_load_plugin(scene_with_component, "./libwasserxr_core.so"),
      "Failed to load the plugin");
  WXR_Entity entity_with_comp = wxr_add_entity(scene_with_component);
  wxr_add_component(scene_with_component, entity_with_comp, "WXR_Transform");

  WXR_Scene *scene_without_component = wxr_create_scene();
  wxr_assert(0 == wxr_load_plugin(scene_without_component,
                                "./libwasserxr_core.so"),
            "Failed to load the plugin");
  WXR_Entity entity_without_comp = wxr_add_entity(scene_without_component);

  WXR_Scene *scene_with_component2 = wxr_create_scene();
  wxr_assert(
      0 == wxr_load_plugin(scene_with_component2, "./libwasserxr_core.so"),
      "Failed to load the plugin");
  WXR_Entity entity_with_comp2 = wxr_add_entity(scene_with_component2);
  wxr_add_component(scene_with_component2, entity_with_comp, "WXR_Transform");

  TestCase cases[] = {
      {null_scene, 0, NULL, 0}, // NULL scene
      {empty_scene, 0, "", 0},  // Empty scene
      {scene_with_component, entity_with_comp, "WXR_Transform",
       1}, // Valid component
      {scene_without_component, entity_without_comp, "WXR_Transform",
       0},                                             // Non-existent component
      {scene_with_component2, 999, "WXR_Transform", 0}, // Invalid entity
  };

  // Constructing Tests
  for (size_t i = 0; i < 5; i++) {
    char *path =
        g_strdup_printf("/wasserxr/test_ecs_entity_get_component/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
