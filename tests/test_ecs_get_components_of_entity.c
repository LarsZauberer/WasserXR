#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>
#include <stdlib.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  WXR_Entity entity;
  size_t expected_count;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    wxr_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  
  size_t count = 0;
  char **components = wxr_get_components_of_entity(&count, input->scene, input->entity);
  
  wxr_assert(count == input->expected_count,
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

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *scene_empty_entity = wxr_create_scene();
  WXR_Entity empty_entity = wxr_add_entity(scene_empty_entity);

  WXR_Scene *scene_with_component = wxr_create_scene();
  wxr_assert(0 == wxr_load_plugin(scene_with_component, "./libwasserxr_core.so"),
            "Failed to load the plugin");
  WXR_Entity entity_with_comp = wxr_add_entity(scene_with_component);
  wxr_add_component(scene_with_component, entity_with_comp, "WXR_Transform");

  TestCase cases[] = {
      {scene_empty_entity, empty_entity, 0},
      {scene_with_component, entity_with_comp, 1},
  };

  // Constructing Tests
  for (size_t i = 0; i < 2; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_get_components_of_entity/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
