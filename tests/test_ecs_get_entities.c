#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>
#include <stdlib.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
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
  WXR_Entity *entities = wxr_get_entities(&count, input->scene);

  wxr_assert(count == input->expected_count,
             "Entity count should match expected");

  if (count > 0) {
    wxr_assert(entities != NULL, "Entities array should not be NULL");
    free(entities);
  }
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *empty_scene = wxr_create_scene();

  WXR_Scene *scene_one_entity = wxr_create_scene();
  wxr_add_entity(scene_one_entity);

  WXR_Scene *scene_multiple_entities = wxr_create_scene();
  wxr_add_entity(scene_multiple_entities);
  wxr_add_entity(scene_multiple_entities);
  wxr_add_entity(scene_multiple_entities);

  TestCase cases[] = {
      {empty_scene, 0},
      {scene_one_entity, 1},
      {scene_multiple_entities, 3},
  };

  // Constructing Tests
  for (size_t i = 0; i < 3; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_get_entities/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
