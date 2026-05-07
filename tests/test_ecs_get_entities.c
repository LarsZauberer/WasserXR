#include "TheSeed/ecs/Scene.h"
#include "glib.h"
#include <TheSeed/ecs/logging.h>
#include <stdlib.h>

typedef struct TestCase TestCase;

struct TestCase {
  TS_Scene *scene;
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
  TS_Entity *entities = ts_get_entities(&count, input->scene);
  
  ts_assert(count == input->expected_count,
            "Entity count should match expected");
  
  if (count > 0) {
    ts_assert(entities != NULL, "Entities array should not be NULL");
    free(entities);
  }
}

int main(int argc, char *argv[]) {
  g_test_init(&argc, &argv, NULL);

  ts_logging_init(TS_LOG_DEBUG);
  ts_add_logger(ts_stdout_logger);

  // Constructing Cases
  TS_Scene *empty_scene = ts_create_scene();

  TS_Scene *scene_one_entity = ts_create_scene();
  ts_add_entity(scene_one_entity);

  TS_Scene *scene_multiple_entities = ts_create_scene();
  ts_add_entity(scene_multiple_entities);
  ts_add_entity(scene_multiple_entities);
  ts_add_entity(scene_multiple_entities);

  TestCase cases[] = {
      {empty_scene, 0},
      {scene_one_entity, 1},
      {scene_multiple_entities, 3},
  };

  // Constructing Tests
  for (size_t i = 0; i < 3; i++) {
    char *path = g_strdup_printf("/theseed/test_ecs_get_entities/%ld", i);
    g_test_add_data_func_full(path, &cases[i], unittest, free_case);
    free(path);
  }

  return g_test_run();
}
