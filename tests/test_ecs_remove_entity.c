#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/core/logging.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
  TS_Entity entity;
  int expected_result;
};

static void free_case(void *ptr) {
  TestCase *input = ptr;
  if (input->scene) {
    ts_destroy_scene(input->scene);
  }
}

static void unittest(const void *ptr) {
  const TestCase *input = ptr;
  
  int result = ts_remove_entity(input->scene, input->entity);
  ts_assert(result == input->expected_result,
            "Remove entity result should match expected");
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *null_scene = NULL;

  TS_Scene *empty_scene = ts_create_scene();

  TS_Scene *scene_with_entity = ts_create_scene();
  TS_Entity valid_entity = ts_add_entity(scene_with_entity);

  TS_Scene *scene_for_nonexistent = ts_create_scene();
  ts_add_entity(scene_for_nonexistent);

  TestCase cases[] = {
      {null_scene, 0, 1},                      // NULL scene should fail
      {empty_scene, 0, 1},                     // Non-existent entity should fail
      {scene_with_entity, valid_entity, 0},    // Valid removal should succeed
      {scene_for_nonexistent, 999, 1},         // Non-existent entity should fail
  };

  // Constructing Tests
  for (size_t i = 0; i < 4; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_remove_entity/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
