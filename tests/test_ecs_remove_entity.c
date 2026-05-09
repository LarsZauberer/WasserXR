#include "WasserXR/ecs/Scene.h"
#include "glib.h"
#include <WasserXR/ecs/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  WXR_Scene *scene;
  WXR_Entity entity;
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

  int result = wxr_remove_entity(input->scene, input->entity);
  wxr_assert(result == input->expected_result,
             "Remove entity result should match expected");
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  wxr_logging_init(WXR_LOG_DEBUG);
  wxr_add_logger(wxr_stdout_logger);

  // Constructing Cases
  WXR_Scene *null_scene = NULL;

  WXR_Scene *empty_scene = wxr_create_scene();

  WXR_Scene *scene_with_entity = wxr_create_scene();
  WXR_Entity valid_entity = wxr_add_entity(scene_with_entity);

  WXR_Scene *scene_for_nonexistent = wxr_create_scene();
  wxr_add_entity(scene_for_nonexistent);

  TestCase cases[] = {
      {null_scene, 0, 1},                   // NULL scene should fail
      {empty_scene, 0, 1},                  // Non-existent entity should fail
      {scene_with_entity, valid_entity, 0}, // Valid removal should succeed
      {scene_for_nonexistent, 999, 1},      // Non-existent entity should fail
  };

  // Constructing Tests
  for (size_t i = 0; i < 4; i++) {
    char *path = g_strdup_printf("/wasserxr/test_ecs_remove_entity/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
