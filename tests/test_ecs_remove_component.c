#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  WXR_Entity entity;
  char *component;
  int expected_result;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    wxr_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;

  int result =
      wxr_remove_component(input->scene, input->entity, input->component);
  wxr_assert(result == input->expected_result,
             "Remove component result should match expected");
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *null_scene = NULL;

  WXR_Scene *empty_scene = wxr_create_scene();

  WXR_Scene *scene_with_component = wxr_create_scene();
  wxr_assert(0 == wxr_load_plugin(scene_with_component,
                                  "./libwasserxr_test_components.so"),
             "Failed to load the plugin");
  WXR_Entity entity_with_comp = wxr_add_entity(scene_with_component);
  wxr_add_component(scene_with_component, entity_with_comp, "WXR_A");

  WXR_Scene *scene_without_component = wxr_create_scene();
  wxr_assert(0 == wxr_load_plugin(scene_without_component,
                                  "./libwasserxr_test_components.so"),
             "Failed to load the plugin");
  WXR_Entity entity_without_comp = wxr_add_entity(scene_without_component);

  WXR_Scene *scene_with_component2 = wxr_create_scene();
  wxr_assert(0 == wxr_load_plugin(scene_with_component2,
                                  "./libwasserxr_test_components.so"),
             "Failed to load the plugin");
  WXR_Entity entity_with_comp2 = wxr_add_entity(scene_with_component);
  wxr_add_component(scene_with_component2, entity_with_comp, "WXR_A");

  TestCase cases[] = {
      {null_scene, 0, NULL, 1},                             // NULL scene
      {empty_scene, 0, "", 1},                              // Empty scene
      {scene_with_component, entity_with_comp, "WXR_A", 0}, // Valid removal
      {scene_without_component, entity_without_comp, "WXR_A",
       1},                                      // Non-existent component
      {scene_with_component2, 999, "WXR_A", 1}, // Invalid entity
  };

  // Constructing Tests
  for (size_t i = 0; i < 5; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_remove_component/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
